use ggez;
use ggez::{graphics, timer, GameError};
use ggez_goodies::scene;

use crate::round::{GameState, Round};
use crate::shaders::GlowParams;
use crate::types::Point2;
use crate::world::World;
use crate::{input, scenes, utils};

pub struct GamePlayScene {
    game_round: Round,
    landscape_image: Option<graphics::Image>,
    glow_params: GlowParams,
    glow_canvas: graphics::Canvas,
}

impl GamePlayScene {
    pub fn new(ctx: &mut ggez::Context, _world: &mut World) -> ggez::GameResult<Self> {
        let (width, height) = utils::screen_size(ctx);
        let game_round = Round::new(width as u16 - 2, height as u16 - 2)
            .map_err(GameError::ResourceLoadError)?;

        let state = Self {
            game_round,
            landscape_image: None,
            glow_params: GlowParams {
                glow_color: [1., 1., 1.],
                glow_intensity: 1.0,
            },
            glow_canvas: graphics::Canvas::with_window_size(ctx)?,
        };

        Ok(state)
    }

    fn update_landscape_image(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        if self.landscape_image.is_none() || self.game_round.landscape.changed() {
            self.landscape_image = Some(self.game_round.landscape.create_image(ctx)?);
        }
        Ok(())
    }
}

impl scene::Scene<World, input::Event> for GamePlayScene {
    fn update(&mut self, world: &mut World, ctx: &mut ggez::Context) -> scenes::Switch {
        if world.input.get_button_pressed(input::Button::Quit) {
            return scene::SceneSwitch::Pop;
        }

        if world.input.get_button_pressed(input::Button::Fire) {
            self.game_round.shoot(world)
        }

        self.game_round.update(world);
        self.update_landscape_image(ctx)
            .expect("Can't update landscape image");
        self.glow_params.glow_intensity =
            (0.5 + (((timer::ticks(ctx) as f32) / 20.0).cos() / 2.0)) * 0.8;

        scene::SceneSwitch::None
    }

    fn draw(&mut self, world: &mut World, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        {
            // Render playable space
            let params = graphics::DrawParam::new().dest([1.0, 1.0]);
            graphics::push_transform(ctx, Some(params.to_matrix()));
            graphics::apply_transformations(ctx)?;

            // Landscape
            if let Some(image) = &self.landscape_image {
                graphics::draw(ctx, image, ([0.0, 0.0],))?;
            }

            // Current tank with glowing effect
            graphics::set_canvas(ctx, Some(&self.glow_canvas));
            graphics::clear(ctx, [0.0, 0.0, 0.0, 0.0].into());
            let cur_tank = &self.game_round.tanks[self.game_round.current_tank];
            cur_tank.draw(ctx, world)?;
            graphics::set_canvas(ctx, None);
            {
                let _lock = graphics::use_shader(ctx, &world.glow_shader);
                world.glow_shader.send(ctx, self.glow_params)?;
                graphics::draw(ctx, &self.glow_canvas, ([-1.0, -1.0],))?;
            }

            // Other tanks
            for (i, tank) in self.game_round.tanks.iter().enumerate() {
                if i != self.game_round.current_tank {
                    tank.draw(ctx, world)?;
                }
            }

            // Missile
            if let GameState::FlyingOfMissile(ref missile) = self.game_round.state {
                graphics::draw(ctx, &world.missile_mesh, (missile.cur_pos(),))?;
            }

            // Explosion
            if let GameState::Exploding(ref explosion) = self.game_round.state {
                let scale = explosion.cur_radius / 1000.0;
                let alpha = (explosion.cur_opacity * 255.0) as u8;
                let draw_params = graphics::DrawParam::new()
                    .dest(explosion.pos)
                    .scale([scale, scale])
                    .color(graphics::Color::from_rgba(242, 68, 15, alpha));
                graphics::draw(ctx, &world.explosion_mesh, draw_params)?;
            }

            graphics::pop_transform(ctx);
            graphics::apply_transformations(ctx)?;
        }

        // Borders
        graphics::draw(ctx, &world.borders_mesh, (Point2::new(1.0, 0.0),))?;

        // Status line
        let angle = self.game_round.gun_angle();
        let text = graphics::Text::new((format!("Angle: {}", angle), world.font, 20.0));
        let dest_point = Point2::new(10.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        let power = self.game_round.gun_power();
        let text = graphics::Text::new((format!("Power: {}", power), world.font, 20.0));
        let dest_point = Point2::new(110.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        let text = graphics::Text::new((
            format!("Wind: {}", self.game_round.wind_power * 10.0),
            world.font,
            20.0,
        ));
        let dest_point = Point2::new(220.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        let player = self.game_round.current_tank + 1;
        let text = graphics::Text::new((format!("Player: {}", player), world.font, 20.0));
        let dest_point = Point2::new(440.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        let health = self.game_round.health();
        let text = graphics::Text::new((format!("Health: {}", health), world.font, 20.0));
        let dest_point = Point2::new(540.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        //        graphics::present(ctx)?;
        Ok(())
    }

    fn input(&mut self, _world: &mut World, ev: input::Event, started: bool) {
        if started {
            if let input::Event::Button(button) = ev {
                match button {
                    input::Button::Left => self.game_round.inc_gun_angle(-1.0),
                    input::Button::Right => self.game_round.inc_gun_angle(1.0),
                    input::Button::Down => self.game_round.inc_gun_power(-1.0),
                    input::Button::Up => self.game_round.inc_gun_power(1.0),
                    _ => (),
                }
            }
        }
    }

    fn name(&self) -> &str {
        "GamePlayScene"
    }
}
