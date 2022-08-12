use bevy::prelude::*;
use crate::physics::*;

#[derive(Component)]
pub struct Room {
}

pub fn update_rooms(query: Query<(&Room, &Position, &Velocity)>) {
}
