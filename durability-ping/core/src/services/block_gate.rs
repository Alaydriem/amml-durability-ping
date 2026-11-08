use crate::config::BlockConfig;
use crate::structs::ToolReading;

pub struct BlockGate;

impl BlockGate {
    /// True when the current action must be cancelled.
    pub fn blocks(reading: Option<&ToolReading>, config: &BlockConfig, override_held: bool) -> bool {
        if !config.enabled || override_held {
            return false;
        }
        reading.is_some_and(|r| config.limit.is_below(r))
    }
}
