use std::{hash::Hash, collections::HashSet};
use bevy::{
    ecs::system::EntityCommands,
    prelude::{Entity, Commands, World, FromWorld, Component},
};
use crate::{
    character::{CharacterBundle, Interact, CharacterType, Health, ActionHistory, MovementInput, CharacterData},
    map_setup::town,
    inventory::{Equipment, Inventory, Item}, behaviors::pathfinder::data::PathfinderGlobalData,
};

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
    Exit,
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
    Obstacle {
        occupier: Option<Occupier>,
    },
}
impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Tile::Ground { .. } => matches!(other, Tile::Ground { .. }),
            Tile::Wall => matches!(other, Tile::Wall),
            Tile::Obstacle { .. } => matches!(other, Tile::Obstacle { .. }),
        }
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
#[derive(Clone)]
pub struct Occupier {
    pub entity: Entity,
    pub sprite: crate::character::Sprite,
    pub collision_type: CollisionType,
}
impl Occupier {
    pub fn new(entity: Entity, sprite: crate::character::Sprite, collision_type: CollisionType) ->  Self {
        Occupier { entity, sprite, collision_type }
    }
}

#[derive(Component)]
pub struct Collision {
    pub collided: bool,
    pub collision_type: CollisionType,
}
impl Collision {
    pub fn new(collision_type: CollisionType) -> Self {
        Collision {
            collided: false,
            collision_type,
        }
    }
}
#[derive(Clone)]
pub enum CollisionType {
    Solid,
    Sensor,
}
impl Tile {
    pub fn new_ground(zone: Zone) -> Self {
        Tile::Ground { occupier: None, zone }
    }
    pub fn default_ground() -> Self {
        Self::new_ground(Zone::Road)
    }
    pub fn is_occupied(&self, collision: &mut Collision) -> bool {
        match self {
            Tile::Ground { occupier, .. } => occupier.is_some(),
            Tile::Wall => {
                collision.collided = true;
                true
            },
            Tile::Obstacle { occupier } => !matches!(collision.collision_type, CollisionType::Sensor) || occupier.is_some(),
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
    pub fn spawn<F: FnOnce(EntityCommands)>(
        &mut self,
        commands: &mut Commands,
        sprite: crate::character::Sprite,
        position: Position,
        velocity: Velocity,
        collision_type: CollisionType,
        spawned_callback: F,
    ) {
        if let Some(tile) = self.get_mut(position.x as usize, position.y as usize) {
            if let Tile::Ground { occupier, .. } | Tile::Obstacle { occupier } = tile {
                let mut entity = commands.spawn();
                *occupier = Some(Occupier::new(entity.id(), sprite, collision_type));
                entity
                    .insert(sprite)
                    .insert(position)
                    .insert(velocity);
                spawned_callback(entity);
            }
        }
    }
    pub fn spawn_character(
        &mut self,
        commands: &mut Commands,
        sprite: crate::character::Sprite,
        position: Position,
        health: Health,
        character_type: CharacterType,
        character_data: CharacterData,
        spawned_callback: fn(EntityCommands),
    ) {
        if let Some(tile) = self.get_mut(position.x as usize, position.y as usize) {
            if let Tile::Ground {
                occupier: ref mut occupier_option,
                ..
             } = tile {
                let mut entity = commands.spawn();
                *occupier_option = Some(Occupier::new(entity.id(), sprite, CollisionType::Solid));
                entity.insert_bundle(CharacterBundle {
                    input_data: crate::character::MovementInput::Idle,
                    sprite,
                    position,
                    health,
                    interact: Interact::from(&character_type),
                    character_type,
                    character_data,
                    action_history: ActionHistory::new(60),
                    inventory: Inventory::new(
                        vec![
                            Box::new(Item::new_pistol()),
                        ],
                    ),
                    equipment: Equipment::default(),
                    collision: Collision::new(CollisionType::Solid),
                });
                spawned_callback(entity);
            }
        }
    }
    pub fn destroy(&mut self, x: usize, y: usize, commands: &mut Commands) {
        if let Some(tile) = self.get_mut(x, y) {
            if let Tile::Ground { occupier, .. } | Tile::Obstacle { occupier } = tile {
                if let Some(o) = occupier {
                    commands.entity(o.entity).despawn();
                    *occupier = None;
                }
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
    pub fn get_in_vision<'a>(&'a self, map_cache: &'a mut MapCache, position: Position) -> &'a HashSet::<Position> {
        map_cache.in_vision.clear();

        self.vision_recursion(position.clone(), &mut map_cache.in_vision, |p| p.x += 1, |p| p.y += 1, |p, i| p.x = i.x);
        self.vision_recursion(position.clone(), &mut map_cache.in_vision, |p| p.x += 1, |p| p.y -= 1, |p, i| p.x = i.x);
        self.vision_recursion(position.clone(), &mut map_cache.in_vision, |p| p.x -= 1, |p| p.y -= 1, |p, i| p.x = i.x);
        self.vision_recursion(position, &mut map_cache.in_vision, |p| p.x -= 1, |p| p.y += 1, |p, i| p.x = i.x);

        self.vision_recursion(position.clone(), &mut map_cache.in_vision, |p| p.y += 1, |p| p.x += 1, |p, i| p.y = i.y);
        self.vision_recursion(position.clone(), &mut map_cache.in_vision, |p| p.y += 1, |p| p.x -= 1, |p, i| p.y = i.y);
        self.vision_recursion(position.clone(), &mut map_cache.in_vision, |p| p.y -= 1, |p| p.x -= 1, |p, i| p.y = i.y);
        self.vision_recursion(position, &mut map_cache.in_vision, |p| p.y -= 1, |p| p.x += 1, |p, i| p.y = i.y);

        &map_cache.in_vision
    }
    fn vision_recursion(
        &self,
        initial_position: Position,
        in_vision: &mut HashSet::<Position>,
        increment_1_position: fn(&mut Position),
        increment_2_position: fn(&mut Position),
        reset_1_position: fn(&mut Position, &Position),
    ) {
        let mut max_value: Option<usize> = None;
        let mut position = initial_position.clone();
        let mut insert = |position: Position| {
            const DISTANCE: u32 = 20;
            if position.distance(&initial_position) < DISTANCE * DISTANCE {
                in_vision.insert(position);
            }
        };
        while let Some(tile) = self.get(position.x as usize, position.y as usize) {
            if matches!(tile, Tile::Wall) {
                insert(position.clone());
                break;
            }
            insert(position.clone());
            let mut saved_position = position.clone();
            increment_1_position(&mut position);
            let mut increment_count = 0;
            while let Some(tile) = self.get(position.x as usize, position.y as usize) {
                increment_count += 1;
                if let Some(max_value) = max_value {
                    if increment_count > max_value {
                        break;
                    }
                }
                if matches!(tile, Tile::Wall) {
                    insert(position.clone());
                    break;
                }
                insert(position.clone());
                saved_position = position.clone();
                increment_1_position(&mut position);
            }
            if let Some(max_value_size) = max_value {
                if increment_count < max_value_size {
                    max_value = Some(increment_count);
                }
            } else {
                max_value = Some(increment_count);
            }
            position = saved_position.clone();
            reset_1_position(&mut position, &initial_position);
            increment_2_position(&mut position);
        }
    }
}
impl FromWorld for Map {
    fn from_world(world: &mut World) -> Self {
        let mut map = Map::new::<60, 30>();
        town(&mut map, world.resource::<PathfinderGlobalData>());
        map
    }
}
#[derive(Default)]
pub struct MapCache {
    in_vision: HashSet<Position>,
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
    pub const fn new(x: i32, y: i32) -> Self {
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
impl std::ops::Mul<i32> for Position {
    type Output = Position;
    fn mul(self, rhs: i32) -> Self::Output {
        Position::new(self.x * rhs, self.y * rhs)
    }
}

#[derive(Default, Clone, Component)]
pub struct Velocity {
    pub movement: MovementInput,
    pub speed: i32,
}
impl Velocity {
    pub fn new(movement: MovementInput, speed: i32) -> Velocity {
        Velocity { movement, speed }
    }
}
