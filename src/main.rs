use bevy::prelude::*;
use client::render::*;
use common::{physics::*, character::*, dialogue::Dialogue, inventory::{inventory_update, Inventory, Item}, util::{spawn_lerain, spawn_werewolf, spawn_chest}, ActionInput, Scene, behaviors::pathfinder::data::PathfinderGlobalData, PlayerState, loot_menu::LootMenu};
use iyes_loopless::condition::IntoConditionalExclusiveSystem;

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
    spawn_lerain(&mut commands, &mut map, Position::new(50, 8));
    //spawn_lerain(&mut commands, &mut map, Position::new(20, 40));
    //spawn_lerain(&mut commands, &mut map, Position::new(30, 10));
    //spawn_lerain(&mut commands, &mut map, Position::new(25, 20));
    //spawn_werewolf(&mut commands, &mut map, Position::new(2, 4));
    
    spawn_chest(&mut commands, &mut map, Position::new(50, 10), Inventory::new(
        vec![
            Box::new(Item::new_apple()),
            Box::new(Item::new_banana()),
            Box::new(Item::new_apple()),
        ],
    ));
    spawn_chest(&mut commands, &mut map, Position::new(49, 10), Inventory::new(
        vec![
            Box::new(Item::new_apple()),
            Box::new(Item::new_banana()),
            Box::new(Item::new_apple()),
            Box::new(Item::new_apple()),
            Box::new(Item::new_banana()),
            Box::new(Item::new_apple()),
        ],
    ));
}

fn pause_main_game(player_state: Res<PlayerState>) -> bool {
    match *player_state {
        PlayerState::Dialogue |
        PlayerState::Looting => true,
        PlayerState::None => false,
    }
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
        .insert_resource(PlayerState::default())
        .init_resource::<PlayerInput>()
        .insert_resource(ActionInput::None)
        .insert_resource(Dialogue::default())
        .insert_resource(LootMenu::default())
        .init_resource::<PathfinderGlobalData>()
        .insert_resource(MapCache::default())
        .init_resource::<Map>()
        .add_startup_system(setup)

        .add_system_set(SystemSet::on_update(Scene::Map)
            .with_system(
                inventory_update
                    .run_if_not(pause_main_game)
                    .label(INVENTORY_LABEL)
            )
            .with_system(
                player_movement_input_update
                    .run_if_not(pause_main_game)
                    .label(PLAYER_INPUT_LABEL)
                    .after(INVENTORY_LABEL)
            )
            .with_system(
                player_movement_update
                    .run_if_not(pause_main_game)
                    .label(PLAYER_MOVEMENT_LABEL)
                    .after(PLAYER_INPUT_LABEL)
            )
            .with_system(
                common::behaviors::pathfinder::pathfinder_update
                .chain(common::behaviors::werewolf::werewolf_update)
                .run_if_not(pause_main_game)
                .label(NPC_BEHAVIOR_UPDATE_LABEL)
                .after(PLAYER_MOVEMENT_LABEL)
            )
            .with_system(
                npc_movement_update
                    .run_if_not(pause_main_game)
                    .label(NPC_MOVEMENT_UPDATE_LABEL)
                    .after(NPC_BEHAVIOR_UPDATE_LABEL)
            )
            .with_system(
                collision_update
                    .run_if_not(pause_main_game)
                    .label(COLLISION_UPDATE_LABEL)
                    .after(NPC_MOVEMENT_UPDATE_LABEL),
            )
            .with_system(
                interact_update
                    .run_if_not(pause_main_game)
                    .label(INTERACT_UPDATE_LABEL)
                    .after(COLLISION_UPDATE_LABEL)
            )
            .with_system(
                destroy_check_update
                    .run_if_not(pause_main_game)
                    .label(DESTORY_CHECK_LABEL)
                    .after(INTERACT_UPDATE_LABEL)
            )
        )

        .add_system_set(SystemSet::on_update(Scene::Inventory)
            .with_system(
                inventory_update
                    .label(INVENTORY_LABEL)
            )
        )

        .run();

}
