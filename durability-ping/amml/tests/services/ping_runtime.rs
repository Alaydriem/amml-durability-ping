use std::path::Path;
use std::time::Duration;

use amm_client::test_support::FakeTable;
use amm_core::structs::event::{KeybindPhase, ModEvent};
use amml_durability_ping::services::{PingLink, PingRuntime};
use amml_durability_ping::types::LinkState;
use durability_ping_core::audio::Cue;

use crate::support::FakeOutput;
use crate::support::fake_game::FakeGame;

const RATE: u32 = 48_000;
const CHANNELS: u16 = 2;
// the uses part alone, so the crossing is at 20 uses whatever the maximum is
const USES_ONLY: &str = "alert {\n  percent = 0\n  uses = 20\n}\n";

fn attached(output: &FakeOutput) -> PingRuntime {
    let runtime = PingRuntime::with_output(Path::new(FakeTable::CONFIG_DIR), output.clone());
    assert_eq!(runtime.attach_with(Ok(FakeTable::table())), LinkState::Attached);
    runtime
}

fn tick_uses(runtime: &PingRuntime, game: &FakeGame, uses: i32) {
    game.hold_uses(uses);
    runtime.tick_with(game);
}

// the audio thread is async; give a wrong extra play time to show up
fn settle() {
    std::thread::sleep(Duration::from_millis(100));
}

#[test]
fn tick_alerts_on_crossing() {
    FakeTable::set_config(USES_ONLY);
    let output = FakeOutput::default();
    let runtime = attached(&output);
    let game = FakeGame::playing();

    tick_uses(&runtime, &game, 21);
    tick_uses(&runtime, &game, 19);

    assert!(output.wait_for(|state| state.plays == 1));
    settle();
    output.read(|state| {
        assert_eq!(state.plays, 1);
        assert_eq!(state.samples[0], Cue::Warning.samples(RATE, CHANNELS, 60));
    });
}

// in game the first tone played on the loading screen, before the world showed (2026-11-08)
#[test]
fn no_alert_while_a_menu_shows_then_one_when_it_closes() {
    FakeTable::set_config(USES_ONLY);
    let output = FakeOutput::default();
    let runtime = attached(&output);
    let game = FakeGame::playing();

    game.set_menu(true);
    tick_uses(&runtime, &game, 10);
    tick_uses(&runtime, &game, 10);
    settle();
    output.read(|state| assert_eq!(state.plays, 0));

    game.set_menu(false);
    tick_uses(&runtime, &game, 10);
    assert!(output.wait_for(|state| state.plays == 1));
}

#[test]
fn a_menu_keeps_the_warning() {
    FakeTable::set_config(USES_ONLY);
    let output = FakeOutput::default();
    let runtime = attached(&output);
    let game = FakeGame::playing();

    tick_uses(&runtime, &game, 10);
    assert!(output.wait_for(|state| state.plays == 1));
    game.set_menu(true);
    tick_uses(&runtime, &game, 10);
    game.set_menu(false);
    tick_uses(&runtime, &game, 10);
    settle();
    output.read(|state| assert_eq!(state.plays, 1));
}

#[test]
fn alert_uses_volume_setting() {
    FakeTable::set_config("alert {\n  percent = 0\n  uses = 20\n  volume = 30\n}\n");
    let output = FakeOutput::default();
    let runtime = attached(&output);
    let game = FakeGame::playing();

    tick_uses(&runtime, &game, 21);
    tick_uses(&runtime, &game, 19);

    assert!(output.wait_for(|state| state.plays == 1));
    output.read(|state| assert_eq!(state.samples[0], Cue::Warning.samples(RATE, CHANNELS, 30)));
}

#[test]
fn alert_disabled_plays_nothing() {
    FakeTable::set_config("alert {\n  enabled = false\n  percent = 0\n  uses = 20\n}\n");
    let output = FakeOutput::default();
    let runtime = attached(&output);
    let game = FakeGame::playing();

    tick_uses(&runtime, &game, 21);
    tick_uses(&runtime, &game, 19);

    settle();
    output.read(|state| assert_eq!(state.plays, 0));
    assert_eq!(output.read(|state| state.opens), 0);
}

#[test]
fn no_player_resets_tracker() {
    FakeTable::set_config(USES_ONLY);
    let output = FakeOutput::default();
    let runtime = attached(&output);
    let game = FakeGame::playing();

    tick_uses(&runtime, &game, 10);
    assert!(output.wait_for(|state| state.plays == 1));

    game.set_player(0);
    tick_uses(&runtime, &game, 10);
    assert_eq!(runtime.local_player(), 0);
    game.set_player(FakeGame::PLAYER);
    tick_uses(&runtime, &game, 10);

    assert!(output.wait_for(|state| state.plays == 2));
    assert_eq!(runtime.local_player(), FakeGame::PLAYER);
}

#[test]
fn settings_changed_event_reloads() {
    FakeTable::set_config(USES_ONLY);
    let output = FakeOutput::default();
    let runtime = attached(&output);
    let game = FakeGame::playing();
    tick_uses(&runtime, &game, 21);
    assert!(runtime.config().alert.enabled);

    FakeTable::set_config("alert {\n  enabled = false\n  percent = 0\n  uses = 20\n}\n");
    FakeTable::push_event(ModEvent::SettingsChanged { paths: 1 });
    tick_uses(&runtime, &game, 19);

    assert!(!runtime.config().alert.enabled);
    settle();
    output.read(|state| assert_eq!(state.plays, 0));
}

#[test]
fn override_events_set_flag() {
    let output = FakeOutput::default();
    let runtime = attached(&output);
    let game = FakeGame::playing();
    assert!(!runtime.override_held());

    FakeTable::push_event(ModEvent::Keybind { action: PingLink::OVERRIDE_ACTION, phase: KeybindPhase::Pressed });
    runtime.tick_with(&game);
    assert!(runtime.override_held());

    FakeTable::push_event(ModEvent::Keybind { action: PingLink::OVERRIDE_ACTION, phase: KeybindPhase::Released });
    runtime.tick_with(&game);
    assert!(!runtime.override_held());
}

#[test]
fn cue_plays_the_cue_at_the_configured_volume() {
    FakeTable::set_config("alert {\n  volume = 45\n}\n");
    let output = FakeOutput::default();
    let runtime = attached(&output);

    runtime.cue(Cue::Blocked);

    assert!(output.wait_for(|state| state.plays == 1));
    output.read(|state| assert_eq!(state.samples[0], Cue::Blocked.samples(RATE, CHANNELS, 45)));
}

#[test]
fn detach_stops_the_audio_thread_and_the_hold() {
    let output = FakeOutput::default();
    let runtime = attached(&output);
    let game = FakeGame::playing();
    FakeTable::push_event(ModEvent::Keybind { action: PingLink::OVERRIDE_ACTION, phase: KeybindPhase::Pressed });
    runtime.tick_with(&game);
    runtime.cue(Cue::Warning);
    assert!(output.wait_for(|state| state.plays == 1));

    runtime.detach();

    assert_eq!(runtime.link_state(), LinkState::Disabled);
    assert!(!runtime.override_held());
    assert_eq!(output.read(|state| state.closes), 1);
    assert_eq!(FakeTable::with(|state| state.unregistered.len()), 1);
}

// the guard on, below 3 uses
const BLOCK_ON: &str = "block {\n  enabled = true\n  percent = 0\n  uses = 3\n}\n";

// attached with the guard on, after one tick that noted the local player
fn guarding(game: &FakeGame) -> PingRuntime {
    FakeTable::set_config(BLOCK_ON);
    let runtime = attached(&FakeOutput::default());
    runtime.tick_with(game);
    runtime
}

#[test]
fn blocks_only_local_player() {
    let game = FakeGame::playing();
    let runtime = guarding(&game);
    game.hold_uses(1);

    assert!(!runtime.should_block(0x9999, &game));
    assert!(!runtime.should_block(0, &game));
    game.set_player(0);
    runtime.tick_with(&game);
    assert!(!runtime.should_block(FakeGame::PLAYER, &game));
}

// a call from another player (a mob, or the server side of a local world) must not touch the game: it can run on
// another thread
#[test]
fn a_non_matching_player_makes_no_game_read() {
    let game = FakeGame::playing();
    let runtime = guarding(&game);
    game.hold_uses(1);
    let before = game.reads();

    assert!(!runtime.should_block(0x9999, &game));
    assert!(!runtime.should_block(0, &game));

    assert_eq!(game.reads(), before);
}

// a freed LocalPlayer address can be reused by a server-thread object: the thread check stops it before any read
#[test]
fn another_thread_is_not_blocked_and_makes_no_game_read() {
    let game = FakeGame::playing();
    let runtime = guarding(&game);
    game.hold_uses(1);

    let reads = std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let other = FakeGame::playing();
                other.hold_uses(1);
                assert!(!runtime.should_block(FakeGame::PLAYER, &other));
                other.reads()
            })
            .join()
            .unwrap()
    });

    assert_eq!(reads, 0);
}

#[test]
fn the_ticking_thread_still_blocks_after_another_thread_asked() {
    let game = FakeGame::playing();
    let runtime = guarding(&game);
    game.hold_uses(1);

    std::thread::scope(|scope| {
        scope.spawn(|| assert!(!runtime.should_block(FakeGame::PLAYER, &FakeGame::playing())));
    });

    assert!(runtime.should_block(FakeGame::PLAYER, &game));
}

#[test]
fn blocks_low_tool() {
    let game = FakeGame::playing();
    let runtime = guarding(&game);
    game.hold_uses(1);

    assert!(runtime.should_block(FakeGame::PLAYER, &game));
}

#[test]
fn a_tool_above_the_limit_and_an_empty_hand_pass() {
    let game = FakeGame::playing();
    let runtime = guarding(&game);

    assert!(!runtime.should_block(FakeGame::PLAYER, &game));
    game.hold_uses(500);
    assert!(!runtime.should_block(FakeGame::PLAYER, &game));
}

#[test]
fn override_lets_through() {
    let game = FakeGame::playing();
    let runtime = guarding(&game);
    game.hold_uses(1);
    FakeTable::push_event(ModEvent::Keybind { action: PingLink::OVERRIDE_ACTION, phase: KeybindPhase::Pressed });
    runtime.tick_with(&game);

    assert!(!runtime.should_block(FakeGame::PLAYER, &game));
}

#[test]
fn block_off_lets_through() {
    FakeTable::set_config(
        "block {
  enabled = false
  percent = 0
  uses = 3
}
",
    );
    let runtime = attached(&FakeOutput::default());
    let game = FakeGame::playing();
    runtime.tick_with(&game);
    game.hold_uses(1);

    assert!(!runtime.should_block(FakeGame::PLAYER, &game));
}

#[test]
fn not_attached_lets_through() {
    let game = FakeGame::playing();
    let runtime = guarding(&game);
    game.hold_uses(1);
    runtime.detach();

    assert!(!runtime.should_block(FakeGame::PLAYER, &game));
}

// Review Focus 1: no tick between the calls, so only a live read can follow the change
#[test]
fn reads_held_tool_live() {
    let game = FakeGame::playing();
    let runtime = guarding(&game);

    game.hold_uses(1);
    assert!(runtime.should_block(FakeGame::PLAYER, &game));
    game.hold_uses(500);
    assert!(!runtime.should_block(FakeGame::PLAYER, &game));
    game.hold_uses(2);
    assert!(runtime.should_block(FakeGame::PLAYER, &game));
}

#[test]
fn blocked_cue_limited() {
    FakeTable::set_config("alert {\n  volume = 45\n}\n");
    let output = FakeOutput::default();
    let runtime = attached(&output);

    for millis in [0, 300, 900, 1100] {
        runtime.note_blocked(Duration::from_millis(millis));
    }

    assert!(output.wait_for(|state| state.plays == 2));
    settle();
    output.read(|state| {
        assert_eq!(state.plays, 2);
        assert!(state.samples.iter().all(|samples| *samples == Cue::Blocked.samples(RATE, CHANNELS, 45)));
    });
}
