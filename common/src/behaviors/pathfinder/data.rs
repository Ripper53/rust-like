use bevy::prelude::FromWorld;
use rand::Rng;
use crate::{physics::Position, character::CharacterType};

// TODO: MAKE OBJECTIVES CLOSER TO
// NPC MORE LIKELY TO OCCUR!
pub struct PathfinderGlobalData {
    points: [Vec<Position>; 2],
    hiding_points: [Vec<Position>; 2],
    krill_exit_points: [Position; 3],
}

impl PathfinderGlobalData {
    fn get_point<const T: usize>(
        character_type: CharacterType,
        points: &[Vec<Position>; T],
    ) -> (Position, usize) {
        match character_type {
            CharacterType::Lerain |
            CharacterType::Rumdare |
            CharacterType::Werewolf |
            CharacterType::Player => {
                let i0 = rand::thread_rng().gen_range(0..points.len());
                let i1 = rand::thread_rng().gen_range(0..points[i0].len());
                (points[i0][i1], i0)
            },
        }
    }
    fn get_point_except<const T: usize>(
        character_type: CharacterType,
        points: &[Vec<Position>; T],
        exclude_index: usize,
    ) -> (Position, usize) {
        match character_type {
            CharacterType::Lerain |
            CharacterType::Rumdare |
            CharacterType::Werewolf |
            CharacterType::Player => {
                let length = points.len();
                let mut i0 = rand::thread_rng().gen_range(0..length);
                if i0 == exclude_index {
                    if i0 == length - 1 {
                        i0 = 0;
                    } else {
                        i0 += 1;
                    }
                }
                let i1 = rand::thread_rng().gen_range(0..points[i0].len());
                (points[i0][i1], i0)
            },
        }
    }
    pub fn get_target(&self, character_type: CharacterType) -> (Position, usize) {
        Self::get_point(character_type, &self.points)
    }
    pub fn get_target_except(&self, character_type: CharacterType, exclude_index: usize) -> (Position, usize) {
        Self::get_point_except(character_type, &self.points, exclude_index)
    }
    pub fn get_hiding_target(&self, character_type: CharacterType) -> (Position, usize) {
        Self::get_point(character_type, &self.hiding_points)
    }
    pub fn get_hiding_target_except(&self, character_type: CharacterType, exclude_index: usize) -> (Position, usize) {
        Self::get_point_except(character_type, &self.hiding_points, exclude_index)
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
        let hiding_points = [
            vec![
                Position::new(30, 30),
            ],
            vec![
                Position::new(201, 73),
            ],
        ];
        let krill_exit_points = [
            Position::new(111, 42),
            Position::new(68, 10),
            Position::new(150, 10),
        ];
        PathfinderGlobalData {
            points,
            hiding_points,
            krill_exit_points,
        }
    }
}
