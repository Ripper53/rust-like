use bevy::prelude::*;
use crate::{physics::*, character::MovementInput};
use pathfinding::prelude::astar;

#[derive(Component)]
pub struct Brain {
    behaviors: Vec<Behavior>,
}
impl Brain {
    pub fn new(behaviors: Vec<Behavior>) -> Brain {
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
    fn execute(&mut self, map: &Map, movement_input: &mut MovementInput, position: &mut Position) {
        // Calculate path.
        if let Some((path, _)) = astar(
            position,
            |p| p.successors(map),
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

pub enum Behavior {
    SlowMovement {
        pathfinder: Pathfinder,
        skip_turn: bool,
    },
}
impl Behavior {
    pub fn default_slow_movement() -> Self {
        Behavior::SlowMovement { pathfinder: Pathfinder::default(), skip_turn: false }
    }
}

impl Behavior {
    fn execute(&mut self, map: &Map, movement_input: &mut MovementInput, position: &mut Position, velocity: &mut Velocity) {
        match self {
            Behavior::SlowMovement { pathfinder, skip_turn } => {
                if *skip_turn {
                    *skip_turn = false;
                    *movement_input = MovementInput::Idle;
                } else {
                    *skip_turn = true;
                    pathfinder.current_goal = Position::new(40, 3);
                    pathfinder.execute(map, movement_input, position);
                }
            },
        }
    }
}

impl Position {
    fn successors(&self, map: &Map) -> Vec<(Position, u32)> {
        let mut neighbors = Vec::<Position>::with_capacity(4);
        let mut add_to_neighbors = |position: Position| {
            if let Some(tile) = map.get(position.x as usize, position.y as usize) {
                if !tile.is_occupied() {
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
    mut query: Query<(&mut Brain, &mut MovementInput, &mut Position, &mut Velocity)>,
    map: Res<Map>,
) {
    query.par_for_each_mut(32, |(mut brain, mut movement_input, mut position, mut velocity)| {
        for behavior in brain.behaviors.iter_mut() {
            behavior.execute(&map, &mut movement_input, &mut position, &mut velocity);
        }
    });
}
