use durability_ping_core::config::PingConfig;
use durability_ping_core::structs::Limit;

#[test]
fn defaults_match_spec() {
    let config = PingConfig::default();
    assert!(config.alert.enabled);
    assert_eq!(config.alert.limit, Limit { percent: 10, uses: 20 });
    assert_eq!(config.alert.volume, 60);
    assert!(!config.block.enabled);
    assert_eq!(config.block.limit, Limit { percent: 2, uses: 3 });
}
