use bevy::prelude::*;
use client::render::*;
use common::{physics::*, character::*};

const X: usize = 30;
const Y: usize = 30;

fn setup(mut commands: Commands) {
    commands.spawn().insert_bundle(CharacterBundle {
        sprite: common::character::Sprite::new('A'),
        position: Position::new(29, 2),
        velocity: Velocity::new(0, 0),
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    app
        .init_resource::<Map::<X, Y>>()
        .add_startup_system(setup)
        .add_system(physics_update::<X, Y>);

    setup_game::<X, Y>(&mut app).expect("Game setup completed.");

    Ok(())
}
