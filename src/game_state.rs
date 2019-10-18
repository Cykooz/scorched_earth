use crate::{Landscape, Tank};
use rand::rngs::ThreadRng;
use rand::Rng;
use sdl2::pixels::Color;

pub struct GameState {
    pub rng: ThreadRng,
    pub landscape: Landscape,
    pub tanks: Vec<Tank>,
    pub need_redraw: bool,
}

impl GameState {
    pub fn new(width: u32, height: u32) -> Result<GameState, String> {
        let mut rng = rand::thread_rng();
        let mut landscape = Landscape::new(width, height)?;
        landscape.set_seed(rng.gen());
        landscape.dx = rng.gen_range(0, width as i32 / 2);
        landscape.generate();

        let tanks = vec![
            Tank::new((100, 50).into(), Color::RGB(245, 71, 32)),
            Tank::new((width as i32 - 100, 50).into(), Color::RGB(42, 219, 39)),
        ];

        Ok(GameState {
            rng,
            landscape,
            tanks,
            need_redraw: true,
        })
    }

    #[inline]
    pub fn update_landscape_seed(&mut self) {
        self.landscape.set_seed(self.rng.gen());
    }

    #[inline]
    pub fn regenerate_landscape(&mut self) {
        self.landscape.generate();
    }

    pub fn update(&mut self) {
        let height = self.landscape.size().1 as i32;

        for tank in self.tanks.iter_mut() {
            'speed: for _ in 0..2 {
                let tank_bottom = tank.bottom_left();
                if tank_bottom.y >= height {
                    continue;
                }
                let under_tank = tank_bottom.offset(0, 1);
                let pixels_under_takn =
                    self.landscape.get_pixels_line_mut(under_tank, tank.width());
                if let Some(pixels) = pixels_under_takn {
                    if pixels.iter().any(|c| *c == 0) {
                        // Landscape under tank is not full filled - clear it and get down tank
                        pixels.iter_mut().for_each(|c| *c = 0);
                        tank.offset((0, 1));
                        self.need_redraw = true;
                    } else {
                        break 'speed;
                    }
                }
            }
        }
    }
}
