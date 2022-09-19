use std::collections::VecDeque;

use bevy::prelude::*;
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
    pub interact: Interact,
    pub health: Health,
    pub data: CharacterType,
    pub action_history: ActionHistory,
    pub inventory: Inventory,
    pub equipment: Equipment,
    pub collision_type: CollisionType,
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
    pub fn get_latest(&self) -> Option<&MovementInput> {
        self.movement_history.back()
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
            Self::Player => matches!(other, Self::Player),
            Self::Lerain => matches!(other, Self::Lerain),
            Self::Rumdare => matches!(other, Self::Rumdare),
            Self::Werewolf => matches!(other, Self::Werewolf),
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

#[derive(Component, Debug, Clone, Copy)]
pub enum MovementInput {
    Idle,
    North,
    East,
    South,
    West,
}
pub struct ToPositionError;
impl std::fmt::Display for ToPositionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "cannot convert to position")
    }
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
    pub fn to_position(&self) -> Result<Position, ToPositionError> {
        match self {
            MovementInput::Idle => Err(ToPositionError),
            MovementInput::North => Ok(Position::new(0, 1)),
            MovementInput::East => Ok(Position::new(1, 0)),
            MovementInput::South => Ok(Position::new(0, -1)),
            MovementInput::West => Ok(Position::new(-1, 0)),
        }
    }
}

#[derive(Component)]
pub enum Interact {
    Player,
    Lerain,
    Rumdare,
    Werewolf,
    Projectile {
        recent_spawn: bool,
        damage: i32,
    },
}
impl Interact {
    fn interact(&mut self, dialogue: &mut Dialogue, character_type: &CharacterType) {
        match self {
            Interact::Player => {
                dialogue.activate("Bruh".to_string(), vec![("Option 1".to_string(), DialogueOption::Leave)]);
            },
            Interact::Lerain | Interact::Rumdare | Interact::Werewolf => {},
            Interact::Projectile { damage, .. } => {
                // Collision!
            },
        }
    }
}

pub enum CollisionCheckResult {
    NoCollision,
    IsOccupied(Position),
    TileDoesNotExist,
    TileIsNotGround,
}
fn check_collision_and_move(
    map: &mut Map,
    entity: Entity,
    collision_type: &CollisionType,
    current_position: &mut Position,
    new_position: &Position,
    sprite: Option<&Sprite>,
) -> CollisionCheckResult {
    let old_position = current_position.clone();
    let mut place_character_at_new_position = || {
        if let Some(tile) = new_position.get_mut_from_map(map) {
            if tile.is_occupied(collision_type) {
                CollisionCheckResult::IsOccupied(new_position.clone())
            } else if let Tile::Ground { occupier, .. } | Tile::Obstacle { occupier } = tile {
                *occupier = if let Some(s) = sprite {
                    Some(Occupier::new(entity, *s))
                } else {
                    Some(Occupier::new(entity, Sprite::new('?')))
                };
                *current_position = new_position.clone();
                CollisionCheckResult::NoCollision
            } else {
                CollisionCheckResult::TileIsNotGround
            }
        } else {
            CollisionCheckResult::TileDoesNotExist
        }
    };
    let result = place_character_at_new_position();
    if matches!(result, CollisionCheckResult::NoCollision) {
        let map = map;
        if let Some(
            Tile::Ground { occupier: ref mut old_value, .. } |
            Tile::Obstacle { occupier: ref mut old_value }
        ) = old_position.get_mut_from_map(map) {
            *old_value = None;
        }
        CollisionCheckResult::NoCollision
    } else {
        result
    }
}
/// True if collided, otherwise set tile to occupied and return false.
pub fn check_collision_and_move_or_interact(
    map: &mut Map,
    entity: Entity,
    collision_type: &CollisionType,
    current_position: &mut Position,
    new_position: &Position,
    sprite: Option<&Sprite>,
    dialogue: &mut Dialogue,
    interact: &mut Interact,
    interact_query: &Query<&CharacterType>,
) -> bool {
    let result = check_collision_and_move(map, entity, collision_type, current_position, new_position, sprite);
    if let CollisionCheckResult::IsOccupied(position) = result {
        if let Some(
            Tile::Ground { occupier, .. } |
            Tile::Obstacle { occupier }
        ) = map.get_mut(position.x as usize, position.y as usize) {
            if let Some(occupier) = occupier {
                if let Ok(character_data) = interact_query.get(occupier.entity) {
                    interact.interact(dialogue, character_data);
                }
            }
        }
        false
    } else {
        true
    }
}

fn move_update(
    mut map: &mut Map,
    entity: Entity,
    collision_type: &CollisionType,
    input: &MovementInput,
    position: &mut Position,
    sprite: Option<&Sprite>,
    dialogue: &mut Dialogue,
    interact: &mut Interact,
    interact_query: &Query<&CharacterType>,
    action_history: Option<&mut ActionHistory>,
) {
    if let Ok(movement) = input.to_position() {
        if check_collision_and_move_or_interact(
            &mut map,
            entity,
            collision_type,
            position,
            &(*position + movement),
            sprite,
            dialogue,
            interact,
            interact_query,
        ) {
            if let Some(action_history) = action_history {
                action_history.add(*input);
            }
        }
    }
}
pub fn player_movement_update(
    mut map: ResMut<Map>,
    mut dialogue: ResMut<Dialogue>,
    mut player_query: Query<(Entity, &MovementInput, &mut Position, Option<&Sprite>, &CollisionType, &mut Interact, Option<&mut ActionHistory>), With<PlayerTag>>,
    interact_query: Query<&CharacterType>,
) {
    for (entity, input, mut position, sprite, collision_type, mut interact, mut action_history) in player_query.iter_mut() {
        move_update(&mut map, entity, collision_type, input, &mut position, sprite, &mut dialogue, &mut interact, &interact_query, action_history.as_deref_mut());
    }
}
pub fn npc_movement_update(
    mut map: ResMut<Map>,
    mut dialogue: ResMut<Dialogue>,
    mut npc_query: Query<(Entity, &mut MovementInput, &mut Position, Option<&Sprite>, &CollisionType, &mut Interact, Option<&mut ActionHistory>, Option<&Velocity>), Without<PlayerTag>>,
    interact_query: Query<&CharacterType>,
) {
    for (entity, mut movement_input, mut position, sprite, collision_type, mut interact, mut action_history, velocity) in npc_query.iter_mut() {
        let times = if let Some(velocity) = velocity {
            if let Interact::Projectile { recent_spawn, .. } = interact.as_mut() {
                if *recent_spawn {
                    *recent_spawn = false;
                    1
                } else {
                    *movement_input = velocity.movement;
                    velocity.speed
                }
            } else {
                *movement_input = velocity.movement;
                velocity.speed
            }
        } else {
            1
        };
        for _ in 0..times {
            move_update(&mut map, entity, collision_type, &movement_input, &mut position, sprite, &mut dialogue, &mut interact, &interact_query, action_history.as_deref_mut());
        }
    }
}
pub fn player_movement_input_update(player_input: Res<PlayerInput>, mut query: Query<&mut MovementInput, With<PlayerTag>>) {
    for mut movement_input in query.iter_mut() {
        *movement_input = player_input.input_movement;
    }
}
