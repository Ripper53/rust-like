use bevy::prelude::Query;
use rand::Rng;
use crate::{
    physics::{Map, MapCache, Position, KrillTheaterZone},
    character::{CharacterType, CharacterData},
    map_brain::{CharacterBehaviorData, HumanState, NewObjective},
};
use super::{PathfinderBehavior, util::get_pathfinder_target, data::PathfinderGlobalData};

fn set_goal<'a>(state: &'a mut HumanState, behavior: &'a mut PathfinderBehavior, goal: (Position, usize)) {
    *state = HumanState::Moving(goal.1);
    behavior.set_goal(goal.0).reach_goal(|params| {
        if let CharacterBehaviorData::Human { state } = params.character_behavior_data {
            if let HumanState::Moving(index) = state {
                *state = HumanState::Idle(Some(NewObjective::WanderButExclude(*index)));
            }
        }
    });
}
fn force_goal<'a>(state: &'a mut HumanState, behavior: &'a mut PathfinderBehavior, goal: (Position, usize)) -> &mut PathfinderBehavior {
    *state = HumanState::Moving(goal.1);
    behavior.force_goal(goal.0)
}

pub fn lerain_pathfinder(
    data: &PathfinderGlobalData,
    behavior: &mut PathfinderBehavior,
    map: &Map,
    map_cache: &mut MapCache,
    character_type: &CharacterType,
    character_data: &CharacterData,
    character_behavior_data: &mut CharacterBehaviorData,
    position: &Position,
    query: &Query<(&CharacterType, &CharacterData, &Position)>,
) {
    if let CharacterBehaviorData::Human { state } = character_behavior_data {
        match state {
            HumanState::Idle(objective) => {
                if !get_pathfinder_target(
                    behavior,
                    map,
                    map_cache,
                    character_type,
                    character_data,
                    position,
                    query,
                    CharacterType::Werewolf,
                ) && behavior.is_at(position.clone()) {
                    if let Some(tile) = map.get(position.x as usize, position.y as usize) {
                        if let Some(krill_theater) = tile.krill_theater() {
                            match krill_theater {
                                KrillTheaterZone::Free => {
                                    set_goal(state, behavior, data.get_target(super::data::CharacterType::Lerain));
                                },
                                KrillTheaterZone::LineUp(target) => {
                                    if let Some((target, index)) = data.is_exit_point(*position) {
                                        force_goal(state, behavior, (target, index)).reach_goal(
                                            |params| {
                                                if let CharacterBehaviorData::Human { state } = params.character_behavior_data {
                                                    if let HumanState::Moving(index) = state {
                                                        *state = HumanState::Idle(Some(NewObjective::WanderButExclude(*index)));
                                                    }
                                                }
                                            }
                                        );
                                    } else {
                                        behavior.set_goal(*target);
                                    }
                                },
                            }
                        } else if let Some(o) = objective {
                            match o {
                                NewObjective::WanderButExclude(index) => {
                                    let goal = data.get_target_except(super::data::CharacterType::Lerain, *index);
                                    set_goal(state, behavior, goal);
                                },
                            }
                        } else {
                            set_goal(state, behavior, data.get_target(super::data::CharacterType::Lerain));
                        }
                    }
                }
            },
            HumanState::Moving(_index) => {},
            HumanState::Panic => {},
        }
    }
}
