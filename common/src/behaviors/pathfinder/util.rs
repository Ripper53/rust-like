use std::cmp::Ordering;
use bevy::prelude::Query;
use rand::Rng;
use crate::{physics::{Map, MapCache, Position}, character::{CharacterType, CharacterData, WereForm}};
use super::{PathfinderBehavior};

pub fn get_pathfinder_target(
    pathfinder: &mut PathfinderBehavior,
    map: &Map,
    map_cache: &mut MapCache,
    character_type: &CharacterType,
    character_data: &CharacterData,
    position: &Position,
    search_query: &Query<(&CharacterType, &CharacterData, &Position)>,
    target_character_type: CharacterType,
) -> bool {
    let mut found_target = false;
    let in_vision = map.get_in_vision(map_cache, position.clone());
    let mut check_found_target = |pos: &Position, data: &CharacterData| {
        found_target = in_vision.contains(pos) && if let CharacterData::Werewolf { form } = data {
            matches!(form, WereForm::Beast)
        } else {
            true
        };
    };
    if let Some((_, target_data, target)) = search_query.iter().min_by(|(type_a, data_a, pos_a), (type_b, data_b, pos_b)| {
        if **type_a == target_character_type {
            check_found_target(pos_a, data_a);
            if **type_b == target_character_type {
                let diff_a = position.distance(pos_a);
                let diff_b = position.distance(pos_b);
                diff_a.cmp(&diff_b)
            } else {
                Ordering::Less
            }
        } else if **type_b == target_character_type {
            check_found_target(pos_b, data_b);
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }) {
        if found_target {
            pathfinder.set_goal(target.clone());
            true
        } else {
            pathfinder.set_goal(position.clone());
            false
        }
    } else {
        false
    }
}

pub fn get_random_target(map: &Map, pathfinder: &mut PathfinderBehavior) {
    let x = rand::thread_rng().gen_range(0..map.get_size_x() as i32);
    let y = rand::thread_rng().gen_range(0..map.get_size_y() as i32);
    pathfinder.set_goal(Position::new(x, y));
}
