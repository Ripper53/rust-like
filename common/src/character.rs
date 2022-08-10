use bevy::prelude::*;
use crate::physics::*;

#[derive(Component)]
pub struct Sprite {
    pub character: char,
}
impl Sprite {
    pub fn new(character: char) -> Sprite {
        Sprite {
            character,
        }
    }
}

#[derive(Bundle)]
pub struct CharacterBundle {
    pub sprite: Sprite,
    pub position: Position,
    pub velocity: Velocity,
}
