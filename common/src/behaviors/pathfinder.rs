use std::cmp::Ordering;
use bevy::prelude::{Query, Res};
use pathfinding::prelude::astar;
use rand::Rng;
use crate::{
    physics::{Map, Position, Collision, Tile, CollisionType, KrillTheaterZone},
    character::{CharacterType, CharacterData, MovementInput, WereForm},
    map_brain::BehaviorData,
};

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

pub struct PathfinderBehavior {
    pathfinder: Pathfinder,
    target: fn(
        &mut PathfinderBehavior,
        &Map,
        &CharacterType,
        &Position,
        &Query<(&CharacterType, &CharacterData, &Position)>,
    ),
    skip_turn: SkipTurn,
}

impl PathfinderBehavior {
    pub fn new(
        skip: u32,
        target: fn(
            &mut PathfinderBehavior,
            &Map,
            &CharacterType,
            &Position,
            &Query<(&CharacterType, &CharacterData, &Position)>,
        ),
    ) -> BehaviorData<PathfinderBehavior> {
        BehaviorData::new(PathfinderBehavior {
            pathfinder: Pathfinder::default(),
            target,
            skip_turn: SkipTurn { count: 0, skip_at: skip },
        })
    }
    fn get_pathfinder_target(
        self: &mut PathfinderBehavior,
        map: &Map,
        character_type: &CharacterType,
        position: &Position,
        search_query: &Query<(&CharacterType, &CharacterData, &Position)>,
    ) {
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
                if !get_target(&mut self.pathfinder, CharacterType::Werewolf) && self.pathfinder.current_goal == *position {
                    if let Some(tile) = map.get(position.x as usize, position.y as usize) {
                        if let Some(krill_theater) = tile.krill_theater() {
                            match krill_theater {
                                KrillTheaterZone::Free => {
                                    get_random_target(&mut self.pathfinder);
                                },
                                KrillTheaterZone::LineUp(target) => {
                                    self.pathfinder.current_goal = *target;
                                },
                            }
                        } else {
                            get_random_target(&mut self.pathfinder);
                        }
                    }
                }
            },
            CharacterType::Rumdare => { get_target(&mut self.pathfinder, CharacterType::Werewolf); },
            CharacterType::Werewolf => { get_target(&mut self.pathfinder, CharacterType::Player); },
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

pub fn pathfinder_update(
    map: Res<Map>,
    mut query: Query<(&mut BehaviorData<PathfinderBehavior>, &CharacterType, &CharacterData, &Position, &mut MovementInput)>,
    search_query: Query<(&CharacterType, &CharacterData, &Position)>,
    mut collision_query: Query<&mut Collision>,
) {
    for (mut pathfinder, character_type, character_data, position, mut movement_input) in query.iter_mut() {
        *movement_input = if pathfinder.check_conditions() {
            if pathfinder.behavior.skip_turn.check() {
                (pathfinder.behavior.target)(&mut pathfinder.behavior, &map, character_type, position, &search_query);
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
