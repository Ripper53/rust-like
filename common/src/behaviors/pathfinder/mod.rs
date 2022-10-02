pub mod util;
pub mod lerain;
pub mod rumdare;
pub mod werewolf;
pub mod data;

use bevy::prelude::{Query, Res, ResMut};
use pathfinding::prelude::astar;
use crate::{
    physics::{Map, Position, Collision, Tile, CollisionType, MapCache},
    character::{CharacterType, CharacterData, MovementInput},
    map_brain::{BehaviorData, CharacterBehaviorData},
};

use self::data::PathfinderGlobalData;

#[derive(Default)]
struct Pathfinder {
    current_goal: Position,
    last_goal: Position,
    last_path: Vec<Position>,
    path_index: usize,
}

struct SkipTurn {
    count: u32,
    skip_at: u32,
}
impl SkipTurn {
    /// False to skip turn, otherwise true.
    fn check(&mut self) -> bool {
        if self.count == self.skip_at {
            self.count = 0;
            false
        } else {
            self.count += 1;
            true
        }
    }
    fn reset_count(&mut self) {
        self.count = 0;
    }
}

pub struct ReachedGoalParams<'a> {
    pub map: &'a Map,
    pub character_type: &'a CharacterType,
    pub character_data: &'a CharacterData,
    pub character_behavior_data: &'a mut CharacterBehaviorData,
    pub position: &'a Position,
}
type GetTarget = fn(
    &PathfinderGlobalData,
    &mut PathfinderBehavior,
    &Map,
    &mut MapCache,
    &CharacterType,
    &CharacterData,
    &mut CharacterBehaviorData,
    &Position,
    &Query<(&CharacterType, &CharacterData, &Position)>,
);
type ReachedGoal = fn(ReachedGoalParams);
pub struct PathfinderBehavior {
    pathfinder: Pathfinder,
    target: GetTarget,
    skip_turn: SkipTurn,
    reached_goal: Option<ReachedGoal>,
}

impl PathfinderBehavior {
    pub fn new(skip: u32, target: GetTarget) -> BehaviorData<PathfinderBehavior> {
        BehaviorData::new(PathfinderBehavior {
            pathfinder: Pathfinder::default(),
            target,
            skip_turn: SkipTurn { count: 0, skip_at: skip },
            reached_goal: None,
        })
    }

    pub fn set_skip_turn(&mut self, skip_at: u32) {
        self.skip_turn.skip_at = skip_at;
        self.skip_turn.count = 0;
    }

    pub fn set_goal(&mut self, goal: Position) -> &mut Self {
        if self.reached_goal.is_some() { return self; }
        self.pathfinder.current_goal = goal;
        self
    }

    pub fn force_goal(&mut self, goal: Position) -> &mut Self {
        self.reached_goal = None;
        self.pathfinder.current_goal = goal;
        self
    }

    pub fn reach_goal(&mut self, reached_goal: ReachedGoal) {
        self.reached_goal = Some(reached_goal);
    }

    pub fn is_at(&self, position: Position) -> bool {
        self.pathfinder.current_goal == position
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
    pub fn distance(&self, position: &Position) -> u32 {
        let diff = position - self;
        (diff.x * diff.x) as u32 + (diff.y * diff.y) as u32
    }
}

pub fn pathfinder_update(
    map: Res<Map>,
    mut map_cache: ResMut<MapCache>,
    pathfinder_global_data: Res<PathfinderGlobalData>,
    mut query: Query<(
        &mut BehaviorData<PathfinderBehavior>,
        &CharacterType,
        &CharacterData,
        &mut CharacterBehaviorData,
        &Position,
        &mut MovementInput,
    )>,
    mut collision_query: Query<&mut Collision>,
    search_query: Query<(&CharacterType, &CharacterData, &Position)>,
) {
    for (mut pathfinder, character_type, character_data, mut character_behavior_data, position, mut movement_input) in query.iter_mut() {
        if pathfinder.behavior.is_at(position.clone()) {
            // We have reached our goal,
            // forget the path whence we came.
            if let Some(reached_goal) = pathfinder.behavior.reached_goal {
                reached_goal(ReachedGoalParams {
                    map: &map,
                    character_type: &character_type,
                    character_behavior_data: &mut character_behavior_data,
                    character_data: &character_data,
                    position: &position,
                });
                pathfinder.behavior.reached_goal = None;
            }
        }
        *movement_input = if pathfinder.check_conditions() {
            if pathfinder.behavior.skip_turn.check() {
                (pathfinder.behavior.target)(
                    &pathfinder_global_data,
                    &mut pathfinder.behavior,
                    &map,
                    &mut map_cache,
                    character_type,
                    character_data,
                    &mut character_behavior_data,
                    position,
                    &search_query,
                );
                let mut pathfinder = &mut pathfinder.behavior.pathfinder;
                // Calculate path.
                if let Some((path, _)) = astar(
                    position,
                    |p| p.successors(&mut collision_query, &map, &pathfinder.current_goal),
                    |p| p.distance(&pathfinder.current_goal),
                    |p| *p == pathfinder.current_goal,
                ) {
                    pathfinder.last_path = path;
                    pathfinder.path_index = 1;
                    pathfinder.last_goal = pathfinder.current_goal;
                }

                if let Some(target) = pathfinder.last_path.get(pathfinder.path_index) {
                    pathfinder.path_index += 1;
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
                }
            } else {
                MovementInput::Idle
            }
        } else {
            pathfinder.behavior.skip_turn.reset_count();
            MovementInput::Idle
        };
    }
}
