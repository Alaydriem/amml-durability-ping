use durability_ping_core::services::CueLimiter;
use std::time::Duration;

#[test]
fn allows_one_per_gap() {
    let mut l = CueLimiter::new();
    assert!(l.allow(Duration::from_secs(0)));
    assert!(!l.allow(Duration::from_millis(500)));
    assert!(l.allow(Duration::from_secs(1)));
    assert!(!l.allow(Duration::from_millis(1200)));
}
