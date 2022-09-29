use bevy::prelude::Query;
use rand::Rng;
use crate::{physics::{Map, MapCache, Position, KrillTheaterZone}, character::{CharacterType, CharacterData, HumanState}};
use super::{PathfinderBehavior, util::get_pathfinder_target};

pub fn lerain_pathfinder(
    mut behavior: &mut PathfinderBehavior,
    map: &Map,
    map_cache: &mut MapCache,
    character_type: &CharacterType,
    character_data: &CharacterData,
    position: &Position,
    query: &Query<(&CharacterType, &CharacterData, &Position)>,
) {
    if let CharacterData::Human { state } = character_data {
        if !get_pathfinder_target(
            &mut behavior,
            map,
            map_cache,
            character_type,
            character_data,
            &position,
            query,
            CharacterType::Werewolf,
        ) && behavior.is_at(position.clone()) {
            if let Some(tile) = map.get(position.x as usize, position.y as usize) {
                if let Some(krill_theater) = tile.krill_theater() {
                    match krill_theater {
                        KrillTheaterZone::Free => {
                            behavior.set_goal(get_target());
                        },
                        KrillTheaterZone::LineUp(target) => {
                            if let Some(target) = EXIT_POSITION.iter().find(|p| p.0 == *position) {
                                behavior.set_goal(target.1).reach_goal(
                                    |params| {
                                        if let CharacterData::Human { state } = params.character_data {
                                            //*state = HumanState::Left;
                                        }
                                    }
                                );
                            } else {
                                behavior.set_goal(*target);
                            }
                        },
                    }
                } else {
                    behavior.set_goal(get_target());
                }
            }
        }
    }
}

pub const POINTS: [Position; 3] = [
    Position::new(110, 42),
    Position::new(69, 10),
    Position::new(151, 10),
];

const EXIT_POSITION: [(Position, Position); 3] = [
    (POINTS[0], Position::new(110, 45)),
    (POINTS[1], Position::new(69, 7)),
    (POINTS[2], Position::new(151, 7)),
];

fn get_target() -> Position {
    let i = rand::thread_rng().gen_range(0..POINTS.len());
    POINTS[i]
}
