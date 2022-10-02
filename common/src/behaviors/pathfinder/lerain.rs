use bevy::prelude::Query;
use rand::Rng;
use crate::{
    physics::{Map, MapCache, Position, KrillTheaterZone},
    character::{CharacterType, CharacterData},
    map_brain::{CharacterBehaviorData, HumanState, NewObjective},
};
use super::{PathfinderBehavior, util::get_pathfinder_target};

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
                                    set_goal(state, behavior, get_target());
                                },
                                KrillTheaterZone::LineUp(target) => {
                                    if let Some(index) = EXIT_POSITION.iter().position(|p| p.0 == *position) {
                                        let target = EXIT_POSITION[index];
                                        force_goal(state, behavior, (target.1, index)).reach_goal(
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
                                    let goal = get_target_except(*index);
                                    set_goal(state, behavior, goal);
                                },
                            }
                        } else {
                            set_goal(state, behavior, get_target());
                        }
                    }
                }
            },
            HumanState::Moving(_index) => {},
            HumanState::Panic => {},
        }
    }
}

pub const POINTS: [Position; 3] = [
    Position::new(110, 42),
    Position::new(69, 10),
    Position::new(151, 10),
];

const EXIT_POSITION: [(Position, Position); 3] = [
    (POINTS[0], Position::new(110, 45)),
    (POINTS[1], Position::new(69, 7)),
    (POINTS[2], Position::new(151, 7)),
];

fn get_target() -> (Position, usize) {
    let i = rand::thread_rng().gen_range(0..POINTS.len());
    (POINTS[i], i)
}

fn get_target_except(exclude_index: usize) -> (Position, usize) {
    const LENGTH: usize = POINTS.len();
    let mut i = rand::thread_rng().gen_range(0..LENGTH);
    if i == exclude_index {
        if i == LENGTH - 1 {
            i = 0;
        } else {
            i += 1;
        }
    }
    (POINTS[i], i)
}
