use bevy::prelude::SystemLabel;

pub mod physics;
pub mod character;
pub mod map_brain;
pub mod map_setup;
pub mod dialogue;
pub mod inventory;
pub mod util;

pub enum ActionInput {
    // Take no action.
    None,
    /// Select item from inventory.
    SelectFromInventory(usize),
}

#[derive(SystemLabel, Debug, Clone, Hash, Eq, PartialEq)]
pub enum Scene {
    Map,
    Inventory,
    Settings,
}
