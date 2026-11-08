use durability_ping_core::structs::ToolReading;

// what one tick reads from the game
pub trait GameView {
    // the address of the local player; 0 when there is none (no world, or the entry is off)
    fn local_player(&self) -> usize;

    // the tool in the selected slot; None for an empty hand, an item without durability, or without the entries
    fn held_tool(&self) -> Option<ToolReading>;

    // whether a menu screen shows: the loading screen, the inventory, the pause menu. false without the entry
    fn showing_menu(&self) -> bool;
}
