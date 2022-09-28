use bevy::prelude::{Query, ResMut};
use crate::{
    map_brain::BehaviorData,
    character::{CharacterData, Sprite, WereForm},
    physics::{Map, Position, Tile, MapCache},
    constants::{WEREWOLF_SKIP_AT, HUMAN_SKIP_AT, WEREWOLF_CHARACTER, HUMAN_CHARACTER},
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
    mut query: Query<(&mut CharacterData, &mut Sprite, &Position, &mut BehaviorData<PathfinderBehavior>)>,
) {
    for (mut character_data, mut sprite, position, mut pathfinder) in query.iter_mut() {
        if let CharacterData::Werewolf { form } = character_data.as_mut() {
            let in_vision = map.get_in_vision(&mut map_cache, position.clone());
            let mut character_count: u32 = 0;
            const BEAST_FORM_COUNT: u32 = 2;
            for p in in_vision {
                if let Some(Tile::Ground { occupier, .. }) = map.get(p.x as usize, p.y as usize) {
                    if occupier.is_some() {
                        character_count += 1;
                        if character_count > BEAST_FORM_COUNT {
                            break;
                        }
                    }
                }
            }

            let new_form = if character_count == BEAST_FORM_COUNT {
                WereForm::Beast
            } else {
                WereForm::Human
            };

            if new_form != *form {
                *form = new_form;
                match form {
                    WereForm::Human => {
                        sprite.set_character(HUMAN_CHARACTER, &mut map, position);
                        pathfinder.behavior.set_skip_turn(HUMAN_SKIP_AT);
                    },
                    WereForm::Beast => {
                        sprite.set_character(WEREWOLF_CHARACTER, &mut map, position);
                        pathfinder.behavior.set_skip_turn(WEREWOLF_SKIP_AT);
                    },
                }
            }
        }
    }
}
