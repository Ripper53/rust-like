use common::physics::{Map, Tile, Position};
use tui::widgets::{Widget, Paragraph, Block, Borders};

pub struct MapCanvas<'a> {
    pub map: &'a Map,
    pub center_position: Position,
}

impl<'a> Widget for MapCanvas<'a> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        fn get_center_coordinate(map_size: usize, screen_size: usize, target: usize) -> usize {
            const OFFSET: usize = 2;
            let diff = map_size - screen_size + OFFSET; // Offset to bottom.
            let clamp_to_bottom_of_center = diff - target.min(diff);
            let halfway = if clamp_to_bottom_of_center == 0 {
                let diff = map_size - target;
                diff - (screen_size / 2).min(diff)
            } else {
                // Minus one to center camera.
                (screen_size / 2) - 1
            };
            let center = (clamp_to_bottom_of_center + halfway).min(map_size - screen_size + OFFSET);
            center
        }

        let size_x = self.map.get_size_x();
        let size_y = self.map.get_size_y();
        let screen_width = area.width as usize;
        let screen_height = area.height as usize;

        let (start_x, start_y) = if size_x > screen_width {
            (
                get_center_coordinate(size_x, screen_width, size_x - self.center_position.x as usize),
                if size_y > screen_height { get_center_coordinate(size_y, screen_height, self.center_position.y as usize) } else { 0 }
            )
        } else if size_y > screen_height {
            (0, get_center_coordinate(size_y, screen_height, self.center_position.y as usize))
        } else {
            (0, 0)
        };

        let mut text = String::with_capacity((size_x * size_y) + size_y);
        for y in start_y..size_y {
            for x in start_x..size_x {
                if let Some(tile) = self.map.get(x, size_y - 1 - y) {
                    let character = match tile {
                        Tile::Ground { occupier: occupier_option, .. } => {
                            if let Some(occupier) = occupier_option {
                                occupier.sprite.character
                            } else {
                                ' '
                            }
                        },
                        Tile::Wall => '#',
                    };
                    text.push(character);
                }
            }
            text.push('\n');
        }

        let p = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("World"))
            .alignment(tui::layout::Alignment::Center);
        p.render(area, buf);
    }
}
