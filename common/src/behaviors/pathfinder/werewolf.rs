use bevy::prelude::Query;
use crate::{physics::{Map, MapCache, Position}, character::{CharacterType, CharacterData, WereForm}};
use super::{PathfinderBehavior, util::get_pathfinder_target};

pub fn werewolf_pathfinder(
    mut behavior: &mut PathfinderBehavior,
    map: &Map,
    map_cache: &mut MapCache,
    character_type: &CharacterType,
    character_data: &CharacterData,
    position: &Position,
    query: &Query<(&CharacterType, &CharacterData, &Position)>,
) {
    if let CharacterData::Werewolf { form } = character_data {
        match form {
            WereForm::Human => {
                get_pathfinder_target(
                    &mut behavior,
                    map,
                    map_cache,
                    character_type,
                    character_data,
                    position,
                    query,
                    CharacterType::Player,
                );
            },
            WereForm::Beast => {
                get_pathfinder_target(
                    &mut behavior,
                    map,
                    map_cache,
                    character_type,
                    character_data,
                    position,
                    query,
                    CharacterType::Player,
                );
                for (character_type, character_data, position) in query.iter() {
                    
                }
            },
        }
    }
}
