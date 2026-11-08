use durability_ping_core::structs::{Limit, ToolReading};

fn reading(max: i32, remaining: i32) -> ToolReading {
    ToolReading { slot: 0, item: "x".to_string(), remaining, max }
}

#[test]
fn below_cases() {
    let limit = Limit { percent: 10, uses: 20 };
    assert!(limit.is_below(&reading(1561, 156)));
    assert!(!limit.is_below(&reading(1561, 200)));
    assert!(limit.is_below(&reading(59, 20)));
    assert!(!limit.is_below(&reading(59, 21)));
    assert!(limit.is_below(&reading(59, 5)));
}

#[test]
fn is_off_only_when_both_zero() {
    assert!(Limit { percent: 0, uses: 0 }.is_off());
    assert!(!Limit { percent: 0, uses: 1 }.is_off());
    assert!(!Limit { percent: 1, uses: 0 }.is_off());
}

#[test]
fn zero_percent_ignores_percent() {
    let limit = Limit { percent: 0, uses: 20 };
    assert!(!limit.is_below(&reading(1000, 21)));
    assert!(limit.is_below(&reading(1000, 20)));
    assert!(limit.is_below(&reading(59, 20)));
}

#[test]
fn zero_uses_ignores_uses() {
    let limit = Limit { percent: 10, uses: 0 };
    // 5 of 1000 is 0 percent, so the percent part triggers.
    assert!(limit.is_below(&reading(1000, 5)));
    // 20 of 59 is 33 percent. The uses part is off, so it is not below.
    assert!(!limit.is_below(&reading(59, 20)));
    assert!(!limit.is_below(&reading(59, 5 + 15)));
}

#[test]
fn off_limit_is_never_below() {
    let limit = Limit { percent: 0, uses: 0 };
    assert!(!limit.is_below(&reading(59, 0)));
}
