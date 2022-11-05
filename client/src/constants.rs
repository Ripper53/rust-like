use common::character::Sprite;
use tui::style::Color;

pub const fn sprite_to_str(sprite: &Sprite) -> (&'static str, Option<Color>) {
    match sprite {
        Sprite::Player => ("@", Some(Color::LightYellow)),
        Sprite::Lerain => ("H", Some(Color::LightCyan)),
        Sprite::Rumdare => ("H", Some(Color::LightCyan)),
        Sprite::Werewolf => ("W", Some(Color::LightRed)),
        Sprite::Projectile => ("o", Some(Color::White)),
        Sprite::Chest => ("M", Some(Color::Yellow)),
        Sprite::Unknown => ("?", None),
    }
}
