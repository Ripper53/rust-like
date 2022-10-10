use bevy::prelude::{App, Entity};

use crate::inventory::Inventory;

#[derive(Default)]
pub struct LootMenu {
    pub inventory: Option<Entity>,
}

impl LootMenu {
    pub fn select(&self, app: &mut App, from_inventory: (Entity, usize), to_inventory: Entity) {
        if let Some(mut inventory) = app.world.entity_mut(from_inventory.0).get_mut::<Inventory>() {
            let item = inventory.remove_item(from_inventory.1);
            if let Some(mut inventory) = app.world.entity_mut(to_inventory).get_mut::<Inventory>() {
                inventory.add_item(item);
            }
        }
    }
}
