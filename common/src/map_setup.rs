use bevy::prelude::Commands;

use crate::{physics::{Map, Tile, Position, Zone, KrillTheaterZone}, behaviors::pathfinder::data::PathfinderGlobalData, util::{spawn_chest, spawn_werewolf, spawn_lerain}, inventory::{Inventory, Item}, character::Health};

impl Map {
    fn create_room<F: Fn(&mut Tile)>(&mut self, bottom_left: Position, top_right: Position, border_tile: Tile, place_tile: F) {
        for y in bottom_left.y..=top_right.y {
            for x in bottom_left.x..=top_right.x {
                if let Some(tile) = self.get_mut(x as usize, y as usize) {
                    place_tile(tile);
                }
            }
        }
        // Borders
        for y in bottom_left.y..=top_right.y {
            if let Some(tile) = self.get_mut(bottom_left.x as usize, y as usize) {
                *tile = border_tile.clone();
            }
            if let Some(tile) = self.get_mut(top_right.x as usize, y as usize) {
                *tile = border_tile.clone();
            }
        }
        for x in bottom_left.x..=top_right.x {
            if let Some(tile) = self.get_mut(x as usize, bottom_left.y as usize) {
                *tile = border_tile.clone();
            }
            if let Some(tile) = self.get_mut(x as usize, top_right.y as usize) {
                *tile = border_tile.clone();
            }
        }
    }
}

impl Map {
    fn set_krill_theater_lineup(&mut self, x: usize, y: usize, position: Position, data: &PathfinderGlobalData) {
        if let Some(tile) = self.get_mut(x, y) {
            let p = Position::new(x as i32, y as i32);
            let z = if data.is_krill_exit(&p) {
                KrillTheaterZone::Exit
            } else {
                KrillTheaterZone::LineUp(position)
            };
            match tile {
                Tile::Ground { zone, .. } => *zone = Zone::KrillTheater {
                    zone: z,
                },
                _ => {},
            }
        }
    }
}
pub fn town(commands: &mut Commands, map: &mut Map, data: &PathfinderGlobalData) {
    map.initialize::<220, 100>();

    fn home(map: &mut Map, bottom_left: Position, top_right: Position, zone: Zone) {
        map.create_room(
            bottom_left,
            top_right,
            Tile::Wall,
            |tile| *tile = Tile::Ground { occupier: None, zone: zone.clone() },
        );
    }
    fn obstacle(map: &mut Map, bottom_left: Position, top_right: Position) {
        map.create_room(
            bottom_left,
            top_right,
            Tile::Obstacle { occupier: None },
            |tile| *tile = Tile::Obstacle { occupier: None },
        );
    }
    fn home_entrance(map: &mut Map, position: Position) {
        if let Some(tile) = map.get_mut(position.x as usize, position.y as usize) {
            *tile = Tile::Ground { occupier: None, zone: Zone::Home };
        }
    }

    {
        const MIN_HOME_POSITION: Position = Position::new(60, 8);
        const MAX_HOME_POSITION: Position = Position::new(160, 44);
        const OFFSET: usize = 2;
        const MIN_POSITION_X: usize = MIN_HOME_POSITION.x as usize + OFFSET;
        const MAX_POSITION_X: usize = MAX_HOME_POSITION.x as usize - OFFSET;
        const MIN_POSITION_Y: usize = MIN_HOME_POSITION.y as usize + OFFSET;
        const MAX_POSITION_Y: usize = MAX_HOME_POSITION.y as usize - OFFSET;
        home(map, MIN_HOME_POSITION, MAX_HOME_POSITION, Zone::KrillTheater { zone: KrillTheaterZone::Free });

        for y in [
            (MIN_POSITION_Y, Position::new(MAX_POSITION_X as i32, MIN_POSITION_Y as i32)),
            (MAX_POSITION_Y, Position::new(MIN_POSITION_X as i32, MAX_POSITION_Y as i32)),
        ] {
            for x in MIN_POSITION_X..=MAX_POSITION_X {
                map.set_krill_theater_lineup(x, y.0, y.1, data);
            }
        }
        for x in [
            (MIN_POSITION_X, Position::new(MIN_POSITION_X as i32, MIN_POSITION_Y as i32)),
            (MAX_POSITION_X, Position::new(MAX_POSITION_X as i32, MAX_POSITION_Y as i32)),
        ] {
            for y in MIN_POSITION_Y..=MAX_POSITION_Y {
                map.set_krill_theater_lineup(x.0, y, x.1, data);
            }
        }
        map.set_krill_theater_lineup(
            MAX_POSITION_X, MAX_POSITION_Y,
            Position::new(MIN_POSITION_X as i32, MAX_POSITION_Y as i32),
            data,
        );
        map.set_krill_theater_lineup(
            MIN_POSITION_X, MAX_POSITION_Y,
            Position::new(MIN_POSITION_X as i32, MIN_POSITION_Y as i32),
            data,
        );
        map.set_krill_theater_lineup(
            MIN_POSITION_X, MIN_POSITION_Y,
            Position::new(MAX_POSITION_X as i32, MIN_POSITION_Y as i32),
            data,
        );
        map.set_krill_theater_lineup(
            MAX_POSITION_X, MIN_POSITION_Y,
            Position::new(MAX_POSITION_X as i32, MAX_POSITION_Y as i32),
            data,
        );

        const HOME_WIDTH: i32 = MAX_HOME_POSITION.x - MIN_HOME_POSITION.x;
        const MIN_THEATER_POSITION: Position = Position::new(4 + MIN_HOME_POSITION.x, 4 + MIN_HOME_POSITION.y);
        const MAX_THEATER_POSITION: Position = Position::new(HOME_WIDTH - 4 + MIN_HOME_POSITION.x, 4 + 10 + MIN_HOME_POSITION.y);
        home(
            map,
            MIN_THEATER_POSITION,
            MAX_THEATER_POSITION,
            Zone::KrillTheater { zone: KrillTheaterZone::Free },
        );

        const MIN_KITCHEN_POSITION: Position = Position::new(4 + MIN_HOME_POSITION.x, MAX_HOME_POSITION.y - 4 - 14);
        const MAX_KITCHEN_POSITION: Position = Position::new(MIN_HOME_POSITION.x + 20, MAX_HOME_POSITION.y - 4);
        home(
            map,
            MIN_KITCHEN_POSITION,
            MAX_KITCHEN_POSITION,
            Zone::KrillTheater { zone: KrillTheaterZone::Free },
        );
        home(
            map,
            MIN_KITCHEN_POSITION + Position::new(0, 6),
            MAX_KITCHEN_POSITION,
            Zone::KrillTheater { zone: KrillTheaterZone::Free },
        );
        obstacle(
            map,
            MIN_KITCHEN_POSITION + Position::new(5, 9),
            MAX_KITCHEN_POSITION - Position::new(5, 3),
        );

        const MIN_INVENTORY_POSITION: Position = Position::new(MAX_HOME_POSITION.x - 4 - 30, MAX_HOME_POSITION.y - 4 - 10);
        const MAX_INVENTORY_POSITION: Position = Position::new(MAX_HOME_POSITION.x - 4, MAX_HOME_POSITION.y - 4);
        home(
            map,
            MIN_INVENTORY_POSITION,
            MAX_INVENTORY_POSITION,
            Zone::KrillTheater { zone: KrillTheaterZone::Free },
        );
        home(
            map,
            MIN_INVENTORY_POSITION + Position::new(10, 0),
            MAX_INVENTORY_POSITION,
            Zone::KrillTheater { zone: KrillTheaterZone::Free },
        );
        home(
            map,
            MIN_INVENTORY_POSITION + Position::new(20, 0),
            MAX_INVENTORY_POSITION,
            Zone::KrillTheater { zone: KrillTheaterZone::Free },
        );

        const THEATER_WIDTH: i32 = MAX_THEATER_POSITION.x - MIN_THEATER_POSITION.x;
        for position in [
            // MAIN GATES
            (MIN_HOME_POSITION.x as usize + (HOME_WIDTH as usize / 2 as usize), MAX_HOME_POSITION.y as usize),
            (MIN_HOME_POSITION.x as usize + (HOME_WIDTH as usize / 2 as usize) + 1, MAX_HOME_POSITION.y as usize),
            // BACK EXITS
            (MIN_HOME_POSITION.x as usize + 9, MIN_HOME_POSITION.y as usize),
            (MAX_HOME_POSITION.x as usize - 9, MIN_HOME_POSITION.y as usize),
            // THEATER GATES
            (MIN_THEATER_POSITION.x as usize + (THEATER_WIDTH as usize / 2 as usize), MAX_THEATER_POSITION.y as usize),
            (MIN_THEATER_POSITION.x as usize + (THEATER_WIDTH as usize / 2 as usize) + 1, MAX_THEATER_POSITION.y as usize),
            // KITCHEN DOOR
            (MAX_KITCHEN_POSITION.x as usize, MAX_KITCHEN_POSITION.y as usize - 2),
            // KITCHEN INNER-DOOR
            (MAX_KITCHEN_POSITION.x as usize - 2, MIN_KITCHEN_POSITION.y as usize + 6),
        ] {
            if let Some(tile) = map.get_mut(position.0, position.1) {
                *tile = Tile::new_ground(Zone::KrillTheater { zone: KrillTheaterZone::Free });
            }
        }
    }

    let position = Position::new(29, 29);
    home(map, position, position + Position::new(10, 6), Zone::Home);
    home_entrance(map, position + Position::new(10, 2));
    home(map, position + Position::new(6, 6), position + Position::new(10, 10), Zone::Home);
    home_entrance(map, position + Position::new(7, 6));
    spawn_chest(commands, map, position + Position::new(7, 9), Inventory::new(
        vec![
            Box::new(Item::new_apple()),
            Box::new(Item::new_banana()),
            Box::new(Item::new_apple()),
        ],
    ));
    let position = Position::new(29, 49);
    home(map, position, position + Position::new(10, 10), Zone::Home);
    home_entrance(map, position + Position::new(0, 8));
    home_entrance(map, position + Position::new(8, 0));
    let position = Position::new(49, 49);
    home(map, position, position + Position::new(10, 10), Zone::Home);
    home_entrance(map, position + Position::new(2, 0));
    home_entrance(map, position + Position::new(10, 8));
    let position = Position::new(200, 60);
    home(map, position, position + Position::new(10, 8), Zone::Home);
    home_entrance(map, position + Position::new(0, 2));
    spawn_chest(commands, map, position + Position::new(9, 1), Inventory::new(
        vec![
            Box::new(Item::new_pistol()),
        ],
    ));
    let position = Position::new(200, 72);
    home(map, position, position + Position::new(10, 10), Zone::Home);
    home_entrance(map, position + Position::new(0, 2));
    let position = Position::new(186, 72);
    home(map, position, position + Position::new(10, 10), Zone::Home);
    home_entrance(map, position + Position::new(0, 2));
    home_entrance(map, position + Position::new(10, 2));

    obstacle(map, Position::new(12, 38), Position::new(13, 46));
    for pos in [Position::new(18, 38), Position::new(21, 38), Position::new(24, 38), Position::new(27, 38)] {
        obstacle(map, pos, pos + Position::new(0, 8));
    }

    map.spawn_character(
        commands,
        crate::character::Sprite::Player,
        Position::new(50, 2),
        Health::new(4),
        crate::character::CharacterType::Player,
        crate::character::CharacterData::Human,
        |mut entity_commands| {
            entity_commands.insert(crate::character::PlayerTag);
        },
    );
    //spawn_lerain(&mut commands, &mut map, Position::new(50, 8));
    //spawn_lerain(&mut commands, &mut map, Position::new(20, 40));
    //spawn_lerain(&mut commands, &mut map, Position::new(30, 10));
    //spawn_lerain(commands, map, Position::new(25, 20));
    spawn_werewolf(commands, map, Position::new(2, 4));
}
