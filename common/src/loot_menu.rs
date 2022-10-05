use bevy::prelude::Entity;

use crate::inventory::Inventory;

pub struct LootMenu {
    pub is_looting: bool,
    /// Inventory entity, and item index.
    pub items: Vec<(Entity, usize)>,
}

impl LootMenu {
    pub fn select(&self, from_inventory: (&mut Inventory, usize), to_inventory: &mut Inventory) {
        let item = from_inventory.0.remove_item(from_inventory.1);
        to_inventory.add_item(item);
    }
}
