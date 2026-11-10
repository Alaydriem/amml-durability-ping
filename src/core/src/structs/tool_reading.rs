#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolReading {
    pub slot: i32,
    pub item: String,
    pub remaining: i32,
    pub max: i32,
}

impl ToolReading {
    /// Returns `None` when the item has no durability (`max <= 0`).
    pub fn new(slot: i32, item: String, damage: i32, max: i32) -> Option<Self> {
        if max <= 0 {
            return None;
        }
        Some(Self { slot, item, remaining: (max - damage).clamp(0, max), max })
    }

    /// Percent of durability left, rounded down.
    pub fn percent_left(&self) -> u32 {
        (i64::from(self.remaining) * 100 / i64::from(self.max)) as u32
    }
}
