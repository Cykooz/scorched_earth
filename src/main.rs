use ggez::conf::{WindowMode, WindowSetup};
use ggez::{event, graphics, timer};

use scorched_earth::{input, scenes, world};

struct MainState {
    scenes: scenes::Stack,
    input_binding: input::Binding,
}

impl MainState {
    fn new(ctx: &mut ggez::Context) -> ggez::GameResult<MainState> {
        let world = world::World::new(ctx)?;
        let mut scene_stack = scenes::Stack::new(ctx, world);
        let main_menu_scene = scenes::main_menu::MainMenuScene::new(ctx, &mut scene_stack.world);
        scene_stack.push(Box::new(main_menu_scene));

        Ok(Self {
            input_binding: input::create_input_binding(),
            scenes: scene_stack,
        })
    }
}

impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        const DESIRED_FPS: u32 = 60;
        while timer::check_update_time(ctx, DESIRED_FPS) {
            self.scenes.update(ctx);
            self.scenes.world.input.update(0.0);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        self.scenes.draw(ctx);
        graphics::present(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut ggez::Context,
        key_code: event::KeyCode,
        _key_mods: event::KeyMods,
        _repeat: bool,
    ) {
        if let Some(ev) = self.input_binding.resolve(key_code) {
            if let input::Event::Button(button) = ev {
                self.scenes.world.input.update_button_down(button)
            }
            self.scenes.input(ev, true);
        }
    }

    fn key_up_event(
        &mut self,
        _ctx: &mut ggez::Context,
        key_code: event::KeyCode,
        _key_mods: event::KeyMods,
    ) {
        if let Some(ev) = self.input_binding.resolve(key_code) {
            if let input::Event::Button(button) = ev {
                self.scenes.world.input.update_button_up(button)
            }
            self.scenes.input(ev, false);
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
