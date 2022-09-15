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

pub fn town(map: &mut Map) {
    map.initialize::<220, 60>();

    fn home(map: &mut Map, bottom_left: Position, top_right: Position, zone: Zone) {
        map.create_room(
            bottom_left,
            top_right,
            |tile| *tile = Tile::Ground { occupier: None, zone: zone.clone() }
        );
    }

    home(map, Position::new(0, 0), Position::new(20, 10), Zone::KrillTheater { zone: KrillTheaterZone::Free });
    for y in [2, 7] {
        for x in 2..18 {
            if let Some(tile) = map.get_mut(x, y) {
                match tile {
                    Tile::Ground { zone, .. } => *zone = Zone::KrillTheater { zone: KrillTheaterZone::LineUp },
                    _ => {},
                }
                //*tile = Tile::Wall;
            }
        }
    }
    for x in [2, 17] {
        for y in 3..7 {
            if let Some(tile) = map.get_mut(x, y) {
                match tile {
                    Tile::Ground { zone, .. } => *zone = Zone::KrillTheater { zone: KrillTheaterZone::LineUp },
                    _ => {},
                }
                //*tile = Tile::Wall;
            }
        }
    }
    if let Some(tile) = map.get_mut(19, 7) {
        *tile = Tile::default_ground()
    }

    home(map, Position::new(29, 49), Position::new(29 + 10, 49 + 10), Zone::Home);
    home(map, Position::new(29, 29), Position::new(29 + 10, 29 + 10), Zone::Home);
}
