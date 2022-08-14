use bevy::prelude::*;
use crate::physics::*;

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
}

#[derive(Bundle)]
pub struct CharacterBundle {
    pub input_data: MovementInput,
    pub sprite: Sprite,
    pub position: Position,
    pub velocity: Velocity,
}

#[derive(Component, Clone, Copy)]
pub enum MovementInput {
    Idle,
    North,
    East,
    South,
    West,
}

/// True if collided, otherwise set tile to occupied and return false.
fn check_collision<const X: usize, const Y: usize>(map: &mut Map<X, Y>, current_position: &Position, new_position: &Position, sprite: Option<&Sprite>) -> bool {
    let mut place_character_at_new_position = || {
        if let Some(Tile::Ground(ref mut value)) = new_position.get_mut_from_map::<X, Y>(map) {
            if value.is_none() {
                *value = if let Some(s) = sprite {
                    Some(*s)
                } else {
                    Some(Sprite::new('N'))
                };
                return true;
            }
        }
        false
    };
    if place_character_at_new_position() {
        if let Some(Tile::Ground(ref mut old_value)) = current_position.get_mut_from_map::<X, Y>(map) {
            *old_value = None;
        }
        false
    } else {
        true
    }
}
pub fn movement_update<const X: usize, const Y: usize>(mut map: ResMut<Map<X, Y>>, mut query: Query<(&MovementInput, &mut Position, Option<&Sprite>)>) {
    for (input, mut position, sprite) in query.iter_mut() {
        let mut move_position = |movement: Position| {
            if check_collision(&mut map, &position, &(*position + movement), sprite) { return; }
            *position += movement;
        };
        match input {
            MovementInput::North => move_position(Position::new(0, 1)),
            MovementInput::East => move_position(Position::new(1, 0)),
            MovementInput::South => move_position(Position::new(0, -1)),
            MovementInput::West => move_position(Position::new(-1, 0)),
            _ => {},
        }
    }
}
pub fn player_update(player_input: Res<PlayerInput>, mut query: Query<(&mut MovementInput, With<PlayerTag>)>) {
    for (mut movement_input, _) in query.iter_mut() {
        *movement_input = player_input.input_movement;
    }
}
