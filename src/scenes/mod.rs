use ggez_goodies::scene;

pub use game_play::GamePlayScene;
pub use main_menu::MainMenuScene;
pub use select_count_of_players::SelectCountOfPlayersScene;

use crate::input;
use crate::world::World;

pub mod game_play;
pub mod main_menu;
pub mod select_count_of_players;

// Shortcuts for our scene type.
pub type Switch = scene::SceneSwitch<World, input::Event>;
pub type Stack = scene::SceneStack<World, input::Event>;
// Useless, since you can't impl type aliases.  :|
//pub trait Scene = scene::Scene<World, input::Event>;
