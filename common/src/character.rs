use std::collections::VecDeque;

use bevy::prelude::*;
use crossterm::style::Stylize;
use crate::{physics::*, dialogue::{Dialogue, DialogueOption}, inventory::{Equipment, Inventory}};

#[derive(Component)]
pub struct PlayerTag;
pub struct PlayerInput {
    pub input_movement: MovementInput,
}
impl FromWorld for PlayerInput {
    fn from_world(_world: &mut World) -> Self {
        PlayerInput { input_movement: MovementInput::Idle }
    }
}

#[derive(Component, Clone, Copy)]
pub struct Sprite {
    pub character: char,
}
impl Sprite {
    pub fn new(character: char) -> Sprite {
        Sprite {
            character,
        }
    }
}

#[derive(Component)]
pub struct Health {
    value: i32,
    max: i32,
}
impl Health {
    pub fn new(value: i32) -> Health {
        Health {
            value,
            max: value,
        }
    }
    pub fn heal(&mut self, value: i32) {
        self.value += value;
        if self.value > self.max {
            self.value = self.max;
        }
    }
}

#[derive(Bundle)]
pub struct CharacterBundle {
    pub input_data: MovementInput,
    pub sprite: Sprite,
    pub position: Position,
    pub velocity: Velocity,
    pub interact: Interact,
    pub health: Health,
    pub data: CharacterType,
    pub action_history: ActionHistory,
    pub inventory: Inventory,
    pub equipment: Equipment,
}

#[derive(Component)]
pub struct ActionHistory {
    movement_history: VecDeque<MovementInput>,
    size: usize,
}
impl ActionHistory {
    pub fn new(size: usize) -> Self {
        ActionHistory {
            movement_history: VecDeque::with_capacity(size),
            size,
        }
    }
    pub fn add(&mut self, movement_input: MovementInput) {
        if self.movement_history.len() == self.size {
            self.movement_history.pop_front();
        }
        self.movement_history.push_back(movement_input);
    }
}
impl ToString for ActionHistory {
    fn to_string(&self) -> String {
        let mut text = String::new();
        for movement in self.movement_history.iter().rev() {
            text.push_str(movement.to_str());
            text.push('\n');
        }
        text
    }
}
#[derive(Component, Debug)]
pub enum CharacterType {
    Player,
    Lerain,
    Rumdare,
    Werewolf,
}
impl PartialEq for CharacterType {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Self::Player => match other {
                Self::Player => true,
                _ => false,
            },
            Self::Lerain => match other {
                Self::Lerain => true,
                _ => false,
            },
            Self::Rumdare => match other {
                Self::Rumdare => true,
                _ => false,
            },
            Self::Werewolf => match other {
                Self::Werewolf => true,
                _ => false,
            },
        }
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl From<&CharacterType> for Interact {
    fn from(value: &CharacterType) -> Self {
        // Get defaults
        match value {
            CharacterType::Player => Interact::Player,
            CharacterType::Lerain => Interact::Lerain,
            CharacterType::Rumdare => Interact::Rumdare,
            CharacterType::Werewolf => Interact::Werewolf,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub enum MovementInput {
    Idle,
    North,
    East,
    South,
    West,
}
impl MovementInput {
    fn to_str(&self) -> &'static str {
        match self {
            MovementInput::Idle => "Idle",
            MovementInput::North => "North",
            MovementInput::East => "East",
            MovementInput::South => "South",
            MovementInput::West => "West",
        }
    }
}

#[derive(Component)]
pub enum Interact {
    Player,
    Lerain,
    Rumdare,
    Werewolf,
}
impl Interact {
    fn interact(&mut self, dialogue: &mut Dialogue, character_data: &CharacterType) {
        match self {
            Interact::Player => {
                dialogue.activate("Bruh".to_string(), vec![("Option 1".to_string(), DialogueOption::Leave)]);
            },
            Interact::Lerain | Interact::Rumdare | Interact::Werewolf => {},
        }
    }
}

/// True if collided, otherwise set tile to occupied and return false.
fn check_collision(
    map: &mut Map,
    entity: Entity,
    current_position: &Position,
    new_position: &Position,
    sprite: Option<&Sprite>,
    dialogue: &mut Dialogue,
    interact: &mut Interact,
    interact_query: &Query<&CharacterType>,
) -> bool {
    let mut place_character_at_new_position = || {
        if let Some(tile) = new_position.get_mut_from_map(map) {
            if tile.is_occupied() {
                if let Tile::Ground { occupier: occupier_option, .. } = tile {
                    if let Some(occupier) = occupier_option {
                        if let Ok(character_data) = interact_query.get(occupier.entity) {
                            interact.interact(dialogue, character_data);
                        }
                    }
                }
            } else {
                if let Tile::Ground { occupier: value, .. } = tile {
                    if let Some(s) = sprite {
                        *value = Some(Occupier::new(entity, *s));
                        return true;
                    }
                }
            }
        }
        false
    };
    if place_character_at_new_position() {
        if let Some(Tile::Ground { occupier: ref mut old_value, .. }) = current_position.get_mut_from_map(map) {
            *old_value = None;
        }
        false
    } else {
        true
    }
}

fn move_update(
    mut map: &mut Map,
    entity: Entity,
    input: &MovementInput,
    position: &mut Position,
    sprite: Option<&Sprite>,
    dialogue: &mut Dialogue,
    interact: &mut Interact,
    interact_query: &Query<&CharacterType>,
    action_history: &mut ActionHistory,
) {
    let mut move_position = |movement: Position, movement_input: MovementInput| {
        if check_collision(&mut map, entity, &position, &(*position + movement), sprite, dialogue, interact, interact_query) { return; }
        *position += movement;
        action_history.add(movement_input);
    };
    match input {
        MovementInput::North => move_position(Position::new(0, 1), MovementInput::North),
        MovementInput::East => move_position(Position::new(1, 0), MovementInput::East),
        MovementInput::South => move_position(Position::new(0, -1), MovementInput::South),
        MovementInput::West => move_position(Position::new(-1, 0), MovementInput::West),
        _ => {},
    }
}
pub fn player_movement_update(
    mut map: ResMut<Map>,
    mut dialogue: ResMut<Dialogue>,
    mut player_query: Query<(Entity, &MovementInput, &mut Position, Option<&Sprite>, &mut Interact, &mut ActionHistory), With<PlayerTag>>,
    interact_query: Query<&CharacterType>,
) {
    for (entity, input, mut position, sprite, mut interact, mut action_history) in player_query.iter_mut() {
        move_update(&mut map, entity, input, &mut position, sprite, &mut dialogue, &mut interact, &interact_query, &mut action_history);
    }
}
pub fn npc_movement_update(
    mut map: ResMut<Map>,
    mut dialogue: ResMut<Dialogue>,
    mut npc_query: Query<(Entity, &MovementInput, &mut Position, Option<&Sprite>, &mut Interact, &mut ActionHistory), Without<PlayerTag>>,
    interact_query: Query<&CharacterType>,
) {
    for (entity, input, mut position, sprite, mut interact, mut action_history) in npc_query.iter_mut() {
        move_update(&mut map, entity, input, &mut position, sprite, &mut dialogue, &mut interact, &interact_query, &mut action_history);
    }
}
pub fn player_movement_input_update(player_input: Res<PlayerInput>, mut query: Query<(&mut MovementInput, With<PlayerTag>)>) {
    for (mut movement_input, _) in query.iter_mut() {
        *movement_input = player_input.input_movement;
    }
}
