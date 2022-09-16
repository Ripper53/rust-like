use std::hash::Hash;
use bevy::{prelude::*, ecs::system::EntityCommands};
use crate::{character::{CharacterBundle, Interact, CharacterType}, map_setup::town};

#[derive(Clone)]
pub enum Zone {
    Road,
    Offroad,
    Home,
    KrillTheater { zone: KrillTheaterZone },
}
#[derive(Clone)]
pub enum KrillTheaterZone {
    Free,
    LineUp(Position),
}
impl Tile {
    pub fn krill_theater(&self) -> Option<&KrillTheaterZone> {
        match self {
            Tile::Ground { zone, .. } => if let Zone::KrillTheater { zone } = zone {
                Some(zone)
            } else {
                None
            },
            _ => None,
        }
    }
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
    pub fn new_ground(zone: Zone) -> Self {
        Tile::Ground { occupier: None, zone }
    }
    pub fn default_ground() -> Self {
        Self::new_ground(Zone::Road)
    }
    pub fn is_occupied(&self) -> bool {
        match self {
            Tile::Ground { occupier, .. } => occupier.is_some(),
            Tile::Wall => true,
        }
    }
}
pub struct Map {
    values: Vec::<Tile>,
    size_x: usize,
    size_y: usize,
}
impl Map {
    pub fn new<const X: usize, const Y: usize>() -> Map {
        let mut values = Vec::<Tile>::with_capacity(X * Y);
        Self::fill_values::<X, Y>(&mut values);
        Map {
            values,
            size_x: X,
            size_y: Y,
        }
    }
    pub fn initialize<const X: usize, const Y: usize>(&mut self) {
        self.values.clear();
        Self::fill_values::<X, Y>(&mut self.values);
        self.size_x = X;
        self.size_y = Y;
    }
    fn fill_values<const X: usize, const Y: usize>(values: &mut Vec::<Tile>) {
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
    }
    pub fn get_size_x(&self) -> usize {
        self.size_x
    }
    pub fn get_size_y(&self) -> usize {
        self.size_y
    }
    pub fn spawn_character(
        &mut self,
        commands: &mut Commands,
        sprite: crate::character::Sprite,
        position: Position,
        velocity: Velocity,
        data: CharacterType,
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
        if x < self.size_x && y < self.size_y {
            self.values.get(x + (self.size_x * y))
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        if x < self.size_x && y < self.size_y {
            self.values.get_mut(x + (self.size_x * y))
        } else {
            None
        }
    }
}
impl FromWorld for Map {
    fn from_world(_world: &mut World) -> Self {
        let mut map = Map::new::<60, 30>();
        town(&mut map);
        map
    }
}
impl Position {
    pub fn get_from_map<'a>(&'a self, map: &'a Map) -> Option<&Tile> {
        if self.x < 0 || self.y < 0 {
            None
        } else {
            map.get(self.x as usize, self.y as usize)
        }
    }
    pub fn get_mut_from_map<'a>(&'a self, map: &'a mut Map) -> Option<&mut Tile> {
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
    pub fn new(x: i32, y: i32) -> Self {
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

pub fn physics_update(map: Res<Map>, mut query: Query<(&mut Position, &Velocity)>) {
    query.par_for_each_mut(32, |(mut pos, vel)| {
        let clone_pos = pos.clone() + vel;
        if let Some(tile) = map.get(clone_pos.x as usize, clone_pos.y as usize) {
            if matches!(tile, Tile::Ground { .. }) {
                *pos += vel;
            }
        }
    });
}
