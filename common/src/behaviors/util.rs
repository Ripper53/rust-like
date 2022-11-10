use bevy::prelude::Entity;
use crate::{physics::{Map, MapCache, Position, Tile, Occupier}, character::CharacterType, map_brain::HumanState};
use super::pathfinder::{data::PathfinderGlobalData, Priority, PathfinderBehavior};

/// Returns any werewolf in vision.
pub fn human_panic(map: &Map, map_cache: &mut MapCache, position: Position) -> Option<(Entity, Position)> {
    let vision = map.get_in_vision(map_cache, position);
    for p in vision.iter() {
        if let Some(Tile::Ground { occupier, .. } | Tile::Obstacle { occupier }) = map.get(p.x as usize, p.y as usize) {
            if let Some(Occupier { character_type: Some(CharacterType::Werewolf), entity,  .. }) = occupier {
                return Some((*entity, *p));
            }
        }
    }
    None
}

/// Sets the state to panic with high priority!
pub fn set_human_panic(
    data: &PathfinderGlobalData,
    behavior: &mut PathfinderBehavior,
    state: &mut HumanState,
    friendly: (CharacterType, Position),
    enemy_position: Position,
) {
    let (position, index) = data.human.panic(friendly).enemy(enemy_position).get();
    behavior.set_goal(position, Priority::High);
    *state = HumanState::Panic(index);
}
