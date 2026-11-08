use crate::structs::Limit;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlertConfig {
    pub enabled: bool,
    pub limit: Limit,
    pub volume: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockConfig {
    pub enabled: bool,
    pub limit: Limit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PingConfig {
    pub alert: AlertConfig,
    pub block: BlockConfig,
}

impl Default for PingConfig {
    fn default() -> Self {
        Self {
            alert: AlertConfig { enabled: true, limit: Limit { percent: 10, uses: 20 }, volume: 60 },
            block: BlockConfig { enabled: false, limit: Limit { percent: 2, uses: 3 } },
        }
    }
}
