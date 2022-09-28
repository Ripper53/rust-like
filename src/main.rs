use bevy::prelude::*;
use client::render::*;
use common::{physics::*, character::*, dialogue::Dialogue, inventory::inventory_update, util::{spawn_lerain, spawn_werewolf}, ActionInput, Scene};
use iyes_loopless::{condition::IntoConditionalExclusiveSystem};

fn setup(mut commands: Commands, mut map: ResMut<Map>) {
    map.spawn_character(
        &mut commands,
        common::character::Sprite::new('@'),
        Position::new(50, 2),
        Health::new(1),
        CharacterType::Player,
        CharacterData::Human,
        |mut entity_commands| {
            entity_commands.insert(PlayerTag);
        },
    );
    spawn_lerain(&mut commands, &mut map, Position::new(2, 1));
    spawn_lerain(&mut commands, &mut map, Position::new(20, 40));
    spawn_lerain(&mut commands, &mut map, Position::new(30, 10));
    spawn_lerain(&mut commands, &mut map, Position::new(25, 20));
    spawn_werewolf(&mut commands, &mut map, Position::new(2, 4));
}

fn in_conversation_condition(dialogue: Res<Dialogue>) -> bool {
    dialogue.in_conversation
}

fn main() {

    const PLAYER_INPUT_LABEL: &str = "player_movement_input_update";
    const PLAYER_MOVEMENT_LABEL: &str = "player_movement_update";
    const NPC_BEHAVIOR_UPDATE_LABEL: &str = "npc_behavior_update";
    const NPC_MOVEMENT_UPDATE_LABEL: &str = "npc_movement_update";
    const COLLISION_UPDATE_LABEL: &str = "collision_update";
    const INTERACT_UPDATE_LABEL: &str = "interact_update";
    const DESTORY_CHECK_LABEL: &str = "destroy_check";

    const INVENTORY_LABEL: &str = "inventory_update";

    App::new()
        .set_runner(runner)
        .add_state(Scene::Map)
        .init_resource::<PlayerInput>()
        .insert_resource(ActionInput::None)
        .insert_resource(Dialogue::default())
        .init_resource::<Map>()
        .insert_resource(MapCache::default())
        .add_startup_system(setup)

        .add_system_set(SystemSet::on_update(Scene::Map)
            .with_system(
                inventory_update
                    .run_if_not(in_conversation_condition)
                    .label(INVENTORY_LABEL)
            )
            .with_system(
                player_movement_input_update
                    .run_if_not(in_conversation_condition)
                    .label(PLAYER_INPUT_LABEL)
                    .after(INVENTORY_LABEL)
            )
            .with_system(
                player_movement_update
                    .run_if_not(in_conversation_condition)
                    .label(PLAYER_MOVEMENT_LABEL)
                    .after(PLAYER_INPUT_LABEL)
            )
            .with_system(
                common::behaviors::pathfinder::pathfinder_update
                .chain(common::behaviors::werewolf::werewolf_update)
                .run_if_not(in_conversation_condition)
                .label(NPC_BEHAVIOR_UPDATE_LABEL)
                .after(PLAYER_MOVEMENT_LABEL)
            )
            .with_system(
                npc_movement_update
                    .run_if_not(in_conversation_condition)
                    .label(NPC_MOVEMENT_UPDATE_LABEL)
                    .after(NPC_BEHAVIOR_UPDATE_LABEL)
            )
            .with_system(
                collision_update
                    .run_if_not(in_conversation_condition)
                    .label(COLLISION_UPDATE_LABEL)
                    .after(NPC_MOVEMENT_UPDATE_LABEL),
            )
            .with_system(
                interact_update
                    .run_if_not(in_conversation_condition)
                    .label(INTERACT_UPDATE_LABEL)
                    .after(COLLISION_UPDATE_LABEL)
            )
            .with_system(
                destroy_check_update
                    .run_if_not(in_conversation_condition)
                    .label(DESTORY_CHECK_LABEL)
                    .after(INTERACT_UPDATE_LABEL)
            )
        )

        .add_system_set(SystemSet::on_update(Scene::Inventory)
            .with_system(
                inventory_update
                    .run_if_not(in_conversation_condition)
                    .label(INVENTORY_LABEL)
            )
        )

        .run();

}
