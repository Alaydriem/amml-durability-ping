use std::cell::{Cell, RefCell};

use amml_durability_ping::traits::GameView;
use durability_ping_core::structs::ToolReading;

// a stand-in for the game: the test sets whether a player exists and what the player holds
pub struct FakeGame {
    player: Cell<usize>,
    tool: RefCell<Option<ToolReading>>,
    reads: Cell<usize>,
    menu: Cell<bool>,
}

impl FakeGame {
    pub const MAX: i32 = 1561;
    pub const PLAYER: usize = 0x1234;

    pub fn playing() -> Self {
        Self { player: Cell::new(Self::PLAYER), tool: RefCell::new(None), reads: Cell::new(0), menu: Cell::new(false) }
    }

    pub fn set_player(&self, player: usize) {
        self.player.set(player);
    }

    // whether a menu screen shows (the loading screen, the inventory, the pause menu)
    pub fn set_menu(&self, menu: bool) {
        self.menu.set(menu);
    }

    // how many times the game was read (local_player or held_tool)
    pub fn reads(&self) -> usize {
        self.reads.get()
    }

    // a pickaxe in slot 0 with `uses` uses left
    pub fn hold_uses(&self, uses: i32) {
        *self.tool.borrow_mut() =
            ToolReading::new(0, "minecraft:diamond_pickaxe".to_string(), Self::MAX - uses, Self::MAX);
    }
}

impl GameView for FakeGame {
    fn local_player(&self) -> usize {
        self.reads.set(self.reads.get() + 1);
        self.player.get()
    }

    fn held_tool(&self) -> Option<ToolReading> {
        self.reads.set(self.reads.get() + 1);
        self.tool.borrow().clone()
    }

    fn showing_menu(&self) -> bool {
        self.menu.get()
    }
}
