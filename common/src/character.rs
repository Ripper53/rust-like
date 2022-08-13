use bevy::prelude::*;
use crate::physics::*;

#[derive(Component)]
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

#[derive(Bundle)]
pub struct CharacterBundle {
    pub sprite: Sprite,
    pub position: Position,
    pub velocity: Velocity,
}

#[derive(Component)]
pub enum MovementInput {
    North,
    East,
    South,
    West,
}

fn check_collision<const X: usize, const Y: usize>(map: &Map<X, Y>, position: &Position) -> bool {
    map.get(position.x as usize, position.y as usize).is_some()
}
pub fn movement_update<const X: usize, const Y: usize>(map: Res<Map<X, Y>>, mut query: Query<(&MovementInput, &mut Position)>) {
    query.par_for_each_mut(32, |(input, mut position)| {
        let mut move_position = |movement: Position| {
            if check_collision(&map, &(*position + movement)) { return; }
            *position += movement;
        };
        match input {
            MovementInput::North => move_position(Position::new(0, 1)),
            MovementInput::East => move_position(Position::new(1, 0)),
            MovementInput::South => move_position(Position::new(0, -1)),
            MovementInput::West => move_position(Position::new(-1, 0)),
        }
    });
}
pub fn commence_battle_update(query: Query<(&Health)>) {
}
