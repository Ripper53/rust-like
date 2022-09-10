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
    fn execute(&mut self, movement_input: &mut MovementInput, position: &mut Position) {
        // Check if new path needs to be calculated
        if self.current_goal != self.last_goal {
            self.path_index = 0;
            self.current_goal = self.last_goal;
            if let Some((path, _)) = astar(
                position,
                |p| p.successors(),
                |p| p.distance(&self.current_goal),
                |p| *p == self.current_goal,
            ) {
                self.last_path = path;
            }
        }

        if let Some(target) = self.last_path.get(self.path_index) {
            self.path_index += 1;
            if target.y > position.y {
                // UP
                *movement_input = MovementInput::North;
            } else if target.x > position.x {
                // RIGHT
                *movement_input = MovementInput::East;
            } else if target.y < position.y {
                // DOWN
                *movement_input = MovementInput::South;
            } else if target.x < position.x {
                // LEFT
                *movement_input = MovementInput::West;
            }
        }
    }
}

pub enum Behavior {
    Lawyer {
        pathfinder: Pathfinder,
    },
}

impl Behavior {
    fn execute(&mut self, movement_input: &mut MovementInput, position: &mut Position, velocity: &mut Velocity) {
        match self {
            Behavior::Lawyer { pathfinder } => {
                pathfinder.current_goal = Position::new(8, 2);
                pathfinder.execute(movement_input, position);
            },
        }
    }
}

impl Position {
    fn successors(&self) -> Vec<(Position, u32)> {
        vec![Position::new(self.x, self.y + 1), Position::new(self.x + 1, self.y),
             Position::new(self.x, self.y - 1), Position::new(self.x - 1, self.y)]
             .into_iter().map(|p| (p, 1)).collect()
    }
    fn distance(&self, position: &Position) -> u32 {
        let diff = position - self;
        ((diff.x * diff.x) as u32) + ((diff.y * diff.y) as u32)
    }
}

pub fn brain_update(mut query: Query<(&mut Brain, &mut MovementInput, &mut Position, &mut Velocity)>) {
    query.par_for_each_mut(32, |(mut brain, mut movement_input, mut position, mut velocity)| {
        for behavior in brain.behaviors.iter_mut() {
            behavior.execute(&mut movement_input, &mut position, &mut velocity);
        }
    });
}
