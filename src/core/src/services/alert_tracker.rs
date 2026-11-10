use crate::config::AlertConfig;
use crate::structs::ToolReading;

/// One warning per tool for each drop below the limit. A tool is its hotbar slot and item name. Selecting a
/// warned tool again stays silent; a reading above the limit re-arms it (a repair, or a lower limit), and another
/// item in its slot drops it. An empty hand changes nothing; `reset` (no player, such as after leaving a world)
/// re-arms every tool.
#[derive(Default)]
pub struct AlertTracker {
    warned: Vec<(i32, String)>,
}

impl AlertTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Forgets every warned tool.
    pub fn reset(&mut self) {
        self.warned.clear();
    }

    /// Returns true when the warning cue must play now.
    pub fn observe(&mut self, reading: Option<&ToolReading>, config: &AlertConfig) -> bool {
        let Some(reading) = reading else { return false };
        // the slot now holds this item, so a warning for another item in it is stale
        self.warned.retain(|(slot, item)| *slot != reading.slot || *item == reading.item);
        let known = self.warned.iter().any(|(slot, item)| *slot == reading.slot && *item == reading.item);
        let below = config.enabled && config.limit.is_below(reading);
        if !below {
            if known {
                self.warned.retain(|(slot, _)| *slot != reading.slot);
            }
            return false;
        }
        if known {
            return false;
        }
        self.warned.push((reading.slot, reading.item.clone()));
        true
    }
}
