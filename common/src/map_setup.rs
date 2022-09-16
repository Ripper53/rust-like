use crate::physics::{Map, Tile, Position, Zone, KrillTheaterZone};

impl Map {
    fn create_room<F: Fn(&mut Tile)>(&mut self, bottom_left: Position, top_right: Position, place_tile: F) {
        for y in bottom_left.y..top_right.y {
            for x in bottom_left.x..top_right.x {
                if let Some(tile) = self.get_mut(x as usize, y as usize) {
                    place_tile(tile);
                }
            }
        }
        // Borders
        for y in bottom_left.y..top_right.y {
            if let Some(tile) = self.get_mut(bottom_left.x as usize, y as usize) {
                *tile = Tile::Wall;
            }
            if let Some(tile) = self.get_mut(top_right.x as usize - 1, y as usize) {
                *tile = Tile::Wall;
            }
        }
        for x in bottom_left.x..top_right.x {
            if let Some(tile) = self.get_mut(x as usize, bottom_left.y as usize) {
                *tile = Tile::Wall;
            }
            if let Some(tile) = self.get_mut(x as usize, top_right.y as usize - 1) {
                *tile = Tile::Wall;
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
    map.initialize::<220, 60>();

    fn home(map: &mut Map, bottom_left: Position, top_right: Position, zone: Zone) {
        map.create_room(
            bottom_left,
            top_right,
            |tile| *tile = Tile::Ground { occupier: None, zone: zone.clone() }
        );
    }

    {
        const MIN_POSITION_Y: usize = 3;
        const MAX_POSITION_Y: usize = 6;
        const MIN_POSITION_X: usize = 2;
        const MAX_POSITION_X: usize = 17;
        home(map, Position::new(0, 0), Position::new(20, 10), Zone::KrillTheater { zone: KrillTheaterZone::Free });
        for y in [(2, Position::new(MAX_POSITION_X as i32, MIN_POSITION_Y as i32)), (7, Position::new(MIN_POSITION_X as i32, MAX_POSITION_Y as i32))] {
            for x in MIN_POSITION_X..=MAX_POSITION_X {
                if let Some(tile) = map.get_mut(x, y.0) {
                    match tile {
                        Tile::Ground { zone, .. } => *zone = Zone::KrillTheater { zone: KrillTheaterZone::LineUp(y.1) },
                        _ => {},
                    }
                    //*tile = Tile::Wall;
                }
            }
        }
        for x in [(2, Position::new(MIN_POSITION_X as i32, MIN_POSITION_Y as i32)), (17, Position::new(MAX_POSITION_X as i32, MAX_POSITION_Y as i32))] {
            for y in MIN_POSITION_Y..=MAX_POSITION_Y {
                if let Some(tile) = map.get_mut(x.0, y) {
                    match tile {
                        Tile::Ground { zone, .. } => *zone = Zone::KrillTheater { zone: KrillTheaterZone::LineUp(x.1) },
                        _ => {},
                    }
                    //*tile = Tile::Wall;
                }
            }
        }
        map.set_krill_theater_lineup(MAX_POSITION_X, MAX_POSITION_Y, Position::new(MIN_POSITION_X as i32, MAX_POSITION_Y as i32));
        map.set_krill_theater_lineup(MIN_POSITION_X, MAX_POSITION_Y, Position::new(MIN_POSITION_X as i32, MIN_POSITION_Y as i32));
        map.set_krill_theater_lineup(MIN_POSITION_X, MIN_POSITION_Y, Position::new(MAX_POSITION_X as i32, MIN_POSITION_Y as i32));
        map.set_krill_theater_lineup(MAX_POSITION_X, MIN_POSITION_Y, Position::new(MAX_POSITION_X as i32, MAX_POSITION_Y as i32));
        if let Some(tile) = map.get_mut(19, 7) {
            *tile = Tile::default_ground()
        }
    }

    home(map, Position::new(29, 49), Position::new(29 + 10, 49 + 10), Zone::Home);
    home(map, Position::new(29, 29), Position::new(29 + 10, 29 + 10), Zone::Home);
}
