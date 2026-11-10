use durability_ping_core::config::{AlertConfig, BlockConfig, PingConfig};
use durability_ping_core::structs::Limit;
use serde::Deserialize;

// the part of config.hcl the mod reads; a missing value keeps its default. The `keys.override` chord belongs to
// the manager and is not read here. Numbers are i64 so a value below the range can be seen and clamped
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct RawSettings {
    alert: RawAlert,
    block: RawBlock,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawAlert {
    enabled: Option<bool>,
    percent: Option<i64>,
    uses: Option<i64>,
    volume: Option<i64>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
struct RawBlock {
    enabled: Option<bool>,
    percent: Option<i64>,
    uses: Option<i64>,
}

// turns the checked config.hcl into a PingConfig; a value outside its field range is clamped with one warning
pub struct SettingsReader;

impl SettingsReader {
    const PERCENT_MAX: i64 = 100;
    const USES_MAX: i64 = 2000;
    const VOLUME_MAX: i64 = 100;

    // text that cannot be read gives the defaults, with one warning
    pub fn read(text: &str) -> PingConfig {
        match hcl::from_str::<RawSettings>(text) {
            Ok(raw) => Self::convert(&raw),
            Err(error) => {
                curia::warn!("config could not be read; using the defaults", { error: error.to_string() });
                PingConfig::default()
            }
        }
    }

    pub fn convert(raw: &RawSettings) -> PingConfig {
        let defaults = PingConfig::default();
        PingConfig {
            alert: AlertConfig {
                enabled: raw.alert.enabled.unwrap_or(defaults.alert.enabled),
                limit: Limit {
                    percent: Self::number("alert.percent", raw.alert.percent, Self::PERCENT_MAX)
                        .map_or(defaults.alert.limit.percent, |value| value as u8),
                    uses: Self::number("alert.uses", raw.alert.uses, Self::USES_MAX)
                        .map_or(defaults.alert.limit.uses, |value| value as u32),
                },
                volume: Self::number("alert.volume", raw.alert.volume, Self::VOLUME_MAX)
                    .map_or(defaults.alert.volume, |value| value as u8),
            },
            block: BlockConfig {
                enabled: raw.block.enabled.unwrap_or(defaults.block.enabled),
                limit: Limit {
                    percent: Self::number("block.percent", raw.block.percent, Self::PERCENT_MAX)
                        .map_or(defaults.block.limit.percent, |value| value as u8),
                    uses: Self::number("block.uses", raw.block.uses, Self::USES_MAX)
                        .map_or(defaults.block.limit.uses, |value| value as u32),
                },
            },
        }
    }

    // None when the setting is missing; otherwise the value clamped to 0..=max, so it always fits its type
    fn number(name: &str, value: Option<i64>, max: i64) -> Option<i64> {
        value.map(|value| {
            let clamped = value.clamp(0, max);
            if clamped != value {
                curia::warn!("setting is outside its range; clamped", { setting: name, value: value, clamped: clamped });
            }
            clamped
        })
    }
}
