use bevy::{prelude::Commands, ecs::system::EntityCommands};
use crate::{
    physics::{
        Map,
        Position,
        Velocity,
        Collision,
        CollisionType,
    },
    character::{
        Sprite,
        CharacterType,
        Health,
        MovementInput,
        Interact,
    },
    map_brain::Behavior,
};

fn spawn_character(
    commands: &mut Commands,
    map: &mut Map,
    sprite: Sprite,
    position: Position,
    health: Health,
    character_type: CharacterType,
    spawned_callback: fn(EntityCommands),
) {
    map.spawn_character(
        commands,
        sprite,
        position,
        health,
        character_type,
        spawned_callback,
    );
}

pub fn spawn_lerain(commands: &mut Commands, map: &mut Map, position: Position) {
    spawn_character(
        commands,
        map,
        Sprite::new('L'),
        position,
        Health::new(1),
        CharacterType::Lerain,
        |mut entity_commands| {
            entity_commands.insert(crate::map_brain::Brain::new(vec![
                Behavior::default_slow_movement(),
            ]));
        },
    )
}

pub fn spawn_rumdare(commands: &mut Commands, map: &mut Map, position: Position) {
    spawn_character(
        commands,
        map,
        Sprite::new('R'),
        position,
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
    spawn_character(
        commands,
        map,
        Sprite::new('W'),
        position,
        Health::new(1),
        CharacterType::Werewolf,
        |mut entity_commands| {
            entity_commands.insert(crate::map_brain::Brain::new(vec![
                Behavior::skip_movement(4),
            ]));
        },
    );
}

pub fn spawn_projectile(
    commands: &mut Commands,
    map: &mut Map,
    sprite: Sprite,
    position: Position,
    velocity: Velocity,
    damage: i32,
) {
    map.spawn(
        commands,
        sprite,
        position,
        velocity,
        CollisionType::Sensor,
        |mut entity_commands| {
            entity_commands
                .insert(MovementInput::Idle)
                .insert(Interact::new(crate::character::InteractData::Projectile {
                    recent_spawn: true,
                    damage,
                }))
                .insert(Collision::new(CollisionType::Sensor));
        },
    );
}
