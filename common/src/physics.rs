use std::hash::Hash;
use bevy::{prelude::*, ecs::system::EntityCommands};
use crate::{character::{CharacterBundle, Interact, CharacterData}, map_setup::legal};
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
        occupier: Option<Occupier>,
        zone: Zone,
    },
    Wall,
}
#[derive(Clone)]
pub struct Occupier {
    pub entity: Entity,
    pub sprite: crate::character::Sprite,
}
impl Occupier {
    pub fn new(entity: Entity, sprite: crate::character::Sprite) ->  Self {
        Occupier { entity, sprite }
    }
}
impl Tile {
    pub fn default_ground() -> Self {
        Tile::Ground { occupier: None, zone: Zone::None }
    }
    pub fn is_occupied(&self) -> bool {
        match self {
            Tile::Ground { occupier, .. } => occupier.is_some(),
            Tile::Wall => true,
        }
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
    pub fn spawn_character(
        &mut self,
        commands: &mut Commands,
        sprite: crate::character::Sprite,
        position: Position,
        velocity: Velocity,
        data: CharacterData,
        spawned_callback: fn(EntityCommands),
    ) {
        if let Some(tile) = self.get_mut(position.x as usize, position.y as usize) {
            if let Tile::Ground {
                occupier: ref mut occupier_option,
                ..
             } = tile {
                let mut entity = commands.spawn();
                *occupier_option = Some(Occupier::new(entity.id(), sprite));
                entity.insert_bundle(CharacterBundle {
                    input_data: crate::character::MovementInput::Idle,
                    sprite,
                    position,
                    velocity,
                    interact: Interact::from(&data),
                    data,
                });
                spawned_callback(entity);
            }
        }
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

#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}
impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position { x, y }
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
