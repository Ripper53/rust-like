use bevy::prelude::SystemLabel;

pub mod constants;
pub mod physics;
pub mod character;
pub mod map_brain;
pub mod map_setup;
pub mod dialogue;
pub mod inventory;
pub mod util;
pub mod behaviors;
pub mod loot_menu;

pub enum ActionInput {
    // Take no action.
    None,
    /// Select item from inventory.
    SelectFromInventory(usize),
    UseEquippedItem,
}

#[derive(SystemLabel, Debug, Clone, Hash, Eq, PartialEq)]
pub enum Scene {
    Map,
    Inventory,
    Settings,
}

#[derive(Default, Clone, Copy)]
pub enum PlayerState {
    #[default]
    None,
    Dialogue,
    Looting,
}
