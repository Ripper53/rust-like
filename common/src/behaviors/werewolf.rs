use bevy::prelude::{Query, Res, ResMut};
use crate::{map_brain::BehaviorData, character::{CharacterData, Sprite, WereForm}, physics::{Map, Position, Tile, MapCache}};

pub struct WerewolfBehavior {

}
impl WerewolfBehavior {
    pub fn new() -> BehaviorData<WerewolfBehavior> {
        BehaviorData::new(WerewolfBehavior { })
    }
}

pub fn werewolf_update(
    map: Res<Map>,
    mut map_cache: ResMut<MapCache>,
    mut query: Query<(&mut CharacterData, &mut Sprite, &Position)>,
) {
    for (mut character_data, mut sprite, position) in query.iter_mut() {
        if let CharacterData::Werewolf { form } = character_data.as_mut() {
            let in_vision = map.get_in_vision(&mut map_cache, position.clone());
            let mut character_count: u32 = 0;
            for p in in_vision {
                if let Some(Tile::Ground { occupier, .. }) = map.get(p.x as usize, p.y as usize) {
                    if occupier.is_some() {
                        character_count += 1;
                    }
                }
            }
            *form = if character_count == 2 {
                WereForm::Beast
            } else {
                WereForm::Human
            };

            // Change sprite!
            match form {
                WereForm::Human => *sprite = Sprite::new('C'),
                WereForm::Beast => *sprite = Sprite::new('W'),
            }
        }
    }
}
