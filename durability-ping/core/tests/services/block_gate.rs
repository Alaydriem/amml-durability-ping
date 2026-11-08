use durability_ping_core::config::BlockConfig;
use durability_ping_core::services::BlockGate;
use durability_ping_core::structs::{Limit, ToolReading};

fn config() -> BlockConfig {
    BlockConfig { enabled: true, limit: Limit { percent: 2, uses: 3 } }
}

fn reading(max: i32, remaining: i32) -> ToolReading {
    ToolReading { slot: 0, item: "x".to_string(), remaining, max }
}

#[test]
fn low_tool_blocks() {
    assert!(BlockGate::blocks(Some(&reading(1561, 3)), &config(), false));
}

#[test]
fn override_allows() {
    assert!(!BlockGate::blocks(Some(&reading(1561, 3)), &config(), true));
}

#[test]
fn disabled_never_blocks() {
    let mut c = config();
    c.enabled = false;
    assert!(!BlockGate::blocks(Some(&reading(1561, 3)), &c, false));
}

#[test]
fn empty_hand_never_blocks() {
    assert!(!BlockGate::blocks(None, &config(), false));
}

#[test]
fn tool_above_limit_does_not_block() {
    assert!(!BlockGate::blocks(Some(&reading(1561, 400)), &config(), false));
}

#[test]
fn off_limit_does_not_block() {
    let mut c = config();
    c.limit = Limit { percent: 0, uses: 0 };
    assert!(!BlockGate::blocks(Some(&reading(1561, 0)), &c, false));
}
