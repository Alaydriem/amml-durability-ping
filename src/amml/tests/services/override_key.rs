use alaydriem_durability_ping::services::{OverrideKey, PingLink};
use amm_core::structs::event::{KeybindPhase, ModEvent};

fn keybind(action: u16, phase: KeybindPhase) -> ModEvent {
    ModEvent::Keybind { action, phase }
}

#[test]
fn starts_not_held() {
    assert!(!OverrideKey::default().held());
}

#[test]
fn pressed_is_held_and_repeat_keeps_it() {
    let mut key = OverrideKey::default();

    key.apply(&keybind(PingLink::OVERRIDE_ACTION, KeybindPhase::Pressed));
    assert!(key.held());
    key.apply(&keybind(PingLink::OVERRIDE_ACTION, KeybindPhase::Repeat));
    assert!(key.held());
}

#[test]
fn released_is_not_held() {
    let mut key = OverrideKey::default();
    key.apply(&keybind(PingLink::OVERRIDE_ACTION, KeybindPhase::Pressed));

    key.apply(&keybind(PingLink::OVERRIDE_ACTION, KeybindPhase::Released));

    assert!(!key.held());
}

#[test]
fn repeat_alone_does_not_hold() {
    let mut key = OverrideKey::default();

    key.apply(&keybind(PingLink::OVERRIDE_ACTION, KeybindPhase::Repeat));

    assert!(!key.held());
}

#[test]
fn disabled_clears_the_hold() {
    let mut key = OverrideKey::default();
    key.apply(&keybind(PingLink::OVERRIDE_ACTION, KeybindPhase::Pressed));

    key.apply(&ModEvent::Disabled);

    assert!(!key.held());
}

#[test]
fn another_action_changes_nothing() {
    let mut key = OverrideKey::default();
    key.apply(&keybind(PingLink::OVERRIDE_ACTION + 1, KeybindPhase::Pressed));
    assert!(!key.held());

    key.apply(&keybind(PingLink::OVERRIDE_ACTION, KeybindPhase::Pressed));
    key.apply(&keybind(PingLink::OVERRIDE_ACTION + 1, KeybindPhase::Released));
    assert!(key.held());
}
