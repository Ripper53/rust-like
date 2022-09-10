use std::hash::Hash;
use bevy::prelude::*;
use crate::{character::CharacterBundle, map_setup::legal};
use rand::prelude::*;

#[derive(Clone)]
pub enum Zone {
    None,
    Lawyer,
}
/// Enum value true if space is occupied, otherwise false.
#[derive(Clone)]
pub enum Tile {
    Ground {
        sprite: Option<crate::character::Sprite>,
        zone: Zone,
    },
    Wall,
}
impl Tile {
    pub fn default_ground() -> Self {
        Tile::Ground { sprite: None, zone: Zone::None }
    }
}
pub struct Map<const X: usize, const Y: usize> {
    values: Vec::<Tile>,
}
impl<const X: usize, const Y: usize> Map<X, Y> {
    pub fn new() -> Map<X, Y> {
        let mut values = Vec::<Tile>::with_capacity(X * Y);
        for _ in 0..X * Y {
            values.push(Tile::default_ground());
        }
        
        // Borders
        for y in 0..Y {
            values[X * y] = Tile::Wall;
            values[X - 1 + (X * y)] = Tile::Wall;
        }
        for x in 0..X {
            values[x] = Tile::Wall;
            values[x + (X * (Y - 1))] = Tile::Wall;
        }

        Map::<X, Y> {
            values,
        }
    }
    pub fn spawn_character(&mut self, sprite: crate::character::Sprite, position: Position, velocity: Velocity) -> Option<CharacterBundle> {
        if let Some(tile) = self.get_mut(position.x as usize, position.y as usize) {
            if let Tile::Ground {
                sprite: ref mut sprite_option,
                ..
             } = tile {
                *sprite_option = Some(sprite);
                return Some(CharacterBundle {
                    input_data: crate::character::MovementInput::Idle,
                    sprite,
                    position,
                    velocity,
                });
            }
        }
        None
    }
    pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
        if x < X && y < Y {
            self.values.get(x + (X * y))
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        if x < X && y < Y {
            self.values.get_mut(x + (X * y))
        } else {
            None
        }
    }
}
impl<const X: usize, const Y: usize> FromWorld for Map<X, Y> {
    fn from_world(_world: &mut World) -> Self {
        let mut map = Map::<X, Y>::new();
        legal(&mut map);
        map
    }
}
impl Position {
    pub fn get_from_map<'a, const X: usize, const Y: usize>(&'a self, map: &'a Map<X, Y>) -> Option<&Tile> {
        if self.x < 0 || self.y < 0 {
            None
        } else {
            map.get(self.x as usize, self.y as usize)
        }
    }
    pub fn get_mut_from_map<'a, const X: usize, const Y: usize>(&'a self, map: &'a mut Map<X, Y>) -> Option<&mut Tile> {
        if self.x < 0 || self.y < 0 {
            None
        } else {
            map.get_mut(self.x as usize, self.y as usize)
        }
    }
}

#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position { x, y }
    }
}
impl Eq for Position {}
impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
    fn ne(&self, other: &Self) -> bool {
        self.x != other.x || self.y != other.y
    }
}
impl Hash for Position {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.hash(state);
        self.y.hash(state);
    }
}
impl std::ops::Add for Position {
    type Output = Position;
    fn add(self, rhs: Self) -> Self::Output {
        Position::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl std::ops::AddAssign for Position {
    fn add_assign(&mut self, rhs: Position) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl std::ops::Add<&Velocity> for Position {
    type Output = Position;
    fn add(self, rhs: &Velocity) -> Position {
        Position::new(self.x + rhs.x, self.y + rhs.y)
    }
}
impl std::ops::AddAssign<&Velocity> for Position {
    fn add_assign(&mut self, rhs: &Velocity) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}
impl std::ops::Sub for Position {
    type Output = Position;
    fn sub(self, rhs: Self) -> Self::Output {
        Position::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl std::ops::Sub for &Position {
    type Output = Position;
    fn sub(self, rhs: Self) -> Self::Output {
        Position::new(self.x - rhs.x, self.y - rhs.y)
    }
}
impl std::ops::Mul for Position {
    type Output = Position;
    fn mul(self, rhs: Self) -> Self::Output {
        Position::new(self.x * rhs.x, self.y * rhs.y)
    }
}

#[derive(Clone, Component)]
pub struct Velocity {
    pub x: i32,
    pub y: i32,
}
impl Velocity {
    pub fn new(x: i32, y: i32) -> Velocity {
        Velocity { x, y }
    }
}

pub fn physics_update<const X: usize, const Y: usize>(map: Res<Map<X, Y>>, mut query: Query<(&mut Position, &Velocity)>) {
    query.par_for_each_mut(32, |(mut pos, vel)| {
        let clone_pos = pos.clone() + vel;
        if let Some(tile) = map.get(clone_pos.x as usize, clone_pos.y as usize) {
            if matches!(tile, Tile::Ground { .. }) {
                *pos += vel;
            }
        }
    });
}
