use durability_ping_core::structs::ToolReading;

#[test]
fn zero_max_is_none() {
    assert_eq!(ToolReading::new(0, "x".to_string(), 0, 0), None);
    assert_eq!(ToolReading::new(0, "x".to_string(), 0, -3), None);
}

#[test]
fn damage_over_max_clamps_to_zero() {
    let r = ToolReading::new(0, "x".to_string(), 70, 59).unwrap();
    assert_eq!(r.remaining, 0);
}

#[test]
fn negative_damage_clamps_to_max() {
    let r = ToolReading::new(0, "x".to_string(), -5, 59).unwrap();
    assert_eq!(r.remaining, 59);
}

#[test]
fn percent_left_rounds_down() {
    let r = ToolReading::new(0, "x".to_string(), 1561 - 156, 1561).unwrap();
    assert_eq!(r.remaining, 156);
    assert_eq!(r.percent_left(), 9);
}
