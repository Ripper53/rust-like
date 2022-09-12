use bevy::prelude::*;
use client::render::*;
use common::{physics::*, character::*, map_brain::Behavior, dialogue::Dialogue, inventory::Inventory};
use iyes_loopless::{condition::IntoConditionalExclusiveSystem};

const X: usize = 60;
const Y: usize = 30;

fn setup(mut commands: Commands, mut map: ResMut<Map>) {
    map.spawn_character(
        &mut commands,
        common::character::Sprite::new('@'),
        Position::new(1, 2),
        Velocity::new(0, 0),
        CharacterData::Player { inventory: Inventory::default() },
        |mut entity_commands| {
            entity_commands.insert(PlayerTag);
        },
    );
    map.spawn_character(
        &mut commands,
        common::character::Sprite::new('L'),
        Position::new(2, 1),
        Velocity::new(0, 0),
        CharacterData::Lerain,
        |mut entity_commands| {
            entity_commands.insert(common::map_brain::Brain::new(vec![
                Behavior::default_slow_movement(),
            ]));
        },
    );
    /*map.spawn_character(
        &mut commands,
        common::character::Sprite::new('L'),
        Position::new(2, 1),
        Velocity::new(0, 0),
        CharacterData::Lerain,
        |mut entity_commands| {
            entity_commands.insert(common::map_brain::Brain::new(vec![
                Behavior::default_lawyer(),
            ]));
        },
    );*/
}

fn in_conversation_condition(dialogue: Res<Dialogue>) -> bool {
    dialogue.in_conversation
}

fn main() {

    const PLAYER_INPUT_LABEL: &str = "player_movement_input_update";
    const PLAYER_MOVEMENT_LABEL: &str = "player_movement_update";
    const BRAIN_UPDATE_LABEL: &str = "brain_update";
    const NPC_MOVEMENT_UPDATE: &str = "npc_movement_update";

    App::new()
        .set_runner(runner::<X, Y>)
        .init_resource::<PlayerInput>()
        .insert_resource(Dialogue::default())
        .insert_resource(Inventory::default())
        .init_resource::<Map>()
        .add_startup_system(setup)

        .add_system(
            player_movement_input_update
                .run_if_not(in_conversation_condition)
                .label(PLAYER_INPUT_LABEL)
        )
        .add_system(
            player_movement_update
                .run_if_not(in_conversation_condition)
                .label(PLAYER_MOVEMENT_LABEL)
                .after(PLAYER_INPUT_LABEL)
        )
        .add_system(
            common::map_brain::brain_update
                .run_if_not(in_conversation_condition)
                .label(BRAIN_UPDATE_LABEL)
                .after(PLAYER_MOVEMENT_LABEL)
        )
        .add_system(
            npc_movement_update
                .run_if_not(in_conversation_condition)
                .label(NPC_MOVEMENT_UPDATE)
                .after(BRAIN_UPDATE_LABEL)
        )
        .add_system(
            physics_update
                .run_if_not(in_conversation_condition)
                .after(NPC_MOVEMENT_UPDATE)
        )

        .run();

}
