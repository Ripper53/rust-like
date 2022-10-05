use bevy::{
    prelude::Commands,
    ecs::system::EntityCommands,
};
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
        CharacterData, LootableTag,
    },
    behaviors::{
        pathfinder::{
            PathfinderBehavior,
            rumdare::rumdare_pathfinder,
            lerain::lerain_pathfinder,
            werewolf::werewolf_pathfinder,
        },
        werewolf::WerewolfBehavior,
    },
    constants::HUMAN_CHARACTER, map_brain::CharacterBehaviorData, inventory::Inventory,
};

fn spawn_character(
    commands: &mut Commands,
    map: &mut Map,
    sprite: Sprite,
    position: Position,
    health: Health,
    character_type: CharacterType,
    character_data: CharacterData,
    spawned_callback: fn(EntityCommands),
) {
    map.spawn_character(
        commands,
        sprite,
        position,
        health,
        character_type,
        character_data,
        spawned_callback,
    );
}

pub fn spawn_lerain(commands: &mut Commands, map: &mut Map, position: Position) {
    spawn_character(
        commands,
        map,
        Sprite::new(HUMAN_CHARACTER),
        position,
        Health::new(1),
        CharacterType::Lerain,
        CharacterData::Human,
        |mut entity_commands| {
            entity_commands
                .insert(CharacterBehaviorData::default_human())
                .insert(PathfinderBehavior::new(1, lerain_pathfinder));
        },
    )
}

pub fn spawn_rumdare(commands: &mut Commands, map: &mut Map, position: Position) {
    spawn_character(
        commands,
        map,
        Sprite::new(HUMAN_CHARACTER),
        position,
        Health::new(1),
        CharacterType::Rumdare,
        CharacterData::Human,
        |mut entity_commands| {
            entity_commands
                .insert(CharacterBehaviorData::default_human())
                .insert(PathfinderBehavior::new(1, rumdare_pathfinder));
        },
    );
}

pub fn spawn_werewolf(commands: &mut Commands, map: &mut Map, position: Position) {
    spawn_character(
        commands,
        map,
        Sprite::new(HUMAN_CHARACTER),
        position,
        Health::new(1),
        CharacterType::Werewolf,
        CharacterData::Werewolf { form: crate::character::WereForm::Human },
        |mut entity_commands| {
            entity_commands
                .insert(PathfinderBehavior::new(4, werewolf_pathfinder))
                .insert(WerewolfBehavior::new());
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

pub fn spawn_chest(
    commands: &mut Commands,
    map: &mut Map,
    position: Position,
    inventory: Inventory,
) {
    map.spawn(
        commands,
        Sprite::new('M'),
        position,
        Velocity::default(),
        CollisionType::Solid,
        |mut entity_commands| {
            entity_commands
                .insert(LootableTag)
                .insert(inventory);
        },
    );
}
