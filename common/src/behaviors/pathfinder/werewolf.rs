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

                            } else {
                                behavior.set_goal(position.clone(), super::Priority::High);
                            }
                        },
                        WerewolfState::Panic => {
                            /*behavior.set_goal(Position::new(0, 0), super::Priority::Low).reach_goal_then(|params| {
                                if let CharacterBehaviorData::Werewolf { state } = params.character_behavior_data {
                                    
                                }
                            });*/
                        },
                    }
                }

            },
        }
    }
}
