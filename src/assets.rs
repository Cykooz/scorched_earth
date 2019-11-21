use ggez;
use ggez::{audio, graphics};

pub struct Assets {
    pub tank_image: graphics::Image,
    pub gun_image: graphics::Image,
    pub font: graphics::Font,
    pub tank_fire_sound: audio::Source,
    pub explosion_sound: audio::Source,
}

impl Assets {
    pub fn new(ctx: &mut ggez::Context) -> ggez::GameResult<Assets> {
        Ok(Assets {
            tank_image: graphics::Image::new(ctx, "/sprites/tank.png")?,
            gun_image: graphics::Image::new(ctx, "/sprites/gun.png")?,
            font: graphics::Font::new(ctx, "/fonts/DejaVuSerif.ttf")?,
            tank_fire_sound: audio::Source::new(ctx, "/sounds/cannon_fire.ogg")?,
            explosion_sound: audio::Source::new(ctx, "/sounds/explosion1.ogg")?,
        })
    }
}
