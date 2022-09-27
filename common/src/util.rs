use std::cmp::Ordering;

use bevy::{
    prelude::{Commands, Query},
    ecs::system::EntityCommands,
};
use rand::Rng;
use crate::{
    physics::{
        Map,
        Position,
        Velocity,
        Collision,
        CollisionType,
        KrillTheaterZone,
    },
    character::{
        Sprite,
        CharacterType,
        Health,
        MovementInput,
        Interact,
        CharacterData, WereForm,
    },
    behaviors::{
        pathfinder::{PathfinderBehavior, Pathfinder},
        werewolf::WerewolfBehavior,
    },
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
        Sprite::new('C'),
        position,
        Health::new(1),
        CharacterType::Lerain,
        CharacterData::Human,
        |mut entity_commands| {
            entity_commands
                .insert(PathfinderBehavior::new(1, get_pathfinder_target));
        },
    )
}

pub fn spawn_rumdare(commands: &mut Commands, map: &mut Map, position: Position) {
    spawn_character(
        commands,
        map,
        Sprite::new('C'),
        position,
        Health::new(1),
        CharacterType::Rumdare,
        CharacterData::Human,
        |mut entity_commands| {
            entity_commands
                .insert(PathfinderBehavior::new(1, get_pathfinder_target));
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
        CharacterData::Werewolf { form: crate::character::WereForm::Human },
        |mut entity_commands| {
            entity_commands
                .insert(PathfinderBehavior::new(
                    4,
                    |behavior, map, self_character_type, self_character_data, self_position, query| {
                        if let CharacterData::Werewolf { form } = self_character_data {
                            behavior.pathfinder.current_goal = Position::new(0, 0);
                            match form {
                                WereForm::Human => {
                                    get_pathfinder_target(
                                        behavior,
                                        map,
                                        self_character_type,
                                        self_character_data,
                                        self_position,
                                        query,
                                    );
                                },
                                WereForm::Beast => {
                                    for (character_type, character_data, position) in query.iter() {
                                        
                                    }
                                },
                            }
                        }
                    },
                ))
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

fn get_pathfinder_target(
    pathfinder: &mut PathfinderBehavior,
    map: &Map,
    character_type: &CharacterType,
    character_data: &CharacterData,
    position: &Position,
    search_query: &Query<(&CharacterType, &CharacterData, &Position)>,
) {
    let get_target = |pathfinder: &mut Pathfinder, target_character_type: CharacterType| {
        let mut found_target = false;
        let in_vision = map.get_in_vision(position.clone());
        if let Some((_, target_data, target)) = search_query.iter().min_by(|(type_a, data_a, pos_a), (type_b, data_b, pos_b)| {
            if **type_a == target_character_type {
                found_target = in_vision.contains(pos_a) && if let CharacterData::Werewolf { form } = data_a {
                    matches!(form, WereForm::Beast)
                } else {
                    true
                };
                if **type_b == target_character_type {
                    let diff_a = position.distance(pos_a);
                    let diff_b = position.distance(pos_b);
                    diff_a.cmp(&diff_b)
                } else {
                    Ordering::Less
                }
            } else if **type_b == target_character_type {
                found_target = true;
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        }) {
            if found_target {
                pathfinder.current_goal = *target;
                true
            } else {
                pathfinder.current_goal = *position;
                false
            }
        } else {
            false
        }
    };
    let get_random_target = |pathfinder: &mut Pathfinder| {
        let x = rand::thread_rng().gen_range(0..map.get_size_x() as i32);
        let y = rand::thread_rng().gen_range(0..map.get_size_y() as i32);
        pathfinder.current_goal = Position::new(x, y);
    };
    match character_type {
        CharacterType::Player => {},
        CharacterType::Lerain => {
            if !get_target(&mut pathfinder.pathfinder, CharacterType::Werewolf) && pathfinder.pathfinder.current_goal == *position {
                if let Some(tile) = map.get(position.x as usize, position.y as usize) {
                    if let Some(krill_theater) = tile.krill_theater() {
                        match krill_theater {
                            KrillTheaterZone::Free => {
                                get_random_target(&mut pathfinder.pathfinder);
                            },
                            KrillTheaterZone::LineUp(target) => {
                                pathfinder.pathfinder.current_goal = *target;
                            },
                        }
                    } else {
                        get_random_target(&mut pathfinder.pathfinder);
                    }
                }
            }
        },
        CharacterType::Rumdare => { get_target(&mut pathfinder.pathfinder, CharacterType::Werewolf); },
        CharacterType::Werewolf => { get_target(&mut pathfinder.pathfinder, CharacterType::Player); },
    }
}
