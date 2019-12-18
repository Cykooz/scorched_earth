use gfx::{self, *};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{event, graphics, timer, GameError};

use ggez::graphics::Canvas;
use scorched_earth::{Assets, GameState, Point2, Round};

// Define the input struct for our shader.
gfx_defines! {
    constant GlowParams {
        glow_color: [f32; 3] = "glow_color",
        glow_intensity: f32 = "glow_intensity",
    }
}

struct MainState {
    game_round: Round,
    assets: Assets,
    landscape_image: Option<graphics::Image>,
    borders_mesh: graphics::Mesh,
    missile_mesh: graphics::Mesh,
    explosion_mesh: graphics::Mesh,
    glow_shader: graphics::Shader<GlowParams>,
    glow_params: GlowParams,
    glow_canvas: Canvas,
}

impl MainState {
    fn new(ctx: &mut ggez::Context) -> ggez::GameResult<MainState> {
        let (width, height) = screen_size(ctx);
        let width = width as u16;
        let height = height as u16;
        let game_round = Round::new(width - 2, height - 2).map_err(GameError::ResourceLoadError)?;
        let glow_params = GlowParams {
            glow_color: [1., 1., 1.],
            glow_intensity: 1.0,
        };

        let state = MainState {
            game_round,
            landscape_image: None,
            assets: Assets::new(ctx)?,
            borders_mesh: graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::stroke(1.0),
                graphics::Rect::new(0.0, 0.0, width as f32 - 1.0, height as f32 - 1.0),
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
            glow_shader: graphics::Shader::new(
                ctx,
                "/shaders/basic_150.glslv",
                "/shaders/glow.glslf",
                glow_params,
                "GlowParams",
                None,
            )?,
            glow_params,
            glow_canvas: Canvas::with_window_size(ctx)?,
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

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        self.game_round.update(&mut self.assets);
        self.update_landscape_image(ctx)?;
        self.glow_params.glow_intensity =
            (0.5 + (((timer::ticks(ctx) as f32) / 20.0).cos() / 2.0)) * 0.8;
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
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
            cur_tank.draw(ctx, &self.assets)?;
            graphics::set_canvas(ctx, None);
            {
                let _lock = graphics::use_shader(ctx, &self.glow_shader);
                self.glow_shader.send(ctx, self.glow_params)?;
                graphics::draw(ctx, &self.glow_canvas, ([-1.0, -1.0],))?;
            }

            // Other tanks
            for (i, tank) in self.game_round.tanks.iter().enumerate() {
                if i != self.game_round.current_tank {
                    tank.draw(ctx, &self.assets)?;
                }
            }

            // Missile
            if let GameState::FlyingOfMissile(ref missile) = self.game_round.state {
                graphics::draw(ctx, &self.missile_mesh, (missile.cur_pos(),))?;
            }

            // Explosion
            if let GameState::Exploding(ref explosion) = self.game_round.state {
                let scale = explosion.cur_radius / 1000.0;
                let alpha = (explosion.cur_opacity * 255.0) as u8;
                let draw_params = graphics::DrawParam::new()
                    .dest(explosion.pos)
                    .scale([scale, scale])
                    .color(graphics::Color::from_rgba(242, 68, 15, alpha));
                graphics::draw(ctx, &self.explosion_mesh, draw_params)?;
            }

            graphics::pop_transform(ctx);
            graphics::apply_transformations(ctx)?;
        }

        // Borders
        graphics::draw(ctx, &self.borders_mesh, (Point2::new(1.0, 0.0),))?;

        // Status line
        let angle = self.game_round.gun_angle();
        let text = graphics::Text::new((format!("Angle: {}", angle), self.assets.font, 20.0));
        let dest_point = Point2::new(10.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        let power = self.game_round.gun_power();
        let text = graphics::Text::new((format!("Power: {}", power), self.assets.font, 20.0));
        let dest_point = Point2::new(110.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        let text = graphics::Text::new((
            format!("Wind: {}", self.game_round.wind_power * 10.0),
            self.assets.font,
            20.0,
        ));
        let dest_point = Point2::new(220.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        let player = self.game_round.current_tank + 1;
        let text = graphics::Text::new((format!("Player: {}", player), self.assets.font, 20.0));
        let dest_point = Point2::new(440.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        let health = self.game_round.health();
        let text = graphics::Text::new((format!("Health: {}", health), self.assets.font, 20.0));
        let dest_point = Point2::new(540.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        ctx: &mut ggez::Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        if keycode == KeyCode::Escape {
            event::quit(ctx);
        }
        if keycode == KeyCode::Delete {
            self.game_round.update_landscape_seed();
            self.game_round.regenerate_landscape();
            self.landscape_image = None;
        }
        if keycode == KeyCode::Left {
            self.game_round.inc_gun_angle(-1.0);
        }
        if keycode == KeyCode::Right {
            self.game_round.inc_gun_angle(1.0);
        }
        if keycode == KeyCode::Up {
            self.game_round.inc_gun_power(1.0);
        }
        if keycode == KeyCode::Down {
            self.game_round.inc_gun_power(-1.0);
        }
        if keycode == KeyCode::Space {
            self.game_round.shoot(&mut self.assets)
        }
    }
}

#[inline]
fn screen_size(ctx: &mut ggez::Context) -> (f32, f32) {
    let screen_rect = graphics::screen_coordinates(ctx);
    (screen_rect.w, screen_rect.h)
}

pub fn main() -> ggez::GameResult {
    let resource_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = std::path::PathBuf::from(manifest_dir);
        path.push("assets");
        path
    } else {
        std::path::PathBuf::from("./assets")
    };

    let mut win_setup: WindowSetup = Default::default();
    win_setup = win_setup
        .title("Scorched Earth - Rust edition")
        .icon("/sprites/app_icon.png");
    let mut win_mode: WindowMode = Default::default();
    win_mode = win_mode.dimensions(1024., 768.);

    let cb = ggez::ContextBuilder::new("scorched_rust", "cykooz")
        .window_setup(win_setup)
        .window_mode(win_mode)
        .add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut MainState::new(ctx)?;
    event::run(ctx, event_loop, state)
}
