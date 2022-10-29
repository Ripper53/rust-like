use bevy::prelude::{Query, ResMut};
use crate::{
    map_brain::{BehaviorData, CharacterBehaviorData, WerewolfState},
    character::{CharacterData, Sprite, WereForm, Health},
    physics::{Map, Position, MapCache, Tile},
    constants::{WEREWOLF_SKIP_AT, HUMAN_SKIP_AT},
};
use super::pathfinder::PathfinderBehavior;

pub struct WerewolfBehavior {

}
impl WerewolfBehavior {
    pub fn new() -> BehaviorData<WerewolfBehavior> {
        BehaviorData::new(WerewolfBehavior { })
    }
}

pub fn werewolf_update(
    mut map: ResMut<Map>,
    mut map_cache: ResMut<MapCache>,
    mut query: Query<(
        &mut CharacterData,
        &mut CharacterBehaviorData,
        &mut Sprite,
        &Position,
        &mut BehaviorData<PathfinderBehavior>,
    )>,
    mut health_query: Query<&mut Health>,
) {
    for (mut character_data, mut character_behavior_data, mut sprite, position, mut pathfinder) in query.iter_mut() {
        if let CharacterData::Werewolf { form } = character_data.as_mut() {
            if let CharacterBehaviorData::Werewolf { state } = character_behavior_data.as_mut() {
                // Attack
                for attack_offset in [Position::new(0, 1), Position::new(1, 0), Position::new(0, -1), Position::new(-1, 0)] {
                    let p = *position + attack_offset;
                    if let Some(Tile::Ground { occupier, .. }) = map.get(p.x as usize, p.y as usize) {
                        if let Some(occupier) = occupier {
                            if let Ok(mut health) = health_query.get_mut(occupier.entity) {
                                health.damage(1);
                            }
                        }
                    }
                }

                // Transition Forms
                let in_vision = map.get_in_vision(&mut map_cache, position.clone());
                let mut character_count = 0;
                const BEAST_FORM_COUNT: u32 = 1;
                let mut nearest_target: Option<Position> = None;
                for p in in_vision {
                    if p != position {
                        if let Some(tile) = map.get(p.x as usize, p.y as usize) {
                            if tile.is_character() {
                                if let Some(target) = nearest_target {
                                    if p.distance(position) < target.distance(position) {
                                        nearest_target = Some(p.clone());
                                    }
                                } else {
                                    nearest_target = Some(p.clone());
                                }
                                character_count += 1;
                                if character_count > BEAST_FORM_COUNT {
                                    break;
                                }
                            }
                        }
                    }
                }

                let set_form = |new_form: WereForm| {
                    match new_form {
                        WereForm::Human(_) => if !matches!(form, WereForm::Human(_)) {
                            return Some(new_form);
                        },
                        WereForm::Beast => if !matches!(form, WereForm::Beast) {
                            return Some(new_form);
                        },
                    }
                    None
                };
                let can_change_form = if let WerewolfState::Panic(target) = state {
                    if let Some(target) = target.0 {
                        *position == target
                    } else {
                        true
                    }
                } else {
                    true
                };
                if can_change_form {
                    let new_form = if character_count == BEAST_FORM_COUNT {
                        *state = if let Some(target) = nearest_target {
                            WerewolfState::Hunt(Some(target))
                        } else {
                            WerewolfState::Hunt(None)
                        };
                        set_form(WereForm::Beast)
                    } else if character_count > BEAST_FORM_COUNT {
                        *state = WerewolfState::Panic((None, None));
                        set_form(WereForm::Beast)
                    } else if check(state, position) {
                        set_form(WereForm::Human(crate::map_brain::HumanState::Idle(None)))
                    } else {
                        None
                    };
    
                    if let Some(new_form) = new_form {
                        *form = new_form;
                        match form {
                            WereForm::Human(_) => {
                                sprite.set_sprite(Sprite::Lerain, &mut map, position);
                                pathfinder.behavior.set_skip_turn(HUMAN_SKIP_AT);
                            },
                            WereForm::Beast => {
                                sprite.set_sprite(Sprite::Werewolf, &mut map, position);
                                pathfinder.behavior.set_skip_turn(WEREWOLF_SKIP_AT);
                            },
                        }
                    }
                }
            }
        }
    }
}

fn check(state: &WerewolfState, position: &Position) -> bool {
    if let WerewolfState::Hunt(Some(target)) = state {
        if position != target {
            return false;
        }
    }
    return true;
}
