use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};
use std::thread::ThreadId;
use std::time::{Duration, Instant};

use amm_abi::types::AmmStatus;
use amm_client::link::ApiTable;
use amm_core::errors::LinkError;
use amm_core::structs::event::ModEvent;
use amml_sdk::{HookHandle, ModRuntime};
use durability_ping_core::audio::Cue;
use durability_ping_core::config::PingConfig;
use durability_ping_core::services::{AlertTracker, BlockGate, CueLimiter};

use crate::services::{CuePlayer, HeldTool, OverrideKey, PingLink, RodioOutput};
use crate::traits::{CueOutput, GameView};
use crate::types::LinkState;

// the manager link, the alert state and the override key of one attach; they live and go together
struct Active {
    link: PingLink,
    tracker: AlertTracker,
    key: OverrideKey,
}

// starts the audio thread; a test hands in a fake output
type PlayerFactory = Box<dyn Fn() -> CuePlayer + Send + Sync>;

// what the hook shares: the game client, the manager link state, the installed hooks and the audio thread.
// The mod manager is reached on the first ClientInstance::update after enable, when every mod of the host
// has loaded; `hooks` holds at most HOOK_CAPACITY handles. The audio thread starts on the first cue
pub struct PingRuntime {
    held: HeldTool,
    config_dir: PathBuf,
    link_state: AtomicU8,
    active: Mutex<Option<Active>>,
    config: Mutex<PingConfig>,
    local_player: AtomicUsize,
    override_held: AtomicBool,
    player: Mutex<Option<CuePlayer>>,
    factory: PlayerFactory,
    hooks: Mutex<Vec<HookHandle>>,
    started: Instant,
    blocked_cue: Mutex<CueLimiter>,
    game_mode_player: AtomicUsize,
    // the thread of the last tick: the client thread
    client_thread: Mutex<Option<ThreadId>>,
    // the log lines of enable, sent through curia to the mod manager's log once the link attaches
    startup: Mutex<Vec<(curia::Level, String)>>,
}

static RUNTIME: OnceLock<PingRuntime> = OnceLock::new();

impl PingRuntime {
    pub const HOOK_CAPACITY: usize = 8;

    pub fn new(config_dir: &Path) -> Self {
        Self::with_factory(config_dir, Box::new(|| CuePlayer::start(RodioOutput::new())))
    }

    // the runtime with its own output, so a test never touches the sound device
    pub fn with_output<O: CueOutput + Clone + Sync>(config_dir: &Path, output: O) -> Self {
        Self::with_factory(config_dir, Box::new(move || CuePlayer::start(output.clone())))
    }

    fn with_factory(config_dir: &Path, factory: PlayerFactory) -> Self {
        Self {
            held: HeldTool::new(),
            config_dir: config_dir.to_path_buf(),
            link_state: AtomicU8::new(LinkState::Waiting.to_u8()),
            active: Mutex::new(None),
            config: Mutex::new(PingConfig::default()),
            local_player: AtomicUsize::new(0),
            override_held: AtomicBool::new(false),
            player: Mutex::new(None),
            factory,
            hooks: Mutex::new(Vec::with_capacity(Self::HOOK_CAPACITY)),
            started: Instant::now(),
            blocked_cue: Mutex::new(CueLimiter::new()),
            game_mode_player: AtomicUsize::new(0),
            client_thread: Mutex::new(None),
            startup: Mutex::new(Vec::new()),
        }
    }

    // the runtime of this DLL; the first call wins
    pub fn install(config_dir: &Path) -> &'static PingRuntime {
        RUNTIME.get_or_init(|| Self::new(config_dir))
    }

    pub fn get() -> Option<&'static PingRuntime> {
        RUNTIME.get()
    }

    pub fn client(&self) -> &HeldTool {
        &self.held
    }

    pub fn link_state(&self) -> LinkState {
        LinkState::from_u8(self.link_state.load(Ordering::SeqCst))
    }

    // enable: the next tick reaches the manager again
    pub fn enable_link(&self) {
        self.link_state.store(LinkState::Waiting.to_u8(), Ordering::SeqCst);
    }

    // the local player of the last tick; 0 when there was none
    pub fn local_player(&self) -> usize {
        self.local_player.load(Ordering::Relaxed)
    }

    // whether the override key is down, as of the last tick
    pub fn override_held(&self) -> bool {
        self.override_held.load(Ordering::Relaxed)
    }

    // a copy of the settings of the last attach or reload
    pub fn config(&self) -> PingConfig {
        *self.config.lock().unwrap_or_else(PoisonError::into_inner)
    }

    // once per ClientInstance::update on the game thread, after the original: keeps the client, reaches the
    // manager on the first call after enable, then handles the manager's events and the held tool
    pub fn tick(&self, client: usize) {
        self.held.capture(client);
        self.tick_with(&self.held);
    }

    // whether the caller runs on the thread of the last tick (the client thread); false before any tick and after detach
    fn on_client_thread(&self) -> bool {
        *self.client_thread.lock().unwrap_or_else(PoisonError::into_inner) == Some(std::thread::current().id())
    }

    pub fn tick_with<G: GameView>(&self, game: &G) {
        *self.client_thread.lock().unwrap_or_else(PoisonError::into_inner) = Some(std::thread::current().id());
        if self.link_state() == LinkState::Waiting {
            self.attach_with(ApiTable::resolve());
        }
        if self.link_state() == LinkState::Attached {
            self.run(game);
        }
    }

    // registers with the manager the table belongs to; on failure the mod stays off and one line is logged.
    // Only a Waiting runtime takes the result: if disable ran meanwhile, a new link is dropped again
    pub fn attach_with(&self, table: Result<ApiTable, LinkError>) -> LinkState {
        let state = match table.and_then(|table| PingLink::new(table, &self.config_dir)) {
            Ok(link) => {
                *self.config.lock().unwrap_or_else(PoisonError::into_inner) = *link.config();
                let active = Active { link, tracker: AlertTracker::new(), key: OverrideKey::default() };
                *self.active.lock().unwrap_or_else(PoisonError::into_inner) = Some(active);
                curia::info!("@amml/durability-ping attached to the mod manager");
                self.send_startup_lines();
                LinkState::Attached
            }
            Err(error) => {
                Self::log_link_error(&error);
                LinkState::Off
            }
        };
        let waiting = LinkState::Waiting.to_u8();
        match self.link_state.compare_exchange(waiting, state.to_u8(), Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => state,
            Err(current) => {
                if state == LinkState::Attached {
                    drop(self.active.lock().unwrap_or_else(PoisonError::into_inner).take());
                }
                LinkState::from_u8(current)
            }
        }
    }

    // keeps a log line of enable for the mod manager's log; enable starts a new list
    pub fn note_startup(&self, level: curia::Level, line: &str) {
        self.startup.lock().unwrap_or_else(PoisonError::into_inner).push((level, line.to_string()));
    }

    pub fn clear_startup(&self) {
        self.startup.lock().unwrap_or_else(PoisonError::into_inner).clear();
    }

    // the kept lines of enable, through curia: the link installed the logger that forwards to the mod manager
    fn send_startup_lines(&self) {
        let lines = std::mem::take(&mut *self.startup.lock().unwrap_or_else(PoisonError::into_inner));
        for (level, line) in lines {
            match level {
                curia::Level::Error => curia::error!(line),
                curia::Level::Warn => curia::warn!(line),
                _ => curia::info!(line),
            }
        }
    }

    // disable: the mod unregisters, the override hold ends and the audio thread stops; no tick reaches the manager
    // until the next enable
    pub fn detach(&self) {
        self.link_state.store(LinkState::Disabled.to_u8(), Ordering::SeqCst);
        drop(self.active.lock().unwrap_or_else(PoisonError::into_inner).take());
        self.override_held.store(false, Ordering::Relaxed);
        self.local_player.store(0, Ordering::Relaxed);
        *self.client_thread.lock().unwrap_or_else(PoisonError::into_inner) = None;
        let player = self.player.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(player) = player {
            player.stop();
        }
    }

    // plays `cue` at the alert volume; the audio thread starts on the first call. Nothing plays once disabled
    pub fn cue(&self, cue: Cue) {
        if self.link_state() == LinkState::Disabled {
            return;
        }
        let volume = self.config().alert.volume;
        let mut player = self.player.lock().unwrap_or_else(PoisonError::into_inner);
        player.get_or_insert_with(|| (self.factory)()).play(cue, volume);
    }

    // whether the action `player` started must be stopped: only the local player's, only while attached, and only
    // when the guard is on, the override key is up and the tool in the hand is below the block limit.
    // The identity test uses the local player the last tick stored (an atomic), so a call for any other player (a
    // mob, or the server side of a local world, which can run on the server thread) makes no game read. Only after
    // a match, on the client's own GameMode call (the client thread), is the held tool read from the game, live at
    // the call: no decision of an earlier tick is kept
    // The thread test comes first: a freed LocalPlayer address can be reused by a server-thread object, so the
    // address match alone does not prove the client thread. Only the thread that ticked passes
    pub fn should_block<G: GameView>(&self, player: usize, game: &G) -> bool {
        if !self.on_client_thread() {
            return false;
        }
        if player == 0 || player != self.local_player() || self.link_state() != LinkState::Attached {
            return false;
        }
        let config = self.config().block;
        let override_held = self.override_held();
        if !config.enabled || override_held {
            return false;
        }
        BlockGate::blocks(game.held_tool().as_ref(), &config, override_held)
    }

    // a stopped action: the blocked tone at the alert volume, at most once per CueLimiter::GAP. `now` is the time
    // since the runtime started (elapsed)
    pub fn note_blocked(&self, now: Duration) {
        let allow = self.blocked_cue.lock().unwrap_or_else(PoisonError::into_inner).allow(now);
        if allow {
            self.cue(Cue::Blocked);
        }
    }

    // the monotonic time since the runtime was made
    pub fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    // the body of a guard detour: true, with the blocked tone, when the action `player` started must be stopped
    pub fn refuse(&self, player: usize) -> bool {
        if !self.should_block(player, &self.held) {
            return false;
        }
        self.note_blocked(self.elapsed());
        true
    }

    // the offset of GameMode::mPlayer (Player&) in GameMode; 0 while unknown, and then no GameMode hook is in
    pub fn game_mode_player_offset(&self) -> usize {
        self.game_mode_player.load(Ordering::Relaxed)
    }

    pub fn set_game_mode_player_offset(&self, offset: usize) {
        self.game_mode_player.store(offset, Ordering::Relaxed);
    }

    // keeps an installed hook; false, keeping nothing, when HOOK_CAPACITY hooks are already kept
    pub fn keep_hook(&self, hook: HookHandle) -> bool {
        let mut hooks = self.hooks.lock().unwrap_or_else(PoisonError::into_inner);
        if hooks.len() >= Self::HOOK_CAPACITY {
            return false;
        }
        hooks.push(hook);
        true
    }

    // the installed hooks, newest first, leaving none behind
    pub fn take_hooks(&self) -> Vec<HookHandle> {
        let mut hooks = self.hooks.lock().unwrap_or_else(PoisonError::into_inner);
        let mut taken = std::mem::take(&mut *hooks);
        taken.reverse();
        taken
    }

    fn run<G: GameView>(&self, game: &G) {
        let warn = {
            let Ok(mut slot) = self.active.lock() else { return };
            let Some(active) = slot.as_mut() else { return };
            for event in active.link.drain() {
                match event {
                    ModEvent::SettingsChanged { .. } => {
                        let config = *active.link.reload();
                        *self.config.lock().unwrap_or_else(PoisonError::into_inner) = config;
                    }
                    other => active.key.apply(&other),
                }
            }
            self.override_held.store(active.key.held(), Ordering::Relaxed);
            let player = game.local_player();
            self.local_player.store(player, Ordering::Relaxed);
            // no player (a menu, or between worlds): the next world starts with every tool armed
            if player == 0 {
                active.tracker.reset();
            }
            let reading = if player == 0 { None } else { game.held_tool() };
            // while a menu shows (the loading screen, the inventory, the pause menu) the tracker waits: a warning
            // plays when the screen closes, and a menu does not clear one
            if player != 0 && game.showing_menu() {
                false
            } else {
                active.tracker.observe(reading.as_ref(), &self.config().alert)
            }
        };
        if warn {
            self.cue(Cue::Warning);
        }
    }

    fn log_link_error(error: &LinkError) {
        let Some(runtime) = ModRuntime::get() else { return };
        let unsupported = LinkError::from(AmmStatus::Unsupported);
        match error {
            LinkError::ManagerMissing | LinkError::AbiMismatch { .. } => {
                runtime.logger().error("@amml/durability-ping needs @amml/modmanager 0.2.x; the alert is off");
            }
            other if *other == unsupported => {
                runtime
                    .logger()
                    .warn("@amml/modmanager is too old for @amml/durability-ping; update it. The alert is off");
            }
            other => {
                runtime
                    .logger()
                    .error(&format!("@amml/modmanager refused @amml/durability-ping ({other}); the alert is off"));
            }
        }
    }
}
