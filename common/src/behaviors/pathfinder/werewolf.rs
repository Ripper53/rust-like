use bevy::prelude::Query;
use crate::{physics::{Map, MapCache, Position}, character::{CharacterType, CharacterData, WereForm}, map_brain::{CharacterBehaviorData, WerewolfState}};
use super::{PathfinderBehavior, util::get_pathfinder_target, data::PathfinderGlobalData};

pub fn werewolf_pathfinder(
    data: &PathfinderGlobalData,
    mut behavior: &mut PathfinderBehavior,
    map: &Map,
    map_cache: &mut MapCache,
    character_type: &CharacterType,
    character_data: &CharacterData,
    character_behavior_data: &mut CharacterBehaviorData,
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
                if let CharacterBehaviorData::Werewolf { state } = character_behavior_data {
                    match state {
                        WerewolfState::Human(state) => {},
                        WerewolfState::Hunt(target) => {
                            if let Some(target) = target {

                            } else {

                            }
                        },
                        WerewolfState::Panic => {
                            behavior.set_goal(Position::new(0, 0)).reach_goal(|params| {
                                if let CharacterBehaviorData::Werewolf { state } = params.character_behavior_data {
                                    
                                }
                            });
                        },
                    }
                }

            },
        }
    }
}
