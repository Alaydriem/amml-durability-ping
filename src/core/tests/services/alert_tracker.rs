use durability_ping_core::config::AlertConfig;
use durability_ping_core::services::AlertTracker;
use durability_ping_core::structs::{Limit, ToolReading};

fn config() -> AlertConfig {
    AlertConfig { enabled: true, limit: Limit { percent: 10, uses: 20 }, volume: 60 }
}

fn tool(slot: i32, item: &str, remaining: i32, max: i32) -> ToolReading {
    ToolReading { slot, item: item.to_string(), remaining, max }
}

// max 150 keeps the percent above 10 until 15 uses are left, so only the uses part triggers at 20.
#[test]
fn crossing_alerts_once() {
    let mut t = AlertTracker::new();
    let c = config();
    let got: Vec<bool> = [30, 21, 20, 19, 18].iter().map(|r| t.observe(Some(&tool(0, "pick", *r, 150)), &c)).collect();
    assert_eq!(got, vec![false, false, true, false, false]);
}

#[test]
fn swap_to_low_tool_alerts_once() {
    let mut t = AlertTracker::new();
    let c = config();
    assert!(!t.observe(Some(&tool(0, "a", 1000, 1561)), &c));
    let low = tool(2, "b", 5, 59);
    assert!(t.observe(Some(&low), &c));
    assert!(!t.observe(Some(&low), &c));
}

// in game, a tone on each scroll back to the same low tool was too many (2026-11-08): one tone per tool per drop
#[test]
fn swap_back_to_a_warned_tool_is_silent() {
    let mut t = AlertTracker::new();
    let c = config();
    let a = tool(0, "a", 5, 59);
    let b = tool(1, "b", 1000, 1561);
    assert!(t.observe(Some(&a), &c));
    assert!(!t.observe(Some(&b), &c));
    assert!(!t.observe(Some(&a), &c));
}

#[test]
fn a_second_low_tool_alerts_once_and_the_first_stays_silent() {
    let mut t = AlertTracker::new();
    let c = config();
    let a = tool(0, "a", 5, 59);
    let b = tool(3, "b", 4, 59);
    assert!(t.observe(Some(&a), &c));
    assert!(t.observe(Some(&b), &c));
    assert!(!t.observe(Some(&a), &c));
    assert!(!t.observe(Some(&b), &c));
}

#[test]
fn another_item_in_the_slot_rearms_that_slot() {
    let mut t = AlertTracker::new();
    let c = config();
    let a = tool(0, "a", 5, 59);
    let b = tool(0, "b", 4, 59);
    assert!(t.observe(Some(&a), &c));
    assert!(t.observe(Some(&b), &c));
    assert!(t.observe(Some(&a), &c));
}

#[test]
fn reset_rearms_every_tool() {
    let mut t = AlertTracker::new();
    let c = config();
    let a = tool(0, "a", 5, 59);
    assert!(t.observe(Some(&a), &c));
    t.reset();
    assert!(t.observe(Some(&a), &c));
}

#[test]
fn repair_rearms() {
    let mut t = AlertTracker::new();
    let c = config();
    assert!(t.observe(Some(&tool(0, "a", 15, 1561)), &c));
    assert!(!t.observe(Some(&tool(0, "a", 900, 1561)), &c));
    assert!(t.observe(Some(&tool(0, "a", 15, 1561)), &c));
}

#[test]
fn empty_hand_keeps_the_warning() {
    let mut t = AlertTracker::new();
    let c = config();
    let a = tool(0, "a", 5, 59);
    assert!(t.observe(Some(&a), &c));
    assert!(!t.observe(None, &c));
    assert!(!t.observe(Some(&a), &c));
}

#[test]
fn disabled_never_alerts() {
    let mut t = AlertTracker::new();
    let mut c = config();
    c.enabled = false;
    let a = tool(0, "a", 5, 59);
    assert!(!t.observe(Some(&a), &c));
    assert!(!t.observe(Some(&a), &c));
    assert!(!t.observe(None, &c));
}

#[test]
fn lowering_limit_under_current_tool_is_silent() {
    let mut t = AlertTracker::new();
    let mut c = config();
    let a = tool(0, "a", 15, 1561);
    assert!(t.observe(Some(&a), &c));
    c.limit = Limit { percent: 0, uses: 5 };
    assert!(!t.observe(Some(&a), &c));
    assert!(!t.observe(Some(&a), &c));
}

#[test]
fn raising_limit_over_held_low_tool_alerts_once() {
    let mut t = AlertTracker::new();
    let mut c = config();
    let a = tool(0, "a", 30, 150);
    assert!(!t.observe(Some(&a), &c));
    c.limit = Limit { percent: 0, uses: 50 };
    assert!(t.observe(Some(&a), &c));
    assert!(!t.observe(Some(&a), &c));
}
