use std::time::Duration;

use alaydriem_durability_ping::services::PingLink;
use amm_core::structs::event::{KeybindPhase, ModEvent};
use durability_ping_core::audio::Cue;

use crate::support::{FakeView, PingRig};

const RATE: u32 = 48_000;
const CHANNELS: u16 = 2;
// the uses part alone, so the crossing is at 20 uses whatever the maximum is
const USES_ONLY: &str = "alert {\n  percent = 0\n  uses = 20\n}\n";
const START: &str = "player.block_break_start";

// the audio thread is async; give a wrong extra play time to show up
fn settle() {
    std::thread::sleep(Duration::from_millis(100));
}

#[test]
fn the_link_attaches_and_logs_it_once() {
    let rig = PingRig::with_config(USES_ONLY);

    rig.tick(&FakeView::no_player());
    rig.tick(&FakeView::no_player());

    assert!(rig.runtime.attached());
    assert_eq!(rig.count_logged("info @alaydriem/durability-ping attached to the mod manager"), 1);
}

#[test]
fn a_crossing_plays_one_warning_and_logs_it() {
    let rig = PingRig::with_config(USES_ONLY);

    rig.tick(&FakeView::holding(21));
    rig.tick(&FakeView::holding(19));
    rig.tick(&FakeView::holding(18));

    assert!(rig.output.wait_for(|state| state.plays == 1));
    settle();
    rig.output.read(|state| {
        assert_eq!(state.plays, 1);
        assert_eq!(state.samples[0], Cue::Warning.samples(RATE, CHANNELS, 60));
    });
    assert!(rig.logged("info alert played slot=0 item=minecraft:iron_pickaxe remaining=19"));
}

#[test]
fn the_held_read_is_logged_once_per_world() {
    let rig = PingRig::with_config(USES_ONLY);

    rig.tick(&FakeView::holding(100));
    rig.tick(&FakeView::holding(100));
    rig.runtime.world_left();
    rig.tick(&FakeView::holding(100));

    assert_eq!(rig.count_logged("info held read slot=0 empty=false"), 2);
}

#[test]
fn no_alert_while_a_menu_shows_then_one_when_it_closes() {
    let rig = PingRig::with_config(USES_ONLY);
    rig.tick(&FakeView::holding(21));
    let mut menu = FakeView::holding(19);
    menu.menu = true;

    rig.tick(&menu);
    settle();
    assert_eq!(rig.output.read(|state| state.plays), 0);

    rig.tick(&FakeView::holding(19));
    assert!(rig.output.wait_for(|state| state.plays == 1));
}

#[test]
fn no_player_resets_the_tracker() {
    let rig = PingRig::with_config(USES_ONLY);
    rig.tick(&FakeView::holding(21));
    rig.tick(&FakeView::holding(19));
    assert!(rig.output.wait_for(|state| state.plays == 1));

    rig.tick(&FakeView::no_player());
    rig.tick(&FakeView::holding(21));
    rig.tick(&FakeView::holding(19));

    assert!(rig.output.wait_for(|state| state.plays == 2));
}

#[test]
fn a_settings_change_is_read_at_the_next_tick() {
    let rig = PingRig::with_config(USES_ONLY);
    rig.tick(&FakeView::no_player());
    assert!(!rig.runtime.config().block.enabled);

    amm_client::test_support::FakeTable::set_config("block {\n  enabled = true\n}\n");
    rig.push(ModEvent::SettingsChanged { paths: 1 });
    rig.tick(&FakeView::no_player());

    assert!(rig.runtime.config().block.enabled);
}

#[test]
fn the_guard_refuses_a_low_tool_of_the_local_player() {
    let rig = PingRig::guarding();

    assert!(rig.runtime.action(START, true, Some(&FakeView::holding(1))));
    assert!(rig.logged("info guard refused kind=player.block_break_start"));
    assert!(rig.output.wait_for(|state| state.plays == 1));
}

#[test]
fn another_player_is_never_refused_and_makes_no_game_read() {
    let rig = PingRig::guarding();
    let view = FakeView::holding(1);

    assert!(!rig.runtime.action(START, false, Some(&view)));
    assert_eq!(view.reads.get(), 0);
}

#[test]
fn a_tool_above_the_limit_and_an_empty_hand_pass() {
    let rig = PingRig::guarding();
    let mut empty = FakeView::holding(1);
    empty.tool = None;

    assert!(!rig.runtime.action(START, true, Some(&FakeView::holding(50))));
    assert!(!rig.runtime.action(START, true, Some(&empty)));
    assert!(!rig.runtime.action(START, true, None::<&FakeView>));
}

#[test]
fn the_override_key_lets_the_action_through_until_it_is_released() {
    let rig = PingRig::guarding();
    rig.push(ModEvent::Keybind { action: PingLink::OVERRIDE_ACTION, phase: KeybindPhase::Pressed });
    rig.tick(&FakeView::holding(1));
    assert!(!rig.runtime.action(START, true, Some(&FakeView::holding(1))));

    rig.push(ModEvent::Keybind { action: PingLink::OVERRIDE_ACTION, phase: KeybindPhase::Released });
    rig.tick(&FakeView::holding(1));
    assert!(rig.runtime.action(START, true, Some(&FakeView::holding(1))));
}

#[test]
fn the_guard_off_lets_everything_through() {
    let rig = PingRig::with_config(USES_ONLY);
    rig.tick(&FakeView::holding(100));

    assert!(!rig.runtime.action(START, true, Some(&FakeView::holding(1))));
}

#[test]
fn the_blocked_tone_and_its_line_come_at_most_once_per_gap() {
    let rig = PingRig::guarding();

    for _ in 0..5 {
        assert!(rig.runtime.action(START, true, Some(&FakeView::holding(1))));
    }

    assert_eq!(rig.count_logged("info guard refused"), 1);
    assert!(rig.output.wait_for(|state| state.plays == 1));
    settle();
    assert_eq!(rig.output.read(|state| state.plays), 1);
}

#[test]
fn each_action_kind_is_logged_once_per_world() {
    let rig = PingRig::guarding();
    let view = FakeView::holding(100);

    rig.runtime.action(START, true, Some(&view));
    rig.runtime.action(START, true, Some(&view));
    rig.runtime.action("player.attack", false, Some(&view));
    rig.runtime.world_left();
    rig.runtime.action(START, true, Some(&view));

    assert_eq!(rig.count_logged("info action seen kind=player.block_break_start local=true"), 2);
    assert!(rig.logged("info action seen kind=player.attack local=false"));
}

#[test]
fn detach_ends_the_hold_and_lets_everything_through() {
    let rig = PingRig::guarding();
    rig.push(ModEvent::Keybind { action: PingLink::OVERRIDE_ACTION, phase: KeybindPhase::Pressed });
    rig.tick(&FakeView::holding(1));
    assert!(rig.runtime.override_held());

    rig.runtime.detach();

    assert!(!rig.runtime.override_held());
    assert!(!rig.runtime.action(START, true, Some(&FakeView::holding(1))));
}
