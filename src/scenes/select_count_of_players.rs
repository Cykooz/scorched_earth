use std::cmp::{max, min};

use ggez;
use ggez::graphics::{self, Color};
use ggez_goodies::scene;

use crate::types::Point2;
use crate::world::World;
use crate::{input, scenes, utils, MAX_PLAYERS_COUNT};

pub struct SelectCountOfPlayersScene {
    count_of_players: u8,
}

impl SelectCountOfPlayersScene {
    pub fn new(_ctx: &mut ggez::Context, world: &mut World) -> Self {
        Self {
            count_of_players: world.players_count(),
        }
    }
}

impl scene::Scene<World, input::Event> for SelectCountOfPlayersScene {
    fn update(&mut self, world: &mut World, ctx: &mut ggez::Context) -> scenes::Switch {
        if world.input.get_button_pressed(input::Button::Quit) {
            return scene::SceneSwitch::Pop;
        }

        if world.input.get_button_pressed(input::Button::Select) {
            world.create_players_count(self.count_of_players);

            let game_play_scene = Box::new(
                scenes::GamePlayScene::new(ctx, world).expect("Can't create GamePlayScene"),
            );
            return scene::SceneSwitch::Replace(game_play_scene);
        }

        scene::SceneSwitch::None
    }

    fn draw(&mut self, world: &mut World, ctx: &mut ggez::Context) -> ggez::GameResult<()> {
        graphics::clear(ctx, [0.0, 0.3, 0.3, 1.0].into());

        let text_fragment = graphics::TextFragment::new("Select Count Of Players: ")
            .font(world.font)
            .scale(graphics::Scale::uniform(40.0))
            .color(Color::new(1., 1., 1., 1.));

        let mut text = graphics::Text::new(text_fragment);
        text.add(
            graphics::TextFragment::new(self.count_of_players.to_string())
                .font(world.font)
                .scale(graphics::Scale::uniform(40.0))
                .color(Color::new(1., 0., 0., 1.)),
        );

        let (width, height) = utils::screen_size(ctx);
        let text_width = (text.width(ctx) + 5) / 10;
        let x = (width - (text_width * 10) as f32) / 2.;
        let y = (height - text.height(ctx) as f32) / 2.;
        let dest_point = Point2::new(x.round(), y.round());
        graphics::draw(ctx, &text, (dest_point,))?;

        Ok(())
    }

    fn input(&mut self, _world: &mut World, ev: input::Event, started: bool) {
        if started {
            if let input::Event::Button(button) = ev {
                match button {
                    input::Button::Down => {
                        self.count_of_players = max(self.count_of_players - 1, 2)
                    }
                    input::Button::Up => {
                        self.count_of_players = min(self.count_of_players + 1, MAX_PLAYERS_COUNT)
                    }
                    _ => (),
                }
            }
        }
    }

    fn name(&self) -> &str {
        "SelectCountOfPlayersScene"
    }
}
