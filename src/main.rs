use ggez::conf::{WindowMode, WindowSetup};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{event, graphics, GameError};

use scorched_earth::{Assets, GameState, Point2, Vector2};

struct MainState {
    game_state: GameState,
    assets: Assets,
    landscape_image: Option<graphics::Image>,
    borders_mesh: graphics::Mesh,
    missile_mesh: graphics::Mesh,
    explosion_mesh: graphics::Mesh,
}

impl MainState {
    fn new(ctx: &mut ggez::Context) -> ggez::GameResult<MainState> {
        let (width, height) = screen_size(ctx);
        let game_state =
            GameState::new(width - 2.0, height - 2.0).map_err(GameError::ResourceLoadError)?;

        let mut state = MainState {
            game_state,
            landscape_image: None,
            assets: Assets::new(ctx)?,
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
        };
        state.build_landscape_image(ctx)?;

        Ok(state)
    }

    fn build_landscape_image(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        self.landscape_image = Some(graphics::Image::from_rgba8(
            ctx,
            self.game_state.width as u16,
            self.game_state.height as u16,
            &self.game_state.landscape.to_rgba(),
        )?);
        self.game_state.landscape.changed = false;
        Ok(())
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        self.game_state.update_tanks();
        self.game_state.update_missile();
        self.game_state.update_explosion();

        if self.landscape_image.is_none() || self.game_state.landscape.changed {
            self.build_landscape_image(ctx)?;
        }

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
                //let dst = Point2::new(0.0, 0.0);
                graphics::draw(ctx, image, ([0.0, 0.0],))?;
            }

            // Tanks
            for tank in &self.game_state.tanks {
                let pos = tank.top_left();
                let gun_params = graphics::DrawParam::new()
                    .dest(pos + Vector2::new(20.5, 20.5))
                    .offset(Point2::new(0.5, 0.5))
                    .rotation(std::f32::consts::PI * tank.angle / 180.0);
                graphics::draw(ctx, &self.assets.gun_image, gun_params)?;
                let tank_params = graphics::DrawParam::new().dest(pos);
                graphics::draw(ctx, &self.assets.tank_image, tank_params)?;
            }

            // Missile
            if let Some(missile) = self.game_state.missile.as_ref() {
                graphics::draw(ctx, &self.missile_mesh, (missile.cur_pos(),))?;
            }

            // Explosion
            if let Some(explosion) = self.game_state.explosion.as_ref() {
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
        let angle = self.game_state.gun_angle();
        let text = graphics::Text::new((format!("Angle: {}", angle), self.assets.font, 20.0));
        let dest_point = Point2::new(10.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        let power = self.game_state.gun_power();
        let text = graphics::Text::new((format!("Power: {}", power), self.assets.font, 20.0));
        let dest_point = Point2::new(110.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        let text = graphics::Text::new((
            format!("Wind: {}", self.game_state.wind_power * 10.0),
            self.assets.font,
            20.0,
        ));
        let dest_point = Point2::new(220.0, 10.0);
        graphics::draw(ctx, &text, (dest_point,))?;

        let player = self.game_state.current_tank + 1;
        let text = graphics::Text::new((format!("Player: {}", player), self.assets.font, 20.0));
        let dest_point = Point2::new(440.0, 10.0);
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
            self.game_state.update_landscape_seed();
            self.game_state.regenerate_landscape();
            self.landscape_image = None;
        }
        if keycode == KeyCode::Left {
            self.game_state.inc_gun_angle(-1.0);
        }
        if keycode == KeyCode::Right {
            self.game_state.inc_gun_angle(1.0);
        }
        if keycode == KeyCode::Up {
            self.game_state.inc_gun_power(1.0);
        }
        if keycode == KeyCode::Down {
            self.game_state.inc_gun_power(-1.0);
        }
        if keycode == KeyCode::Space {
            self.game_state.shoot();
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
