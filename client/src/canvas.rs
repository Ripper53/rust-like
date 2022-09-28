use bevy::prelude::World;
use common::physics::{Map, Tile, Position, MapCache};
use tui::{widgets::{Widget, Paragraph, Block, Borders}, style::{Style, Color}, text::{Span, Spans}};

pub struct MapCanvas<'a> {
    pub world: &'a mut World,
    pub center_position: Position,
    pub vision_position: Position,
}

impl<'a> Widget for MapCanvas<'a> {
    fn render(mut self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        fn get_center_coordinate(map_size: usize, screen_size: usize, target: usize) -> usize {
            const OFFSET: usize = 2;
            let diff = map_size - screen_size + OFFSET; // Offset to bottom.
            let clamp_to_bottom_of_center = diff - target.min(diff);
            let halfway = if clamp_to_bottom_of_center == 0 {
                let diff = map_size - target;
                diff - (screen_size / 2).min(diff)
            } else {
                // Minus one to center camera.
                let value = screen_size / 2;
                if value != 0 {
                    value - 1
                } else {
                    value
                }
            };
            let center = (clamp_to_bottom_of_center + halfway).min(map_size - screen_size + OFFSET);
            center
        }

        let map = self.world.resource::<Map>();
        let size_x = map.get_size_x();
        let size_y = map.get_size_y();
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

        let mut text = Vec::<Spans>::with_capacity(size_y);
        //let mut map_cache = self.world.resource_mut::<MapCache>();
        let in_vision = map.get_in_vision(&mut MapCache::default(), self.vision_position);
        for y in start_y..size_y {
            let mut t = Vec::<Span>::with_capacity(size_x);
            for x in start_x..size_x {
                let y = size_y - 1 - y;
                if let Some(tile) = map.get(x, y) {
                    let character = if /*in_vision.contains(&Position::new(x as i32, y as i32))*/ true {
                        match tile {
                            Tile::Ground { occupier, .. } => {
                                if let Some(occupier) = occupier {
                                    Span::raw(occupier.sprite.character.to_string())
                                } else {
                                    Span::raw(" ")
                                }
                            },
                            Tile::Wall => Span::raw("#"),
                            Tile::Obstacle { occupier } => {
                                if let Some(occupier) = occupier {
                                    Span::raw(occupier.sprite.character.to_string())
                                } else {
                                    Span::raw("%")
                                }
                            },
                        }
                    } else {
                        Span::styled("X", Style::default().fg(Color::DarkGray))
                    };
                    t.push(character);
                }
            }
            text.push(Spans::from(t));
        }

        let p = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("World"))
            .alignment(tui::layout::Alignment::Center);
        p.render(area, buf);
    }
}
