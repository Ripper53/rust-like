use bevy::prelude::*;
use crate::physics::*;
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

pub enum Behavior {
    Pathfinder {
        current_goal: Position,
        last_goal: Position,
        last_path: Vec<Position>,
        path_index: usize,
    },
}

impl Behavior {
    fn execute(&mut self, position: &mut Position, velocity: &mut Velocity) {
        match self {
            Behavior::Pathfinder {
                ref mut current_goal,
                ref last_goal,
                ref mut last_path,
                ref mut path_index,
            } => {
                // Check if new path needs to be calculated
                if current_goal != last_goal {
                    *path_index = 0;
                    *current_goal = *last_goal;
                    if let Some((path, _)) = astar(
                        &Position::new(position.x, position.y),
                        |p| p.successors(),
                        |p| p.distance(&current_goal),
                        |p| *p == *current_goal,
                    ) {
                        *last_path = path;
                    }
                }

                if let Some(p) = last_path.get(*path_index) {
                    *path_index += 1;
                }
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

pub fn brain_update(mut query: Query<(&mut Brain, &mut Position, &mut Velocity)>) {
    query.par_for_each_mut(32, |(mut brain, mut position, mut velocity)| {
        for behavior in brain.behaviors.iter_mut() {
            behavior.execute(&mut position, &mut velocity);
        }
    });
}
