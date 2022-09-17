use crate::physics::{Map, Tile, Position, Zone, KrillTheaterZone};

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
    fn set_krill_theater_lineup(&mut self, x: usize, y: usize, position: Position) {
        if let Some(tile) = self.get_mut(x, y) {
            match tile {
                Tile::Ground { zone, .. } => *zone = Zone::KrillTheater { zone: KrillTheaterZone::LineUp(position) },
                _ => {},
            }
        }
    }
}
pub fn town(map: &mut Map) {
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
            Tile::Obstacle,
            |tile| *tile = Tile::Obstacle,
        );
    }

    {
        const MIN_HOME_POSITION: Position = Position { x: 60, y: 8 };
        const MAX_HOME_POSITION: Position = Position { x: 160, y: 44 };
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
                if let Some(tile) = map.get_mut(x, y.0) {
                    match tile {
                        Tile::Ground { zone, .. } => *zone = Zone::KrillTheater { zone: KrillTheaterZone::LineUp(y.1) },
                        _ => {},
                    }
                }
            }
        }
        for x in [
            (MIN_POSITION_X, Position::new(MIN_POSITION_X as i32, MIN_POSITION_Y as i32)),
            (MAX_POSITION_X, Position::new(MAX_POSITION_X as i32, MAX_POSITION_Y as i32)),
        ] {
            for y in MIN_POSITION_Y..=MAX_POSITION_Y {
                if let Some(tile) = map.get_mut(x.0, y) {
                    match tile {
                        Tile::Ground { zone, .. } => *zone = Zone::KrillTheater { zone: KrillTheaterZone::LineUp(x.1) },
                        _ => {},
                    }
                }
            }
        }
        map.set_krill_theater_lineup(MAX_POSITION_X, MAX_POSITION_Y, Position::new(MIN_POSITION_X as i32, MAX_POSITION_Y as i32));
        map.set_krill_theater_lineup(MIN_POSITION_X, MAX_POSITION_Y, Position::new(MIN_POSITION_X as i32, MIN_POSITION_Y as i32));
        map.set_krill_theater_lineup(MIN_POSITION_X, MIN_POSITION_Y, Position::new(MAX_POSITION_X as i32, MIN_POSITION_Y as i32));
        map.set_krill_theater_lineup(MAX_POSITION_X, MIN_POSITION_Y, Position::new(MAX_POSITION_X as i32, MAX_POSITION_Y as i32));

        const HOME_WIDTH: i32 = MAX_HOME_POSITION.x - MIN_HOME_POSITION.x;
        const MIN_THEATER_POSITION: Position = Position { x: 4 + MIN_HOME_POSITION.x, y: 4 + MIN_HOME_POSITION.y };
        const MAX_THEATER_POSITION: Position = Position { x: HOME_WIDTH - 4 + MIN_HOME_POSITION.x, y: 4 + 10 + MIN_HOME_POSITION.y };
        home(
            map,
            MIN_THEATER_POSITION,
            MAX_THEATER_POSITION,
            Zone::KrillTheater { zone: KrillTheaterZone::Free },
        );

        const MIN_KITCHEN_POSITION: Position = Position { x: 4 + MIN_HOME_POSITION.x, y: MAX_HOME_POSITION.y - 4 - 14 };
        const MAX_KITCHEN_POSITION: Position = Position { x: MIN_HOME_POSITION.x + 20, y: MAX_HOME_POSITION.y - 4 };
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

        const MIN_INVENTORY_POSITION: Position = Position { x: MAX_HOME_POSITION.x - 4 - 30, y: MAX_HOME_POSITION.y - 4 - 10 };
        const MAX_INVENTORY_POSITION: Position = Position { x: MAX_HOME_POSITION.x - 4, y: MAX_HOME_POSITION.y - 4 };
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

    home(map, Position::new(29, 49), Position::new(29 + 10, 49 + 10), Zone::Home);
    home(map, Position::new(29, 29), Position::new(29 + 10, 29 + 10), Zone::Home);
}
