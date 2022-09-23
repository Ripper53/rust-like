use std::cmp::Ordering;

use bevy::prelude::{Component, Query, Res};
use rand::prelude::*;
use crate::{physics::*, character::{MovementInput, CharacterType, CharacterData, WereForm, Sprite}};
use pathfinding::prelude::astar;

#[derive(Component)]
pub struct Brain {
    behaviors: Vec<BehaviorData>,
}
impl Brain {
    pub fn new(behaviors: Vec<BehaviorData>) -> Brain {
        Brain { behaviors }
    }
}

#[derive(Default)]
pub struct Pathfinder {
    current_goal: Position,
    last_goal: Position,
    last_path: Vec<Position>,
    path_index: usize,
}
impl Pathfinder {
    fn execute(&mut self, collision_query: &mut Query<&mut Collision>, map: &Map, movement_input: &mut MovementInput, position: &Position) {
        // Calculate path.
        if let Some((path, _)) = astar(
            position,
            |p| p.successors(collision_query, map, &self.current_goal),
            |p| p.distance(&self.current_goal),
            |p| *p == self.current_goal,
        ) {
            self.last_path = path;
            self.path_index = 1;
            self.last_goal = self.current_goal;
        }

        *movement_input = if let Some(target) = self.last_path.get(self.path_index) {
            self.path_index += 1;
            if target.y > position.y { 
                // UP
                MovementInput::North
            } else if target.x > position.x {
                // RIGHT
                MovementInput::East
            } else if target.y < position.y {
                // DOWN
                MovementInput::South
            } else if target.x < position.x {
                // LEFT
                MovementInput::West
            } else {
                MovementInput::Idle
            }
        } else {
            MovementInput::Idle
        };
    }
}

pub struct BehaviorData {
    pub behavior: Behavior,
    pub conditions: Vec<fn() -> bool>,
}
impl BehaviorData {
    pub fn new(behavior: Behavior) -> Self {
        BehaviorData { behavior, conditions: Vec::default() }
    }
    pub fn run_if(mut self, condition: fn() -> bool) -> Self {
        self.conditions.push(condition);
        self
    }
}
pub struct SkipTurn {
    count: u32,
    skip_at: u32,
}
pub enum Behavior {
    SlowMovement {
        pathfinder: Pathfinder,
        skip_turn: SkipTurn,
    },
    Werewolf,
}
impl Behavior {
    pub fn skip_movement(skip_at: u32) -> BehaviorData {
        BehaviorData::new(Behavior::SlowMovement { pathfinder: Pathfinder::default(), skip_turn: SkipTurn { count: 0, skip_at } })
    }
    pub fn default_slow_movement() -> BehaviorData {
        Self::skip_movement(1)
    }
    pub fn default_werewolf() -> BehaviorData {
        BehaviorData::new(Behavior::Werewolf)
    }
}

impl Behavior {
    fn get_pathfinder_target(map: &Map, search_query: &Query<(&CharacterType, &CharacterData, &Position)>, character_type: &CharacterType, pathfinder: &mut Pathfinder, position: &Position) {
        let get_target = |pathfinder: &mut Pathfinder, target_character_type: CharacterType| {
            let mut found_target = false;
            if let Some((_, target_data, target)) = search_query.iter().min_by(|(type_a, data_a, pos_a), (type_b, data_b, pos_b)| {
                if **type_a == target_character_type {
                    found_target = if let CharacterData::Werewolf { form } = data_a {
                        matches!(form, WereForm::Beast)
                    } else {
                        true
                    };
                    if **type_b == target_character_type {
                        let diff_a = position.distance(pos_a);
                        let diff_b = position.distance(pos_b);
                        diff_a.cmp(&diff_b)
                    } else {
                        Ordering::Less
                    }
                } else if **type_b == target_character_type {
                    found_target = true;
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }) {
                if found_target {
                    pathfinder.current_goal = *target;
                    true
                } else {
                    pathfinder.current_goal = *position;
                    false
                }
            } else {
                false
            }
        };
        let get_random_target = |pathfinder: &mut Pathfinder| {
            let x = rand::thread_rng().gen_range(0..map.get_size_x() as i32);
            let y = rand::thread_rng().gen_range(0..map.get_size_y() as i32);
            pathfinder.current_goal = Position::new(x, y);
        };
        match character_type {
            CharacterType::Player => {},
            CharacterType::Lerain => {
                if !get_target(pathfinder, CharacterType::Werewolf) && pathfinder.current_goal == *position {
                    if let Some(tile) = map.get(position.x as usize, position.y as usize) {
                        if let Some(krill_theater) = tile.krill_theater() {
                            match krill_theater {
                                KrillTheaterZone::Free => {
                                    get_random_target(pathfinder);
                                },
                                KrillTheaterZone::LineUp(target) => {
                                    pathfinder.current_goal = *target;
                                },
                            }
                        } else {
                            get_random_target(pathfinder);
                        }
                    }
                }
            },
            CharacterType::Rumdare => { get_target(pathfinder, CharacterType::Werewolf); },
            CharacterType::Werewolf => { get_target(pathfinder, CharacterType::Player); },
        }
    }
    fn execute(
        &mut self,
        map: &Map,
        character_type: &CharacterType,
        character_data: &mut CharacterData,
        movement_input: &mut MovementInput,
        position: &Position,

        collision_query: &mut Query<&mut Collision>,
        pathfinder_search_query: &Query<(&CharacterType, &Position)>,
    ) {
        match self {
            Behavior::SlowMovement { pathfinder, skip_turn } => {
                Self::get_pathfinder_target(map, pathfinder_search_query, character_type, pathfinder, position);
                if skip_turn.skip_at != 0 && skip_turn.count == skip_turn.skip_at {
                    skip_turn.count = 0;
                    *movement_input = MovementInput::Idle;
                } else {
                    skip_turn.count += 1;
                    pathfinder.execute(collision_query, map, movement_input, position);
                }
            },
            Behavior::Werewolf => {
                if let CharacterData::Werewolf { form } = character_data {
                    if rand::thread_rng().gen_range(0..10 as i32) < 2 {
                        *form = WereForm::Beast;
                    } else {
                        *form = WereForm::Human;
                    }
                }
            },
        }
    }
}

impl Position {
    fn successors(&self, collision_query: &mut Query<&mut Collision>, map: &Map, target: &Position) -> Vec<(Position, u32)> {
        self.neighbors(collision_query, map, target).into_iter().map(|p| (p, 1)).collect()
    }
    fn is_neighbor(&self, collision_query: &mut Query<&mut Collision>, map: &Map) -> bool {
        if let Some(tile) = map.get(self.x as usize, self.y as usize) {
            match tile {
                Tile::Ground { occupier, .. } | Tile::Obstacle { occupier } => {
                    if let Some(occupier) = occupier {
                        if let Ok(mut collision) = collision_query.get_mut(occupier.entity) {
                            return !tile.is_occupied(&mut collision);
                        }
                    }
                    !tile.is_occupied(&mut Collision::new(CollisionType::Solid))
                },
                Tile::Wall => !tile.is_occupied(&mut Collision::new(CollisionType::Solid)),
            }
        } else {
            false
        }
    }
    fn neighbors(&self, collision_query: &mut Query<&mut Collision>, map: &Map, target: &Position) -> Vec<Position> {
        let mut neighbors = Vec::<Position>::with_capacity(4);
        let mut add_to_neighbors = |position: Position| {
            if position.is_neighbor(collision_query, map) || position == *target {
                neighbors.push(position);
            }
        };
        add_to_neighbors(Position::new(self.x, self.y + 1));
        add_to_neighbors(Position::new(self.x + 1, self.y));
        add_to_neighbors(Position::new(self.x, self.y - 1));
        add_to_neighbors(Position::new(self.x - 1, self.y));
        neighbors
    }
    fn distance(&self, position: &Position) -> u32 {
        let diff = position - self;
        (diff.x * diff.x) as u32 + (diff.y * diff.y) as u32
    }
}

pub fn brain_update(
    mut query: Query<(&mut Brain, &mut MovementInput, &CharacterType, &mut CharacterData, &Position)>,
    map: Res<Map>,

    mut collision_query: Query<&mut Collision>,
    pathfinder_search_query: Query<(&CharacterType, &Position)>,
) {
    for (mut brain, mut movement_input, character_type, mut character_data, position) in query.iter_mut() {
        for behavior in brain.behaviors.iter_mut() {
            if behavior.conditions.len() != 0 {
                for condition in behavior.conditions.iter() {
                    if condition() {
                        behavior.behavior.execute(
                            &map, character_type, &mut character_data, &mut movement_input, position,
                            &mut collision_query, &pathfinder_search_query,
                        );
                        break;
                    }
                }
            } else {
                behavior.behavior.execute(
                    &map, character_type, &mut character_data, &mut movement_input, position,
                    &mut collision_query, &pathfinder_search_query,
                );
            }
        }
    }
}

pub fn werewolf_update(mut query: Query<(&CharacterData, &mut Sprite)>) {
    for (character_data, mut sprite) in query.iter_mut() {
        if let CharacterData::Werewolf { form } = character_data {
            match form {
                WereForm::Human => *sprite = Sprite::new('C'),
                WereForm::Beast => *sprite = Sprite::new('W'),
            }
        }
    }
}
