use bevy::prelude::FromWorld;
use rand::Rng;
use crate::physics::Position;

pub struct PathfinderGlobalData {
    points: [Vec<Position>; 2],
    exit_points: [(Position, Position); 3],
}

pub enum CharacterType {
    Lerain,
    Rumdare,
}

impl PathfinderGlobalData {
    pub fn get_target(&self, character_type: CharacterType) -> (Position, usize) {
        match character_type {
            CharacterType::Lerain | CharacterType::Rumdare => {
                let i0 = rand::thread_rng().gen_range(0..self.points.len());
                let i1 = rand::thread_rng().gen_range(0..self.points[i0].len());
                (self.points[i0][i1], i0)
            },
        }
    }
    pub fn get_target_except(&self, character_type: CharacterType, exclude_index: usize) -> (Position, usize) {
        match character_type {
            CharacterType::Lerain | CharacterType::Rumdare => {
                let length = self.points.len();
                let mut i0 = rand::thread_rng().gen_range(0..length);
                if i0 == exclude_index {
                    if i0 == length - 1 {
                        i0 = 0;
                    } else {
                        i0 += 1;
                    }
                }
                let i1 = rand::thread_rng().gen_range(0..self.points[i0].len());
                (self.points[i0][i1], i0)
            },
        }
    }
    pub fn is_exit_point(&self, position: Position) -> Option<(Position, usize)> {
        if let Some(index) = self.exit_points.iter().position(|p| p.0 == position) {
            Some((self.exit_points[index].1, index))
        } else {
            None
        }
    }

    /// DEBUG PURPOSES
    pub fn contains_point(&self, position: Position) -> bool {
        self.points.iter().find(|v| {
            for p in v.iter() {
                if *p == position {
                    return true;
                }
            }
            false
        }).is_some()
    }
}

impl FromWorld for PathfinderGlobalData {
    fn from_world(_world: &mut bevy::prelude::World) -> Self {
        let points = [
            vec![
                Position::new(110, 42),
                Position::new(69, 10),
                Position::new(151, 10),
            ],
            vec![
                Position::new(3, 3),
            ],
        ];
        let exit_points = [
            (points[0][0], Position::new(110, 45)),
            (points[0][1], Position::new(69, 7)),
            (points[0][2], Position::new(151, 7)),
        ];
        PathfinderGlobalData {
            points,
            exit_points,
        }
    }
}
