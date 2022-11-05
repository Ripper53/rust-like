use bevy::prelude::Query;
use crate::{
    physics::{Map, MapCache, Position, KrillTheaterZone, Tile, Occupier},
    character::{CharacterType, CharacterData, self},
    map_brain::{CharacterBehaviorData, HumanState, NewObjective},
};
use super::{PathfinderBehavior, util::get_pathfinder_target, data::PathfinderGlobalData, Priority};

fn set_goal(state: &mut HumanState, behavior: &mut PathfinderBehavior, goal: (Position, usize), priority: Priority) {
    *state = HumanState::Moving(goal.1);
    behavior.set_goal(goal.0, priority).reach_goal_then(|params| {
        if let CharacterBehaviorData::Human { human_state } | CharacterBehaviorData::Werewolf { human_state, .. } = params.character_behavior_data {
            if let HumanState::Moving(index) = human_state {
                *human_state = HumanState::Idle(Some(NewObjective::WanderButExclude(*index)));
            }
        }
    });
}

pub fn lerain_pathfinder(
    data: &PathfinderGlobalData,
    behavior: &mut PathfinderBehavior,
    map: &Map,
    map_cache: &mut MapCache,
    character_type: &CharacterType,
    character_data: &mut CharacterData,
    character_behavior_data: &mut CharacterBehaviorData,
    position: &Position,
    query: &Query<(&CharacterType, &Position)>,
) {
    if let CharacterBehaviorData::Human { human_state: state } = character_behavior_data {
        human_pathfinder(
            character_type,
            state,
            data,
            behavior,
            map,
            map_cache,
            position,
            query,
            CharacterType::Werewolf,
        );
    }
}

pub fn human_pathfinder(
    character_type: &CharacterType,
    state: &mut HumanState,
    data: &PathfinderGlobalData,
    behavior: &mut PathfinderBehavior,
    map: &Map,
    map_cache: &mut MapCache,
    position: &Position,
    query: &Query<(&CharacterType, &Position)>,
    target_character_type: CharacterType,
) {
    match state {
        HumanState::Idle(objective) => {
            if !get_pathfinder_target(
                behavior,
                map,
                map_cache,
                position,
                query,
                target_character_type,
            ) && behavior.is_at(position.clone()) {
                if let Some(tile) = map.get(position.x as usize, position.y as usize) {
                    if let Some(krill_theater) = tile.krill_theater() {
                        match krill_theater {
                            KrillTheaterZone::Free => {
                                set_goal(
                                    state,
                                    behavior,
                                    data.get_target(CharacterType::Lerain),
                                    Priority::Low,
                                );
                            },
                            KrillTheaterZone::LineUp(target) => {
                                behavior.set_goal(*target, Priority::Low);
                            },
                            KrillTheaterZone::Exit => {
                                if let Some(objective) = objective {
                                    let index: Option<usize> = if let NewObjective::WanderButExclude(i) = objective {
                                        Some(*i)
                                    } else {
                                        None
                                    };
                                    if let Some(index) = index {
                                        set_goal(
                                            state,
                                            behavior,
                                            data.get_target_except(CharacterType::Lerain, index),
                                            Priority::Medium,
                                        );
                                    }
                                } else {
                                    set_goal(
                                        state,
                                        behavior,
                                        data.get_target(CharacterType::Lerain),
                                        Priority::Medium,
                                    );
                                }
                            },
                        }
                    } else if let Some(o) = objective {
                        match o {
                            NewObjective::WanderButExclude(index) => {
                                let goal = data.get_target_except(CharacterType::Lerain, *index);
                                set_goal(state, behavior, goal, Priority::Low);
                            },
                        }
                    } else {
                        set_goal(
                            state,
                            behavior,
                            data.get_target(CharacterType::Lerain),
                            Priority::Low,
                        );
                    }
                }
            }
        },
        HumanState::Moving(index) => {
            match character_type {
                CharacterType::Player => {},
                CharacterType::Lerain | CharacterType::Rumdare => {
                    let vision = map.get_in_vision(map_cache, *position);
                    for p in vision.iter() {
                        if let Some(Tile::Ground { occupier, .. } | Tile::Obstacle { occupier }) = map.get(p.x as usize, p.y as usize) {
                            if let Some(Occupier { character_type: Some(CharacterType::Werewolf), .. }) = occupier {
                                /* TO SET PANIC GOAL */
                                *state = HumanState::Panic(*index);
                                break;
                            }
                        }
                    }
                },
                CharacterType::Werewolf => { /* TODO */},
            }
        },
        HumanState::Panic(index) => {
            // TODO, CREATE PANIC BEHAVIOR FOR HUMANS (or when werewolf is in its HUMAN FORM)!
            match character_type {
                CharacterType::Player => {},
                CharacterType::Lerain | CharacterType::Rumdare => {
                    /* TODO REWORK */
                    *state = HumanState::Idle(Some(NewObjective::WanderButExclude(*index)));
                },
                CharacterType::Werewolf => {},
            }
        },
    }
}
