use std::time::Duration;

use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

use crate::{Draw, GameState, Resources, Window};

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;
//const DX_STEP: i32 = 5;
const AMP_STEP: f64 = 10.;

pub struct App {
    window: Window,
}

impl App {
    pub fn new() -> Result<App, String> {
        //        let cur_dir =
        //            std::env::current_dir().map_err(|e| format!("Can't get current directory: {}", e))?;
        //        let resources_dir = cur_dir.join("resources");
        //        let resources = Resources::new(resources_dir)?;
        let window = Window::new(WIDTH, HEIGHT, "Scorched Earth - Rust Edition");

        Ok(App { window })
    }

    pub fn run(&mut self) -> Result<(), String> {
        let mut game_state = GameState::new(WIDTH, HEIGHT)?;

        for events in self.window.get_events_iter()? {
            self.process_events(&events, &mut game_state)?;

            game_state.update();

            if game_state.need_redraw {
                self.draw(&game_state)?;
                game_state.need_redraw = false;
            } else {
                std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            }
        }

        Ok(())
    }

    fn process_events(&self, events: &[Event], game_state: &mut GameState) -> Result<(), String> {
        let mut regen_landscape = false;

        for event in events {
            if let Event::KeyDown { keycode, .. } = event {
                match keycode {
                    //                    Some(Keycode::Left) => {
                    //                        game_state.landscape.dx -= DX_STEP;
                    //                        regen_landscape = true;
                    //                    }
                    //                    Some(Keycode::Right) => {
                    //                        game_state.landscape.dx += DX_STEP;
                    //                        regen_landscape = true;
                    //                    }
                    Some(Keycode::Up) => {
                        game_state.landscape.amplitude += AMP_STEP;
                        regen_landscape = true;
                    }
                    Some(Keycode::Down) => {
                        game_state.landscape.amplitude -= AMP_STEP;
                        regen_landscape = true;
                    }
                    Some(Keycode::Space) => {
                        game_state.update_landscape_seed();
                        regen_landscape = true;
                    }
                    _ => (),
                }
            }
        }

        if regen_landscape {
            game_state.regenerate_landscape();
            game_state.need_redraw = true;
        }

        Ok(())
    }

    fn draw(&mut self, game_state: &GameState) -> Result<(), String> {
        let canvas = &mut self.window.canvas;
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // Draw landscape
        game_state.landscape.draw(canvas)?;

        // Draw tanks
        for tank in game_state.tanks.iter() {
            tank.draw(canvas)?;
        }

        // Texts
        let text_color = Color::RGB(255, 255, 255);
        let strings = [
            format!("Seed: {}", game_state.landscape.seed()),
            format!("Amplitude: {}", game_state.landscape.amplitude),
            format!("Dx: {}", game_state.landscape.dx),
        ];
        for (row, string) in strings.iter().enumerate() {
            canvas.string(5, (row * 15 + 5) as _, string, text_color)?;
        }

        canvas.present();
        Ok(())
    }
}
