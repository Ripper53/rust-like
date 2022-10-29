use bevy::prelude::Query;
use crate::{physics::{Map, MapCache, Position}, character::{CharacterType, CharacterData, WereForm}, map_brain::{CharacterBehaviorData, WerewolfState, HumanState}};
use super::{PathfinderBehavior, data::PathfinderGlobalData, lerain::human_pathfinder};

pub fn werewolf_pathfinder(
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
    if let CharacterData::Werewolf { form } = character_data {
        match form {
            WereForm::Human(state) => {
                human_pathfinder(
                    state,
                    data,
                    behavior,
                    map,
                    map_cache,
                    position,
                    query,
                    CharacterType::Player,
                );
            },
            WereForm::Beast => {
                if let CharacterBehaviorData::Werewolf { state } = character_behavior_data {
                    match state {
                        WerewolfState::Hunt(target) => {
                            if let Some(target) = target {
                                behavior.set_goal(target.clone(), super::Priority::Medium);
                            } else {
                                behavior.set_goal(position.clone(), super::Priority::Medium);
                            }
                        },
                        WerewolfState::Panic(target) => {
                            let target = if let Some(target) = target.0 {
                                target.clone()
                            } else {
                                let target = if let Some(except) = target.1 {
                                    data.get_hiding_target_except(character_type.clone(), except)
                                } else {
                                    data.get_hiding_target(character_type.clone())
                                };
                                let position = target.0.clone();
                                *state = WerewolfState::Panic((Some(target.0), Some(target.1)));
                                position
                            };
                            behavior.set_goal(target, super::Priority::Medium);
                        },
                    }
                }
            },
        }
    }
}
