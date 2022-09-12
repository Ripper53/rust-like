use crate::physics::{Map, Tile, Position, Zone};

impl Map {
    fn create_room(&mut self, bottom_left: Position, top_right: Position, place_tile: fn(&mut Tile)) {
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

    fn home(map: &mut Map, bottom_left: Position, top_right: Position) {
        map.create_room(
            bottom_left,
            top_right,
            |tile| *tile = Tile::Ground { occupier: None, zone: Zone::Road }
        );
    }

    home(map, Position::new(0, 0), Position::new(20, 10));
    if let Some(tile) = map.get_mut(19, 7) {
        *tile = Tile::default_ground()
    }

    home(map, Position::new(29, 49), Position::new(29 + 10, 49 + 10));
    home(map, Position::new(29, 29), Position::new(29 + 10, 29 + 10));
}
