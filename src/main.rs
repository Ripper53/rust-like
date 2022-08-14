use bevy::prelude::*;
use client::render::*;
use common::{physics::*, character::*};

const X: usize = 60;
const Y: usize = 30;

fn setup<const X: usize, const Y: usize>(mut commands: Commands, mut map: ResMut<Map<X, Y>>) {
    if let Some(character) = map.spawn_character(common::character::Sprite::new('@'), Position::new(1, 2), Velocity::new(0, 0)) {
        commands.spawn().insert_bundle(character).insert(PlayerTag);
    }
}

fn main() {

    App::new()
        .set_runner(runner::<X, Y>)
        .init_resource::<PlayerInput>()
        .init_resource::<Map<X, Y>>()
        .add_startup_system(setup::<X, Y>)
        .add_system(player_update)
        .add_system(common::map_brain::brain_update)
        .add_system(movement_update::<X, Y>.after(player_update).after(common::map_brain::brain_update))
        .add_system(physics_update::<X, Y>.after(movement_update::<X, Y>))
        .add_system(common::battle_brain::brain_update.after(physics_update::<X, Y>))
        .run();

}
