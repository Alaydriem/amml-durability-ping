use super::ToolReading;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limit {
    /// 0 turns off the percent part.
    pub percent: u8,
    /// 0 turns off the uses part.
    pub uses: u32,
}

impl Limit {
    pub fn is_off(&self) -> bool {
        self.percent == 0 && self.uses == 0
    }

    pub fn is_below(&self, reading: &ToolReading) -> bool {
        let by_percent = self.percent > 0 && reading.percent_left() <= u32::from(self.percent);
        let by_uses = self.uses > 0 && i64::from(reading.remaining) <= i64::from(self.uses);
        by_percent || by_uses
    }
}
