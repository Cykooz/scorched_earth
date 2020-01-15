use ggez;
use ggez::{audio, graphics};

use crate::types::Point2;
use crate::{input, shaders, utils};

pub struct World {
    pub input: input::State,
    pub tank_image: graphics::Image,
    pub gun_image: graphics::Image,
    pub font: graphics::Font,
    pub tank_fire_sound: audio::Source,
    pub explosion_sound: audio::Source,
    pub borders_mesh: graphics::Mesh,
    pub missile_mesh: graphics::Mesh,
    pub explosion_mesh: graphics::Mesh,
    pub glow_shader: shaders::GlowShader,
}

impl World {
    pub fn new(ctx: &mut ggez::Context) -> ggez::GameResult<Self> {
        let (width, height) = utils::screen_size(ctx);

        let world = Self {
            input: input::State::new(),
            tank_image: graphics::Image::new(ctx, "/sprites/tank.png")?,
            gun_image: graphics::Image::new(ctx, "/sprites/gun.png")?,
            font: graphics::Font::new(ctx, "/fonts/DejaVuSerif.ttf")?,
            tank_fire_sound: audio::Source::new(ctx, "/sounds/cannon_fire.ogg")?,
            explosion_sound: audio::Source::new(ctx, "/sounds/explosion1.ogg")?,
            borders_mesh: graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::stroke(1.0),
                graphics::Rect::new(0.0, 0.0, width - 1.0, height - 1.0),
                graphics::Color::from_rgb(255, 255, 255),
            )?,
            missile_mesh: graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                Point2::new(0.0, 0.0),
                2.0,
                1.0,
                graphics::WHITE,
            )?,
            explosion_mesh: graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                Point2::new(0.0, 0.0),
                1000.0,
                0.5,
                graphics::WHITE,
            )?,
            glow_shader: shaders::load_glow_shader(ctx)?,
        };

        Ok(world)
    }
}
