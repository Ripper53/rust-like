use bevy::prelude::*;
use client::render::*;
use common::{physics::*, character::*, map_brain::Behavior, dialogue::Dialogue};

const X: usize = 60;
const Y: usize = 30;

fn setup<const X: usize, const Y: usize>(mut commands: Commands, mut map: ResMut<Map<X, Y>>) {
    map.spawn_character(
        &mut commands,
        common::character::Sprite::new('@'),
        Position::new(1, 2),
        Velocity::new(0, 0),
        CharacterData::Player { inventory: Inventory { } },
        |mut entity_commands| {
            entity_commands.insert(PlayerTag);
        },
    );
    map.spawn_character(
        &mut commands,
        common::character::Sprite::new('L'),
        Position::new(2, 1),
        Velocity::new(0, 0),
        CharacterData::Lawyer,
        |mut entity_commands| {
            entity_commands.insert(common::map_brain::Brain::new(vec![
                Behavior::default_lawyer(),
            ]));
        },
    );
}

fn main() {

    App::new()
        .set_runner(runner::<X, Y>)
        .init_resource::<PlayerInput>()
        .insert_resource(Dialogue::default())
        .init_resource::<Map<X, Y>>()
        .add_startup_system(setup::<X, Y>)
        .add_system(player_update)
        .add_system(player_movement_update::<X, Y>.after(player_update))
        .add_system(common::map_brain::brain_update::<X, Y>.after(player_movement_update::<X, Y>))
        .add_system(npc_movement_update::<X, Y>.after(player_update).after(common::map_brain::brain_update::<X, Y>))
        .add_system(physics_update::<X, Y>.after(npc_movement_update::<X, Y>))
        .run();

}
