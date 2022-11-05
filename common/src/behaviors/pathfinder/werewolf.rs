use bevy::prelude::Query;
use crate::{physics::{Map, MapCache, Position}, character::{CharacterType, CharacterData, WereForm}, map_brain::{CharacterBehaviorData, WerewolfState, HumanState}, util::Cooldown};
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
            WereForm::Human => {
                if let CharacterBehaviorData::Werewolf { human_state, .. } = character_behavior_data {
                    human_pathfinder(
                        human_state,
                        data,
                        behavior,
                        map,
                        map_cache,
                        position,
                        query,
                        CharacterType::Player,
                    );
                }
            },
            WereForm::Beast => {
                if let CharacterBehaviorData::Werewolf { werewolf_state, .. } = character_behavior_data {
                    match werewolf_state {
                        WerewolfState::Hunt(target) => {
                            if let Some(target) = target {
                                behavior.set_goal(target.clone(), super::Priority::Medium);
                            } else {
                                behavior.set_goal(position.clone(), super::Priority::Medium);
                            }
                        },
                        WerewolfState::Panic { target, exclude_target_index, calm_cooldown } => {
                            let target = if let Some(target) = target {
                                if calm_cooldown.execute() {
                                    Some(target.clone())
                                } else {
                                    *form = WereForm::Human;
                                    None
                                }
                            } else {
                                let target = if let Some(except) = exclude_target_index {
                                    data.get_hiding_target_except(character_type.clone(), *except)
                                } else {
                                    data.get_hiding_target(character_type.clone())
                                };
                                let position = target.0.clone();
                                *werewolf_state = WerewolfState::Panic {
                                    target: Some(target.0),
                                    exclude_target_index: Some(target.1),
                                    calm_cooldown: Cooldown(position.distance(&target.0) as usize),
                                };
                                Some(position)
                            };
                            if let Some(target) = target {
                                behavior.set_goal(target, super::Priority::Medium);
                            }
                        },
                    }
                }
            },
        }
    }
}
