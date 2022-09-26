use bevy::prelude::Query;
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
            match form {
                WereForm::Human => *sprite = Sprite::new('C'),
                WereForm::Beast => *sprite = Sprite::new('W'),
            }
        }
    }
}
