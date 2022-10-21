use std::cmp::Ordering;
use bevy::prelude::Query;
use rand::Rng;
use crate::{physics::{Map, MapCache, Position}, character::CharacterType};
use super::PathfinderBehavior;

pub fn get_pathfinder_target(
    pathfinder: &mut PathfinderBehavior,
    map: &Map,
    map_cache: &mut MapCache,
    position: &Position,
    search_query: &Query<(&CharacterType, &Position)>,
    target_character_type: CharacterType,
) -> bool {
    let mut found_target = false;
    let in_vision = map.get_in_vision(map_cache, position.clone());
    let mut check_found_target = |pos: &Position, character_type: &CharacterType| {
        found_target = in_vision.contains(pos) && matches!(character_type, CharacterType::Werewolf);
    };
    if let Some((_, target)) = search_query.iter().min_by(|(type_a, pos_a), (type_b, pos_b)| {
        if **type_a == target_character_type {
            check_found_target(pos_a, type_a);
            if **type_b == target_character_type {
                let diff_a = position.distance(pos_a);
                let diff_b = position.distance(pos_b);
                diff_a.cmp(&diff_b)
            } else {
                Ordering::Less
            }
        } else if **type_b == target_character_type {
            check_found_target(pos_b, type_b);
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }) {
        if found_target {
            pathfinder.set_goal(target.clone(), super::Priority::Low);
            true
        } else {
            pathfinder.set_goal(position.clone(), super::Priority::Low);
            false
        }
    } else {
        false
    }
}

pub fn get_random_target(map: &Map, pathfinder: &mut PathfinderBehavior) {
    let x = rand::thread_rng().gen_range(0..map.get_size_x() as i32);
    let y = rand::thread_rng().gen_range(0..map.get_size_y() as i32);
    pathfinder.set_goal(Position::new(x, y), super::Priority::Low);
}
