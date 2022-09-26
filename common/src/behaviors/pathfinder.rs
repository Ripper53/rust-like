use bevy::prelude::{Query, Res};
use pathfinding::prelude::astar;
use crate::{
    physics::{Map, Position, Collision, Tile, CollisionType},
    character::{CharacterType, CharacterData, MovementInput},
    map_brain::BehaviorData,
};

#[derive(Default)]
pub struct Pathfinder {
    pub current_goal: Position,
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
    pub pathfinder: Pathfinder,
    target: fn(
        &mut PathfinderBehavior,
        &Map,
        &CharacterType,
        &CharacterData,
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
            &CharacterData,
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
    mut query: Query<(&mut BehaviorData<PathfinderBehavior>, &CharacterType, &CharacterData, &Position, &mut MovementInput)>,
    search_query: Query<(&CharacterType, &CharacterData, &Position)>,
    mut collision_query: Query<&mut Collision>,
) {
    for (mut pathfinder, character_type, character_data, position, mut movement_input) in query.iter_mut() {
        *movement_input = if pathfinder.check_conditions() {
            if pathfinder.behavior.skip_turn.check() {
                (pathfinder.behavior.target)(
                    &mut pathfinder.behavior,
                    &map,
                    character_type,
                    character_data,
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
