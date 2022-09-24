use bevy::prelude::Query;
use rand::Rng;
use crate::{map_brain::BehaviorData, character::{CharacterData, Sprite, WereForm}};

pub struct WerewolfBehavior {

}
impl WerewolfBehavior {
    pub fn new() -> BehaviorData<WerewolfBehavior> {
        BehaviorData::new(WerewolfBehavior { })
    }
}

pub fn werewolf_update(mut query: Query<(&mut CharacterData, &mut Sprite)>) {
    for (mut character_data, mut sprite) in query.iter_mut() {
        if let CharacterData::Werewolf { form } = character_data.as_mut() {
            *form = if rand::thread_rng().gen_range(0..100 as i32) < 50 {
                WereForm::Beast
            } else {
                WereForm::Human
            };
            match form {
                WereForm::Human => *sprite = Sprite::new('C'),
                WereForm::Beast => *sprite = Sprite::new('W'),
            }
        }
    }
}
