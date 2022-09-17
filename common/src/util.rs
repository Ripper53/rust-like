use bevy::prelude::Commands;
use crate::{physics::{Map, Position, Velocity}, character::{Sprite, CharacterType, Health}, map_brain::Behavior};

pub fn spawn_lerain(commands: &mut Commands, map: &mut Map, position: Position) {
    map.spawn_character(
        commands,
        Sprite::new('L'),
        position,
        Velocity::new(0, 0),
        Health::new(1),
        CharacterType::Lerain,
        |mut entity_commands| {
            entity_commands.insert(crate::map_brain::Brain::new(vec![
                Behavior::default_slow_movement(),
            ]));
        },
    );
}

pub fn spawn_rumdare(commands: &mut Commands, map: &mut Map, position: Position) {
    map.spawn_character(
        commands,
        Sprite::new('R'),
        position,
        Velocity::new(0, 0),
        Health::new(1),
        CharacterType::Rumdare,
        |mut entity_commands| {
            entity_commands.insert(crate::map_brain::Brain::new(vec![
                Behavior::default_slow_movement(),
            ]));
        },
    );
}

pub fn spawn_werewolf(commands: &mut Commands, map: &mut Map, position: Position) {
    map.spawn_character(
        commands,
        Sprite::new('W'),
        position,
        Velocity::new(0, 0),
        Health::new(1),
        CharacterType::Werewolf,
        |mut entity_commands| {
            entity_commands.insert(crate::map_brain::Brain::new(vec![
                Behavior::skip_movement(4),
            ]));
        },
    );
}
