use ggez;
use ggez::graphics;

pub struct Assets {
    pub tank_image: graphics::Image,
    pub gun_image: graphics::Image,
    pub font: graphics::Font,
}

impl Assets {
    pub fn new(ctx: &mut ggez::Context) -> ggez::GameResult<Assets> {
        Ok(Assets {
            tank_image: graphics::Image::new(ctx, "/sprites/tank.png")?,
            gun_image: graphics::Image::new(ctx, "/sprites/gun.png")?,
            font: graphics::Font::new(ctx, "/fonts/DejaVuSerif.ttf")?,
        })
    }
}
