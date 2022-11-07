use bevy::prelude::FromWorld;
use rand::Rng;
use crate::{physics::Position, character::CharacterType};

// TODO: MAKE OBJECTIVES CLOSER TO
// NPC MORE LIKELY TO OCCUR!
pub struct PathfinderGlobalData {
    points: Vec<Vec<Position>>,
    krill_exit_points: [Position; 3],
    pub human: HumanPathfinderGlobalData,
    pub werewolf: WerewolfPathfinderGlobalData,
}

fn get_fn<
    'a,
    I,
    F0: FnOnce(&I) -> (usize, usize),
    F1: FnOnce(&I, (usize, usize)) -> Position,
>(
    points: I,
    character_type: CharacterType,
    get_indexes: F0,
    get_points: F1,
) -> (Position, usize) {
    match character_type {
        CharacterType::Lerain |
        CharacterType::Rumdare |
        CharacterType::Werewolf |
        CharacterType::Player => {
            let indexes = get_indexes(&points);
            (get_points(&points, indexes), indexes.0)
        },
    }
}

pub struct GetPoint<'a> {
    character_type: CharacterType,
    points: Vec<&'a Vec<Position>>,
}
impl<'a> GetPoint<'a> {
    pub fn get(self) -> (Position, usize) {
        get_fn(
            self.points,
            self.character_type,
            move |points| {
                let i0 = rand::thread_rng().gen_range(0..points.len());
                let i0 = 0; // TO REMOVE
                let i1 = rand::thread_rng().gen_range(0..points[i0].len());
                (i0, i1)
            },
            move |points, indexes| points[indexes.0][indexes.1],
        )
    }
    pub fn get_except(self, exclude_index: usize) -> (Position, usize) {
        get_fn(
            self.points,
            self.character_type,
            move |points| {
                let length = points.len();
                let mut i0 = rand::thread_rng().gen_range(0..length);
                if i0 == exclude_index {
                    if i0 == length - 1 {
                        i0 = 0;
                    } else {
                        i0 += 1;
                    }
                }
                let i0 = 0; // TO REMOVE
                let i1 = rand::thread_rng().gen_range(0..points[i0].len());
                (i0, i1)
            },
            move |points, indexes| points[indexes.0][indexes.1],
        )
    }
}

pub struct GetPanicPoint<'a, const T: usize> {
    points: &'a [Vec<Position>; T],
    friendly: (CharacterType, Position),
}
impl<'a, const T: usize> GetPanicPoint<'a, T> {
    pub fn enemy(self, enemy_position: Position) -> GetPanicPointWithEnemy<'a, T> {
        GetPanicPointWithEnemy {
            panic_point: GetPanicPoint {
                points: self.points,
                friendly: self.friendly,
            },
            enemy_position,
        }
    }
    pub fn get(self) -> (Position, usize) {
        let character_type = self.friendly.0.clone();
        GetPoint {
            points: self.get_points(),
            character_type,
        }.get()
    }
    pub fn get_except(self, exclude_index: usize) -> (Position, usize) {
        let character_type = self.friendly.0.clone();
        GetPoint {
            points: self.get_points(),
            character_type,
        }.get_except(exclude_index)
    }
    fn get_points(self) -> Vec<&'a Vec<Position>> {
        self.points.iter()
            .min_by(|v1, v2| {
                let point1 = v1[0];
                let point2 = v2[0];
                let dis1 = point1.distance(&self.friendly.1);
                let dis2 = point2.distance(&self.friendly.1);
                dis1.cmp(&dis2)
            })
            .into_iter()
            .collect()
    }
}

pub struct GetPanicPointWithEnemy<'a, const T: usize> {
    panic_point: GetPanicPoint<'a, T>,
    enemy_position: Position,
}
impl<'a, const T: usize> GetPanicPointWithEnemy<'a, T> {
    pub fn get(self) -> (Position, usize) {
        let character_type = self.panic_point.friendly.0.clone();
        GetPoint {
            points: self.get_points(),
            character_type,
        }.get()
    }
    pub fn get_except(self, exclude_index: usize) -> (Position, usize) {
        let character_type = self.panic_point.friendly.0.clone();
        GetPoint {
            points: self.get_points(),
            character_type,
        }.get_except(exclude_index)
    }
    fn get_points(self) -> Vec<&'a Vec<Position>> {
        self.panic_point.points.into_iter()
            .max_by(|v1, v2| {
                let point1 = v1[0];
                let point2 = v2[0];
                let dis1 = point1.distance(&self.enemy_position);
                let dis2 = point2.distance(&self.enemy_position);
                dis1.cmp(&dis2)
            })
            .into_iter()
            .collect()
    }
}

pub struct HumanPathfinderGlobalData {
    hiding_points: [Vec<Position>; 2],
}

impl HumanPathfinderGlobalData {
    pub fn panic(&self, friendly: (CharacterType, Position)) -> GetPanicPoint<2> {
        GetPanicPoint {
            points: &self.hiding_points,
            friendly,
        }
    }
}

pub struct WerewolfPathfinderGlobalData {
    hiding_points: [Vec<Position>; 2],
}

impl WerewolfPathfinderGlobalData {
    pub fn panic(&self, friendly: (CharacterType, Position)) -> GetPanicPoint<2> {
        GetPanicPoint {
            points: &self.hiding_points,
            friendly,
        }
    }
}

impl PathfinderGlobalData {
    pub fn target(&self, character_type: CharacterType) -> GetPoint {
        GetPoint {
            character_type,
            points: self.points.iter().collect(),
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
        let points = vec![
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
        let human = HumanPathfinderGlobalData {
            hiding_points: [
                vec![
                    Position::new(30, 30),
                ],
                vec![
                    Position::new(201, 73),
                ],
            ]
        };
        let werewolf = WerewolfPathfinderGlobalData {
            hiding_points: [
                vec![
                    Position::new(201, 73),
                ],
                vec![
                    Position::new(30, 30),
                ],
            ],
        };
        PathfinderGlobalData {
            points,
            krill_exit_points,
            human,
            werewolf,
        }
    }
}
