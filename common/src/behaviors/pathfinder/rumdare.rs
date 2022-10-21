use bevy::prelude::Query;
use crate::{physics::{Map, MapCache, Position}, character::{CharacterType, CharacterData}, map_brain::CharacterBehaviorData};
use super::{PathfinderBehavior, util::{get_random_target, get_pathfinder_target}, data::PathfinderGlobalData};

pub fn rumdare_pathfinder(
    data: &PathfinderGlobalData,
    mut behavior: &mut PathfinderBehavior,
    map: &Map,
    map_cache: &mut MapCache,
    character_type: &CharacterType,
    character_data: &mut CharacterData,
    character_behavior_data: &mut CharacterBehaviorData,
    position: &Position,
    query: &Query<(&CharacterType, &Position)>,
) {
    if !get_pathfinder_target(
        &mut behavior,
        map,
        map_cache,
        &position,
        query,
        CharacterType::Werewolf,
    ) {
        get_random_target(map, &mut behavior);
    }
}
