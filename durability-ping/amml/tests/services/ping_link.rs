use std::path::Path;

use amm_client::test_support::FakeTable;
use amm_core::structs::event::{KeybindPhase, ModEvent};
use amml_durability_ping::services::PingLink;
use durability_ping_core::config::PingConfig;

#[test]
fn link_reads_the_defaults_and_watches_no_keys() {
    let link = PingLink::new(FakeTable::table(), Path::new(FakeTable::CONFIG_DIR)).unwrap();

    assert_eq!(*link.config(), PingConfig::default());
    assert!(FakeTable::with(|state| state.watched()).is_empty());
}

#[test]
fn reload_reads_the_new_settings() {
    let mut link = PingLink::new(FakeTable::table(), Path::new(FakeTable::CONFIG_DIR)).unwrap();
    FakeTable::set_config("block {\n  enabled = true\n}\n");

    assert!(link.reload().block.enabled);
}

#[test]
fn drain_returns_the_queued_events_in_order() {
    let mut link = PingLink::new(FakeTable::table(), Path::new(FakeTable::CONFIG_DIR)).unwrap();
    let event = ModEvent::Keybind { action: PingLink::OVERRIDE_ACTION, phase: KeybindPhase::Pressed };
    FakeTable::push_event(event);
    FakeTable::push_event(ModEvent::Disabled);

    assert_eq!(link.drain(), [event, ModEvent::Disabled]);
    assert!(link.drain().is_empty());
}
