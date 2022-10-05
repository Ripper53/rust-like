use bevy::prelude::FromWorld;
use rand::Rng;
use crate::physics::Position;

pub struct PathfinderGlobalData {
    points: [Vec<Position>; 2],
    krill_exit_points: [Position; 3],
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
    pub fn is_krill_exit(&self, position: &Position) -> bool {
        self.krill_exit_points.contains(position)
    }

    /// DEBUG PURPOSES
    pub fn contains_point(&self, position: Position) -> Option<char> {
        if self.points.iter().find(|v| {
            for p in v.iter() {
                if *p == position {
                    return true;
                }
            }
            false
        }).is_some() {
            Some('0')
        } else if self.krill_exit_points.contains(&position) {
            Some('e')
        } else {
            None
        }
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
                Position::new(80, 60),
            ],
        ];
        let krill_exit_points = [
            Position::new(111, 42),
            Position::new(68, 10),
            Position::new(150, 10),
        ];
        PathfinderGlobalData {
            points,
            krill_exit_points,
        }
    }
}
