use bevy::prelude::*;
use crate::{
    character::{PlayerTag, Health, Sprite, ActionHistory},
    physics::{Map, Position, Velocity},
    ActionInput,
    util::spawn_projectile,
};

#[derive(Clone)]
pub enum Item {
    Food {
        info: ItemBasicInfo,
        heal: i32,
    },
    Gun {
        info: ItemBasicInfo,
        damage: i32,
        speed: i32,
    },
}

#[derive(Clone)]
pub struct ItemBasicInfo {
    name: String,
    description: String,
}

impl Item {
    pub fn get_name(&self) -> String {
        match self {
            Item::Food { info, .. } |
            Item::Gun { info, .. } => info.name.clone(),
        }
    }
    pub fn get_description(&self) -> String {
        match self {
            Item::Food { info, .. } |
            Item::Gun { info, .. } => info.description.clone(),
        }
    }
    fn new_food(name: String, heal: i32) -> Self {
        Item::Food {
            info: ItemBasicInfo {
                name,
                description: format!("Heal for {heal}."),
            },
            heal,
        }
    }
    pub fn new_apple() -> Self {
        Self::new_food("Apple".to_string(), 1)
    }
    pub fn new_banana() -> Self {
        Self::new_food("Banana".to_string(), 2)
    }

    fn new_gun(name: String, description: String, damage: i32, speed: i32) -> Self {
        Item::Gun {
            info: ItemBasicInfo {
                name, description,
            },
            damage,
            speed,
        }
    }
    pub fn new_pistol() -> Self {
        Self::new_gun("Pistol".to_string(), "Gun".to_string(), 1, 2)
    }
}

impl PartialEq for &Box<Item> {
    fn eq(&self, other: &Self) -> bool {
        let left: *const Item = self.as_ref();
        let right: *const Item = other.as_ref();
        left == right
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

#[derive(Component, Default)]
pub struct Inventory {
    items: Vec<Box<Item>>,
}

impl Inventory {
    pub fn new(items: Vec<Box<Item>>) -> Self {
        Inventory { items }
    }
    pub fn get_index(&self, item: &Box<Item>) -> Option<usize> {
        for i in 0..self.items.len() {
            if &self.items[i] == item {
                return Some(i);
            }
        }
        None
    }
    pub fn add_item(&mut self, item: Box<Item>) {
        // TODO? PERHAPS CHECK FOR DUPLICATE ITEMS!
        // BECAUSE WE ARE NOT USING A SET!
        self.items.push(item);
    }
    pub fn remove_item(&mut self, index: usize) -> Box<Item> {
        self.items.remove(index)
    }
    pub fn items(&self) -> &Vec<Box<Item>> {
        &self.items
    }
}

#[derive(Component, Default)]
pub struct Equipment {
    pub equipped: Option<Box<Item>>,
}

pub fn inventory_update(
    mut commands: Commands,
    mut map: ResMut<Map>,
    mut action_input: ResMut<ActionInput>,
    mut query: Query<(&Position, &mut Inventory, &mut Health, &mut Equipment, &ActionHistory), With<PlayerTag>>,
) {
    for (position, mut inventory, mut health, mut equipment, action_history) in query.iter_mut() {
        match *action_input {
            ActionInput::None => { /* Take no action! */},
            ActionInput::SelectFromInventory(index) => {
                if let Some(item) = inventory.items.get_mut(index) {
                    match item.as_mut() {
                        Item::Food { heal, .. } => {
                            health.heal(*heal);
                            inventory.items.remove(index);
                        },
                        Item::Gun { .. } => equipment.equipped = Some(inventory.items.remove(index)),
                    }
                }
            },
            ActionInput::UseEquippedItem => {
                if let Some(equipped) = &equipment.equipped {
                    match equipped.as_ref() {
                        Item::Gun { damage, speed, .. } => {
                            // Shoot projectile!
                            if let Some(latest_movement_input) = action_history.get_latest() {
                                if let Ok(movement) = latest_movement_input.to_position() {
                                    spawn_projectile(
                                        &mut commands,
                                        &mut map,
                                        Sprite::Projectile,
                                        *position + movement,
                                        Velocity::new(latest_movement_input.clone(), *speed),
                                        *damage,
                                    );
                                }
                            }
                        },
                        _ => {},
                    }
                }
            },
        }
    }
    *action_input = ActionInput::None;
}
