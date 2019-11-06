use ggez;
use ggez::conf::{NumSamples, WindowMode, WindowSetup};
use ggez::event::quit;
use ggez::graphics;
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::{event, GameError};
use scorched_earth::{Assets, GameState, Point2, Vector2};

struct MainState {
    game_state: GameState,
    assets: Assets,
    landscape_image: Option<graphics::Image>,
    missile_mesh: graphics::Mesh,
}

impl MainState {
    fn new(ctx: &mut ggez::Context) -> ggez::GameResult<MainState> {
        let (width, height) = graphics::size(ctx);
        let game_state = GameState::new(width, height).map_err(GameError::ResourceLoadError)?;

        let mut state = MainState {
            game_state,
            landscape_image: None,
            assets: Assets::new(ctx)?,
            missile_mesh: graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                Point2::new(0.0, 0.0),
                2.0,
                1.0,
                graphics::WHITE,
            )?,
        };
        state.build_landscape_image(ctx)?;

        Ok(state)
    }

    fn build_landscape_image(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        let (width, height) = graphics::size(ctx);
        self.landscape_image = Some(graphics::Image::from_rgba8(
            ctx,
            width as u16,
            height as u16,
            &self.game_state.landscape.to_rgba(),
        )?);
        Ok(())
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        if self.game_state.update_landscape() || self.landscape_image.is_none() {
            self.build_landscape_image(ctx)?;
        }

        let (width, height) = graphics::size(ctx);
        self.game_state.update_missile(width, height);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 1.0].into());

        if let Some(image) = &self.landscape_image {
            let dst = Point2::new(0.0, 0.0);
            graphics::draw(ctx, image, (dst,))?;
        }

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

        if let Some(missile) = self.game_state.missile.as_ref() {
            graphics::draw(ctx, &self.missile_mesh, (missile.cur_pos,))?;
        }

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
        let dest_point = Point2::new(210.0, 10.0);
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
            quit(ctx);
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
        .icon("/sprites/tank.png")
        .samples(NumSamples::Sixteen);
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
