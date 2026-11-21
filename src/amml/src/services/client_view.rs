use amml_api::{Client, Hand};
use durability_ping_core::structs::ToolReading;

use crate::traits::GameView;

// the game as one listener call sees it, through the loader's client of that call
pub struct ClientView<'a, 'f> {
    client: &'a Client<'f>,
}

impl<'a, 'f> ClientView<'a, 'f> {
    pub fn new(client: &'a Client<'f>) -> Self {
        Self { client }
    }
}

impl GameView for ClientView<'_, '_> {
    fn has_local_player(&self) -> bool {
        self.client.local_player().is_some()
    }

    fn held_slot(&self) -> Option<(u8, bool)> {
        let player = self.client.local_player()?;
        let slot = player.inventory()?.selected_slot()?;
        let empty = player.held_item(Hand::Main).is_none_or(|item| item.is_empty());
        Some((slot, empty))
    }

    fn held_tool(&self) -> Option<ToolReading> {
        let player = self.client.local_player()?;
        let slot = player.inventory()?.selected_slot()?;
        let item = player.held_item(Hand::Main)?;
        ToolReading::new(i32::from(slot), item.type_name()?, item.damage()?, item.max_damage()?)
    }

    fn showing_menu(&self) -> bool {
        self.client.showing_menu().unwrap_or(false)
    }
}
