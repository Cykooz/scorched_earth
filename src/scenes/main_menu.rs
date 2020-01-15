use std::cmp::min;

use ggez;
use ggez::graphics::Color;
use ggez::{event, graphics};
use ggez_goodies::scene;

use crate::input;
use crate::scenes;
use crate::types::Point2;
use crate::world::World;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum MenuItem {
    Play,
    Quit,
}

const MENU_ITEMS: [(MenuItem, &str); 2] = [(MenuItem::Play, "Play"), (MenuItem::Quit, "Quit")];

pub struct MainMenuScene {
    current_item: usize,
}

impl MainMenuScene {
    pub fn new(_ctx: &mut ggez::Context, _world: &mut World) -> Self {
        MainMenuScene { current_item: 0 }
    }
}

impl scene::Scene<World, input::Event> for MainMenuScene {
    fn update(&mut self, world: &mut World, ctx: &mut ggez::Context) -> scenes::Switch {
        if world.input.get_button_pressed(input::Button::Quit) {
            event::quit(ctx);
            return scene::SceneSwitch::None;
        }

        if world.input.get_button_pressed(input::Button::Up) {
            self.current_item = self.current_item.saturating_sub(1);
        }
        if world.input.get_button_pressed(input::Button::Down) {
            self.current_item = min(self.current_item + 1, MENU_ITEMS.len() - 1)
        }

        if world.input.get_button_pressed(input::Button::Select) {
            match MENU_ITEMS[self.current_item].0 {
                MenuItem::Play => {
                    let game_play_scene = Box::new(
                        scenes::game_play::GamePlayScene::new(ctx, world)
                            .expect("Can't create GamePlayScene"),
                    );
                    scene::SceneSwitch::Push(game_play_scene)
                }
                MenuItem::Quit => {
                    event::quit(ctx);
                    scene::SceneSwitch::None
                }
            }
        } else {
            scene::SceneSwitch::None
        }
    }

    fn draw(&mut self, world: &mut World, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        graphics::clear(ctx, graphics::Color::from((0.0, 0.2, 0.4, 0.0)));

        for (index, &(_, item_text)) in MENU_ITEMS.iter().enumerate() {
            let text_color = if index == self.current_item {
                Color::new(1., 0., 0., 1.)
            } else {
                Color::new(1., 1., 1., 1.)
            };
            let text_fragment = graphics::TextFragment::new(item_text)
                .font(world.font)
                .scale(graphics::Scale::uniform(40.0))
                .color(text_color);

            let text = graphics::Text::new(text_fragment);
            let y = index as f32 * 50.0 + 330.0;
            let dest_point = Point2::new(480.0, y);
            graphics::draw(ctx, &text, (dest_point,))?;
        }

        Ok(())
    }

    fn input(&mut self, _world: &mut World, _ev: input::Event, _started: bool) {}

    fn name(&self) -> &str {
        "MainMenuScene"
    }
}
