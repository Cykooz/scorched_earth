use ggez::graphics::Color;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate::missile::G;
use crate::{Landscape, Missile, Tank, Vector2};

pub struct GameState {
    pub rng: ThreadRng,
    pub landscape: Landscape,
    pub wind_power: f32,
    pub tanks: Vec<Tank>,
    pub current_tank: usize,
    pub missile: Option<Missile>,
}

impl GameState {
    pub fn new(width: f32, height: f32) -> Result<GameState, String> {
        let mut rng = rand::thread_rng();
        let mut landscape = Landscape::new(width as u32, height as u32)?;
        landscape.set_seed(rng.gen());
        landscape.dx = rng.gen_range(0, width as i32 / 2);
        landscape.generate();

        let tanks = vec![
            Tank::new([100., 50.].into(), Color::from_rgb(245, 71, 32)),
            Tank::new([width - 100., 50.].into(), Color::from_rgb(42, 219, 39)),
        ];

        let wind_power = (rng.gen_range(-10.0_f32, 10.0_f32) * 10.0).round() / 10.0;

        Ok(GameState {
            rng,
            landscape,
            wind_power,
            tanks,
            current_tank: 0,
            missile: None,
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

    pub fn update_landscape(&mut self) -> bool {
        let row_offset = Vector2 { x: 0., y: 1. };
        let height = self.landscape.size().1 as f32;
        let mut res = false;

        for tank in self.tanks.iter_mut() {
            'speed: for _ in 0..2 {
                let tank_bottom = tank.bottom_left();
                if tank_bottom.y >= height {
                    continue;
                }
                let under_tank = tank_bottom + row_offset;
                let tank_width = tank.width() as u32;
                let pixels_under_tank = self.landscape.get_pixels_line_mut(under_tank, tank_width);
                if let Some(pixels) = pixels_under_tank {
                    let empty_count = bytecount::count(pixels, 0);
                    if empty_count > 0 {
                        if empty_count < tank_width as usize {
                            // Landscape under tank is not empty - clear it
                            pixels.iter_mut().for_each(|c| *c = 0);
                            res = true;
                        }
                        // Get down tank
                        tank.offset(0., 1.);
                    } else {
                        break 'speed;
                    }
                }
            }
        }

        res
    }

    pub fn update_missile(&mut self, width: f32, height: f32) {
        let width = width as i32;
        let height = height as i32;

        if let Some(missile) = self.missile.as_mut() {
            let mut destroy_missile = false;

            for (x, y) in missile.positions_iter(None) {
                if x < 0 || x >= width || y >= height {
                    destroy_missile = true;
                } else if self.landscape.is_not_empty(x, y) {
                    destroy_missile = true;
                    break;
                }
            }

            if destroy_missile {
                self.missile = None;
            }
        }
    }

    /// Increment angle of gun of current tank
    pub fn inc_gun_angle(&mut self, delta: f32) {
        if let Some(tank) = self.tanks.get_mut(self.current_tank) {
            let mut angle = tank.angle + delta;
            if angle > 90.0 {
                angle = 90.0;
            } else if angle < -90.0 {
                angle = -90.0;
            }
            tank.angle = angle;
        }
    }

    #[inline]
    pub fn gun_angle(&self) -> f32 {
        match self.tanks.get(self.current_tank) {
            Some(tank) => tank.angle,
            None => 90.0,
        }
    }

    #[inline]
    pub fn gun_power(&self) -> f32 {
        match self.tanks.get(self.current_tank) {
            Some(tank) => tank.power,
            None => 0.0,
        }
    }

    /// Increment power of gun of current tank
    pub fn inc_gun_power(&mut self, delta: f32) {
        if let Some(tank) = self.tanks.get_mut(self.current_tank) {
            let mut power = tank.power + delta;
            if power > 100.0 {
                power = 100.0;
            } else if power < 0.0 {
                power = 0.0;
            }
            tank.power = power;
        }
    }

    pub fn shoot(&mut self) {
        if self.missile.is_none() {
            if let Some(tank) = self.tanks.get(self.current_tank) {
                let acceleration = Vector2::new(self.wind_power, G);
                self.missile = Some(tank.shoot(acceleration));
            }
        }
    }
}
