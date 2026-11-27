use durability_ping_core::structs::ToolReading;

// what the mod reads from the game in one listener call
pub trait GameView {
    // whether the client has a local player (a world is joined)
    fn has_local_player(&self) -> bool;

    // the selected hotbar slot and whether the main hand is empty; None without a local player or when the loader
    // refuses the read
    fn held_slot(&self) -> Option<(u8, bool)>;

    // the tool in the main hand; None for an empty hand, an item without durability, or a refused read
    fn held_tool(&self) -> Option<ToolReading>;

    // whether a menu screen shows: the loading screen, the inventory, the pause menu. false when the read is refused
    fn showing_menu(&self) -> bool;
}
