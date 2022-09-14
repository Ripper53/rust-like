use std::cmp::Ordering;

use bevy::prelude::*;
use crate::{physics::*, character::{MovementInput, CharacterType}};
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
    fn execute(&mut self, map: &Map, movement_input: &mut MovementInput, position: &Position) {
        // Calculate path.
        if let Some((path, _)) = astar(
            position,
            |p| p.successors(map, &self.current_goal),
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
}
impl Behavior {
    pub fn skip_movement(skip_at: u32) -> BehaviorData {
        BehaviorData::new(Behavior::SlowMovement { pathfinder: Pathfinder::default(), skip_turn: SkipTurn { count: 0, skip_at } })
    }
    pub fn default_slow_movement() -> BehaviorData {
        Self::skip_movement(1)
    }
}

impl Behavior {
    fn get_pathfinder_target(search_query: &Query<(&CharacterType, &Position)>, character_data: &CharacterType, pathfinder: &mut Pathfinder, position: &Position) {
        let mut get_target = |target_character_data: CharacterType| {
            if let Some((_, target)) = search_query.iter().min_by(|(data_a, pos_a), (data_b, pos_b)| {
                if **data_a == target_character_data {
                    if **data_b == target_character_data {
                        let diff_a = position.distance(pos_a);
                        let diff_b = position.distance(pos_b);
                        diff_a.cmp(&diff_b)
                    } else {
                        Ordering::Less
                    }
                } else if **data_b == target_character_data {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }) {
                pathfinder.current_goal = *target;
            }
        };
        match character_data {
            CharacterType::Player { .. } => {},
            CharacterType::Lerain => get_target(CharacterType::Werewolf),
            CharacterType::Rumdare => get_target(CharacterType::Werewolf),
            CharacterType::Werewolf => get_target(CharacterType::Player),
        }
    }
    fn execute(
        &mut self,
        map: &Map,
        character_data: &CharacterType,
        movement_input: &mut MovementInput,
        position: &Position,
        velocity: &mut Velocity,

        pathfinder_search_query: &Query<(&CharacterType, &Position)>,
    ) {
        match self {
            Behavior::SlowMovement { pathfinder, skip_turn } => {
                Self::get_pathfinder_target(pathfinder_search_query, character_data, pathfinder, position);
                if skip_turn.skip_at != 0 && skip_turn.count == skip_turn.skip_at {
                    skip_turn.count = 0;
                    *movement_input = MovementInput::Idle;
                } else {
                    skip_turn.count += 1;
                    pathfinder.execute(map, movement_input, position);
                }
            },
        }
    }
}

impl Position {
    fn successors(&self, map: &Map, target: &Position) -> Vec<(Position, u32)> {
        let mut neighbors = Vec::<Position>::with_capacity(4);
        let mut add_to_neighbors = |position: Position| {
            if let Some(tile) = map.get(position.x as usize, position.y as usize) {
                if !tile.is_occupied() || position == *target {
                    neighbors.push(position);
                }
            }
        };
        add_to_neighbors(Position::new(self.x, self.y + 1));
        add_to_neighbors(Position::new(self.x + 1, self.y));
        add_to_neighbors(Position::new(self.x, self.y - 1));
        add_to_neighbors(Position::new(self.x - 1, self.y));
        neighbors.into_iter().map(|p| (p, 1)).collect()
    }
    fn distance(&self, position: &Position) -> u32 {
        let diff = position - self;
        (diff.x * diff.x) as u32 + (diff.y * diff.y) as u32
    }
}

pub fn brain_update(
    mut query: Query<(&mut Brain, &mut MovementInput, &CharacterType, &Position, &mut Velocity)>,
    map: Res<Map>,

    pathfinder_search_query: Query<(&CharacterType, &Position)>,
) {
    query.par_for_each_mut(32, |(mut brain, mut movement_input, character_data, position, mut velocity)| {
        for behavior in brain.behaviors.iter_mut() {
            if behavior.conditions.len() != 0 {
                for condition in behavior.conditions.iter() {
                    if condition() {
                        behavior.behavior.execute(
                            &map, character_data, &mut movement_input, position, &mut velocity,
                            &pathfinder_search_query,
                        );
                        break;
                    }
                }
            } else {
                behavior.behavior.execute(
                    &map, character_data, &mut movement_input, position, &mut velocity,
                    &pathfinder_search_query,
                );
            }
        }
    });
}
