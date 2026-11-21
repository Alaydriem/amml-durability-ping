use std::cell::Cell;

use alaydriem_durability_ping::traits::GameView;
use durability_ping_core::structs::ToolReading;

// a game for one runtime call: a player or none, the held tool, the menu flag; counts the held-tool reads
pub struct FakeView {
    pub player: bool,
    pub slot: u8,
    pub tool: Option<ToolReading>,
    pub menu: bool,
    pub reads: Cell<usize>,
}

impl FakeView {
    // in a world with an iron pickaxe in slot 0 with `uses` uses left of 250
    pub fn holding(uses: i32) -> Self {
        Self {
            player: true,
            slot: 0,
            tool: ToolReading::new(0, "minecraft:iron_pickaxe".to_string(), 250 - uses, 250),
            menu: false,
            reads: Cell::new(0),
        }
    }

    // on the title screen: no player
    pub fn no_player() -> Self {
        Self { player: false, slot: 0, tool: None, menu: false, reads: Cell::new(0) }
    }
}

impl GameView for FakeView {
    fn has_local_player(&self) -> bool {
        self.player
    }

    fn held_slot(&self) -> Option<(u8, bool)> {
        self.player.then_some((self.slot, self.tool.is_none()))
    }

    fn held_tool(&self) -> Option<ToolReading> {
        self.reads.set(self.reads.get() + 1);
        self.tool.clone()
    }

    fn showing_menu(&self) -> bool {
        self.menu
    }
}
