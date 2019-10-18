use sdl2::render::WindowCanvas;

pub use app::App;
pub use game_state::GameState;
pub use landscape::Landscape;
pub use resources::Resources;
pub use tank::Tank;
pub use window::Window;

mod app;
mod game_state;
mod landscape;
mod resources;
mod tank;
mod window;

pub trait Draw {
    fn draw(&self, canvas: &mut WindowCanvas) -> Result<(), String>;
}
