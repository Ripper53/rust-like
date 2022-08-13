use bevy::prelude::*;
use crate::character::Health;

#[derive(Component)]
pub struct Brain {
    behaviors: Vec<Behavior>,
    order: i32,
}
impl Brain {
    pub fn new(behaviors: Vec<Behavior>) -> Brain {
        Brain { behaviors, order: 0 }
    }
}

pub enum Behavior {
    Primary,
}
impl Behavior {
    fn execute(&mut self, health: &mut Health) {
        match self {
            Behavior::Primary => {},
        }
    }
}

pub fn brain_update(mut query: Query<(&mut Brain, &mut Health)>) {
    let mut sorted_query = Vec::from_iter(query.iter_mut());
    sorted_query.sort_by(|(brain_a, _), (brain_b, _)| brain_a.order.cmp(&brain_b.order));

    for (brain, health) in sorted_query.iter_mut() {
        for behavior in brain.behaviors.iter_mut() {
            behavior.execute(&mut *health);
        }
    }
}
