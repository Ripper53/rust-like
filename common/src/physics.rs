use std::hash::Hash;

use bevy::prelude::*;
use tui::widgets::canvas::Shape;

#[derive(Clone)]
pub enum Tile {
    Ground,
    Wall,
}
pub struct Map<const X: usize, const Y: usize> {
    values: Vec::<Tile>,
}
impl<const X: usize, const Y: usize> Map<X, Y> {
    pub fn new() -> Map<X, Y> {
        Map::<X, Y> {
            values: vec![Tile::Ground; X * Y]
        }
    }
    pub fn get(&self, x: usize, y: usize) -> Option<&Tile> {
        self.values.get(x + (Y * y))
    }
}
impl<const X: usize, const Y: usize> FromWorld for Map<X, Y> {
    fn from_world(world: &mut World) -> Self {
        Map::<X, Y>::new()
    }
}
impl<const X: usize, const Y: usize> Shape for Map<X, Y> {
    fn draw(&self, painter: &mut tui::widgets::canvas::Painter) {
    }
}

#[derive(Clone, Copy, Component)]
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
        self.x != other.x && self.y != other.y
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
            if matches!(tile, Tile::Ground) {
                *pos += vel;
            }
        }
    });
}
