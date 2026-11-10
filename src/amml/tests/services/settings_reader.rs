use alaydriem_durability_ping::services::SettingsReader;
use durability_ping_core::config::{AlertConfig, BlockConfig, PingConfig};
use durability_ping_core::structs::Limit;

#[test]
fn empty_text_gives_the_defaults() {
    assert_eq!(SettingsReader::read(""), PingConfig::default());
}

#[test]
fn a_full_config_round_trips() {
    let text = "alert {
  enabled = false
  percent = 25
  uses = 100
  volume = 35
}
block {
  enabled = true
  percent = 5
  uses = 7
}
";

    let config = SettingsReader::read(text);

    assert_eq!(
        config,
        PingConfig {
            alert: AlertConfig { enabled: false, limit: Limit { percent: 25, uses: 100 }, volume: 35 },
            block: BlockConfig { enabled: true, limit: Limit { percent: 5, uses: 7 } },
        }
    );
}

#[test]
fn percent_above_the_range_clamps_to_100() {
    let config = SettingsReader::read("alert {\n  percent = 150\n}\n");

    assert_eq!(config.alert.limit.percent, 100);
}

#[test]
fn negative_uses_clamp_to_0() {
    let config = SettingsReader::read("alert {\n  uses = -1\n}\n");

    assert_eq!(config.alert.limit.uses, 0);
}

#[test]
fn uses_and_volume_clamp_to_their_ranges() {
    let config = SettingsReader::read("block {\n  uses = 99999\n}\nalert {\n  volume = 500\n}\n");

    assert_eq!(config.block.limit.uses, 2000);
    assert_eq!(config.alert.volume, 100);
}

#[test]
fn unreadable_text_gives_the_defaults() {
    assert_eq!(SettingsReader::read("alert {"), PingConfig::default());
}
