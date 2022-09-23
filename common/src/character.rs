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
    pub value: i32,
    pub max: i32,
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
    pub fn damage(&mut self, value: i32) {
        self.value -= value;
        if self.value < 0 {
            self.value = 0;
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
    pub character_type: CharacterType,
    pub character_data: CharacterData,
    pub action_history: ActionHistory,
    pub inventory: Inventory,
    pub equipment: Equipment,
    pub collision: Collision,
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
pub enum WereForm {
    Human,
    Beast,
}
#[derive(Component)]
pub enum CharacterData {
    Human,
    Werewolf {
        form: WereForm,
    },
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
            CharacterType::Player => Interact::new(InteractData::Player),
            CharacterType::Lerain => Interact::new(InteractData::Lerain),
            CharacterType::Rumdare => Interact::new(InteractData::Rumdare),
            CharacterType::Werewolf => Interact::new(InteractData::Werewolf),
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
pub struct Interact {
    pub info: Option<InteractInfo>,
    pub data: InteractData,
}
pub struct InteractInfo {
    pub entity: Entity,
    pub position: Position,
    pub other_entity: Entity,
    pub other_position: Position,
}
pub enum InteractData {
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
    pub fn new(data: InteractData) -> Self {
        Interact {
            info: None,
            data,
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
    collision: &mut Collision,
    current_position: &mut Position,
    new_position: &Position,
    sprite: Option<&Sprite>,
) -> CollisionCheckResult {
    let old_position = current_position.clone();
    let mut place_character_at_new_position = || {
        if let Some(tile) = new_position.get_mut_from_map(map) {
            if tile.is_occupied(collision) {
                CollisionCheckResult::IsOccupied(new_position.clone())
            } else if let Tile::Ground { occupier, .. } | Tile::Obstacle { occupier } = tile {
                *occupier = if let Some(s) = sprite {
                    Some(Occupier::new(entity, *s, collision.collision_type.clone()))
                } else {
                    Some(Occupier::new(entity, Sprite::new('?'), collision.collision_type.clone()))
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
    collision: &mut Collision,
    current_position: &mut Position,
    new_position: &Position,
    sprite: Option<&Sprite>,
    interact: &mut Interact,
) -> bool {
    let result = check_collision_and_move(map, entity, collision, current_position, new_position, sprite);
    if let CollisionCheckResult::IsOccupied(position) = result {
        if let Some(
            Tile::Ground { occupier, .. } |
            Tile::Obstacle { occupier }
        ) = map.get_mut(position.x as usize, position.y as usize) {
            if let Some(occupier) = occupier {
                interact.info = Some(InteractInfo {
                    entity,
                    other_entity: occupier.entity,
                    position: *current_position,
                    other_position: position,
                });
            }
        }
        false
    } else {
        true
    }
}

fn move_update(
    map: &mut Map,
    entity: Entity,
    collision: &mut Collision,
    input: &MovementInput,
    position: &mut Position,
    sprite: Option<&Sprite>,
    interact: &mut Interact,
    action_history: Option<&mut ActionHistory>,
) {
    if let Ok(movement) = input.to_position() {
        if check_collision_and_move_or_interact(
            map,
            entity,
            collision,
            position,
            &(*position + movement),
            sprite,
            interact,
        ) {
            if let Some(action_history) = action_history {
                action_history.add(*input);
            }
        }
    }
}
pub fn player_movement_update(
    mut map: ResMut<Map>,
    mut player_query: Query<(Entity, &MovementInput, &mut Position, Option<&Sprite>, &mut Collision, &mut Interact, Option<&mut ActionHistory>), With<PlayerTag>>,
) {
    for (entity, input, mut position, sprite, mut collision, mut interact, mut action_history) in player_query.iter_mut() {
        move_update(
            &mut map,
            entity,
            &mut collision,
            input,
            &mut position,
            sprite,
            &mut interact,
            action_history.as_deref_mut(),
        );
    }
}
pub fn npc_movement_update(
    mut map: ResMut<Map>,
    mut npc_query: Query<(Entity, &mut MovementInput, &mut Position, Option<&Sprite>, &mut Collision, &mut Interact, Option<&mut ActionHistory>, Option<&Velocity>), Without<PlayerTag>>,
) {
    for (entity, mut movement_input, mut position, sprite, mut collision, mut interact, mut action_history, velocity) in npc_query.iter_mut() {
        let times = if let Some(velocity) = velocity {
            if let InteractData::Projectile { ref mut recent_spawn, .. } = interact.data {
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
            move_update(
                &mut map,
                entity,
                &mut collision,
                &movement_input,
                &mut position,
                sprite,
                &mut interact,
                action_history.as_deref_mut(),
            );
        }
    }
}
pub fn player_movement_input_update(player_input: Res<PlayerInput>, mut query: Query<&mut MovementInput, With<PlayerTag>>) {
    for mut movement_input in query.iter_mut() {
        *movement_input = player_input.input_movement;
    }
}

#[derive(Component)]
pub struct LootableTag;
pub fn interact_update(
    mut query: Query<&mut Interact>,
    mut commands: Commands,
    mut map: ResMut<Map>,
    mut dialogue: ResMut<Dialogue>,

    character_type_query: Query<&CharacterType>,
    mut health_query: Query<&mut Health>,
    mut lootable_query: Query<&mut Inventory, With<LootableTag>>,
) {
    for mut interact in query.iter_mut() {
        if let Some(info) = &interact.info {
            match interact.data {
                InteractData::Player => {
                    if let Ok(_character_type) = character_type_query.get(info.other_entity) {
                        dialogue.activate("Bruh".to_string(), vec![
                            ("Option 1".to_string(), DialogueOption::Leave),
                            ("Option 2".to_string(), DialogueOption::Leave),
                            ("Option 3".to_string(), DialogueOption::Leave),
                        ]);
                    }
                    if let Ok(_lootable_inventory) = lootable_query.get(info.other_entity) {
                        dialogue.activate("LOOTABLE INVENTORY".to_string(), vec![("Option 1".to_string(), DialogueOption::Leave)]);
                    }
                },
                InteractData::Lerain | InteractData::Rumdare | InteractData::Werewolf => {},
                InteractData::Projectile { damage, .. } => {
                    // Collision!
                    map.destroy(info.position.x as usize, info.position.y as usize, &mut commands);
                    if let Ok(mut health) = health_query.get_mut(info.other_entity) {
                        health.damage(damage);
                    }
                },
            }
            interact.info = None;
        }
    }
}

pub fn collision_update(
    mut query: Query<(&mut Collision, &Position)>,
    mut commands: Commands,
    mut map: ResMut<Map>,
) {
    for (mut collision, position) in query.iter_mut() {
        if collision.collided {
            collision.collided = false;
            match collision.collision_type {
                CollisionType::Sensor => {
                    map.destroy(position.x as usize, position.y as usize, &mut commands);
                },
                _ => {},
            }
        }
    }
}

pub fn destroy_check_update(mut commands: Commands, mut map: ResMut<Map>, query: Query<(&Position, &Health)>) {
    for (position, health) in query.iter() {
        if health.value == 0 {
            map.destroy(position.x as usize, position.y as usize, &mut commands);
        }
    }
}
