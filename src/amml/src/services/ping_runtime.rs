use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;
use std::time::Instant;

use amm_client::link::LinkState;
use amm_core::structs::event::ModEvent;
use amml_api::Logger;
use durability_ping_core::audio::Cue;
use durability_ping_core::config::PingConfig;
use durability_ping_core::services::{AlertTracker, BlockGate, CueLimiter};

use crate::services::{CuePlayer, OverrideKey, PingLink, RodioOutput};
use crate::traits::{CueOutput, GameView};

// starts the audio thread; a test hands in a fake output
type PlayerFactory = Box<dyn Fn() -> CuePlayer + Send + Sync>;

// the state of one world session: the alert tracker, whether the held read was logged and the action kinds seen
struct Session {
    tracker: AlertTracker,
    held_logged: bool,
    kinds_seen: Vec<&'static str>,
}

impl Session {
    fn new() -> Self {
        Self { tracker: AlertTracker::new(), held_logged: false, kinds_seen: Vec::new() }
    }
}

// what the listeners share: the manager link, the settings, the override key, the world session and the audio
// thread. Lock order is link, then key, then session. The audio thread starts on the first cue
pub struct PingRuntime {
    log: Logger,
    link: Mutex<Option<PingLink>>,
    session: Mutex<Session>,
    key: Mutex<OverrideKey>,
    config: Mutex<PingConfig>,
    attached: AtomicBool,
    override_held: AtomicBool,
    player: Mutex<Option<CuePlayer>>,
    factory: PlayerFactory,
    started: Instant,
    blocked_cue: Mutex<CueLimiter>,
}

impl PingRuntime {
    pub fn new(log: Logger) -> Self {
        Self::with_factory(log, Box::new(|| CuePlayer::start(RodioOutput::new())))
    }

    // the runtime with its own output, so a test never touches the sound device
    pub fn with_output<O: CueOutput + Clone + Sync>(log: Logger, output: O) -> Self {
        Self::with_factory(log, Box::new(move || CuePlayer::start(output.clone())))
    }

    fn with_factory(log: Logger, factory: PlayerFactory) -> Self {
        Self {
            log,
            link: Mutex::new(None),
            session: Mutex::new(Session::new()),
            key: Mutex::new(OverrideKey::default()),
            config: Mutex::new(PingConfig::default()),
            attached: AtomicBool::new(false),
            override_held: AtomicBool::new(false),
            player: Mutex::new(None),
            factory,
            started: Instant::now(),
            blocked_cue: Mutex::new(CueLimiter::new()),
        }
    }

    // enable: the mod works with this link until `detach`; the settings are read once it connects
    pub fn attach(&self, link: PingLink) {
        let mut slot = Self::lock(&self.link);
        drop(slot.take());
        *slot = Some(link);
        self.attached.store(false, Ordering::SeqCst);
    }

    // disable: the link closes, the override hold ends and the audio thread stops
    pub fn detach(&self) {
        let link = Self::lock(&self.link).take();
        self.attached.store(false, Ordering::SeqCst);
        self.override_held.store(false, Ordering::Relaxed);
        *Self::lock(&self.key) = OverrideKey::default();
        drop(link);
        let player = Self::lock(&self.player).take();
        if let Some(player) = player {
            player.stop();
        }
    }

    // whether the link is connected and the settings were read
    pub fn attached(&self) -> bool {
        self.attached.load(Ordering::SeqCst)
    }

    // whether the override key is down, as of the last tick
    pub fn override_held(&self) -> bool {
        self.override_held.load(Ordering::Relaxed)
    }

    // a copy of the settings of the last attach or reload
    pub fn config(&self) -> PingConfig {
        *Self::lock(&self.config)
    }

    // the time since the runtime was made
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    // once per client tick on the game thread: the manager's events, then the held tool. `game` is None when the
    // tick has no client
    pub fn tick<G: GameView>(&self, game: Option<&G>) {
        if !self.pump_link() {
            return;
        }
        let warn = {
            let mut session = Self::lock(&self.session);
            let Some(game) = game.filter(|game| game.has_local_player()) else {
                // no player (a menu, or between worlds): the next world starts with every tool armed
                *session = Session::new();
                return;
            };
            if !session.held_logged
                && let Some((slot, empty)) = game.held_slot()
            {
                session.held_logged = true;
                self.log.info("held read", &[("slot", &slot.to_string()), ("empty", &empty.to_string())]);
            }
            let reading = game.held_tool();
            // while a menu shows (the loading screen, the inventory, the pause menu) the tracker waits: a warning
            // plays when the screen closes, and a menu does not clear one
            if game.showing_menu() {
                None
            } else if session.tracker.observe(reading.as_ref(), &self.config().alert) {
                reading
            } else {
                None
            }
        };
        if let Some(reading) = warn {
            self.log.info(
                "alert played",
                &[
                    ("slot", &reading.slot.to_string()),
                    ("item", &reading.item),
                    ("remaining", &reading.remaining.to_string()),
                ],
            );
            self.cue(Cue::Warning);
        }
    }

    // a world was left: the tracker, the held read line and the seen kinds start again
    pub fn world_left(&self) {
        *Self::lock(&self.session) = Session::new();
    }

    // one player action from the loader; true when the action must be stopped. The blocked tone plays at most once
    // per CueLimiter::GAP, with one `guard refused` line. Only the local player's action is judged, with the held
    // tool read live at the call: no decision of an earlier tick is kept
    pub fn action<G: GameView>(&self, kind: &'static str, local: bool, game: Option<&G>) -> bool {
        self.note_seen(kind, local);
        if !local || !self.attached() {
            return false;
        }
        let config = self.config().block;
        let override_held = self.override_held();
        if !config.enabled || override_held {
            return false;
        }
        let reading = game.and_then(|game| game.held_tool());
        if !BlockGate::blocks(reading.as_ref(), &config, override_held) {
            return false;
        }
        if Self::lock(&self.blocked_cue).allow(self.elapsed()) {
            self.log.info("guard refused", &[("kind", kind)]);
            self.cue(Cue::Blocked);
        }
        true
    }

    // plays `cue` at the alert volume; the audio thread starts on the first call. Nothing plays while detached
    pub fn cue(&self, cue: Cue) {
        if !self.attached() {
            return;
        }
        let volume = self.config().alert.volume;
        let mut player = Self::lock(&self.player);
        player.get_or_insert_with(|| (self.factory)()).play(cue, volume);
    }

    // logs the first action of each kind in a world
    fn note_seen(&self, kind: &'static str, local: bool) {
        let mut session = Self::lock(&self.session);
        if session.kinds_seen.contains(&kind) {
            return;
        }
        session.kinds_seen.push(kind);
        self.log.info("action seen", &[("kind", kind), ("local", &local.to_string())]);
    }

    // reads the manager's events, and the settings once the link is connected. False while the link is not
    // connected
    fn pump_link(&self) -> bool {
        let mut slot = Self::lock(&self.link);
        let Some(link) = slot.as_mut() else { return false };
        let connected = link.state() == LinkState::Connected;
        if connected && !self.attached() {
            *Self::lock(&self.config) = link.settings();
            self.attached.store(true, Ordering::SeqCst);
            self.log.info("@alaydriem/durability-ping attached to the mod manager", &[]);
        }
        let mut reload = false;
        let mut key = Self::lock(&self.key);
        for event in link.drain() {
            match event {
                ModEvent::SettingsChanged { .. } => reload = true,
                // the manager is gone: no release comes, so the hold ends here
                ModEvent::ManagerGone => key.apply(&ModEvent::Disabled),
                other => key.apply(other),
            }
        }
        self.override_held.store(key.held(), Ordering::Relaxed);
        drop(key);
        if !connected {
            self.attached.store(false, Ordering::SeqCst);
            return false;
        }
        if reload {
            *Self::lock(&self.config) = link.settings();
        }
        true
    }

    // the runtime's state is plain data, so a lock poisoned by a panic is recovered
    fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        mutex.lock().unwrap_or_else(PoisonError::into_inner)
    }
}
