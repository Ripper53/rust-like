use crate::physics::{Map, Tile, Position, Zone};

impl<const X: usize, const Y: usize> Map<X, Y> {
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

pub fn legal<const X: usize, const Y: usize>(map: &mut Map<X, Y>) {
    map.create_room(
        Position::new(0, 0),
        Position::new(20, 10),
        |tile| *tile = Tile::Ground { occupier: None, zone: Zone::Lawyer }
    );
    *map.get_mut(19, 7).unwrap() = Tile::default_ground();
}
