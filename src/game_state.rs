use cgmath::InnerSpace;
use ggez::graphics::Color;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate::explosion::Explosion;
use crate::{Landscape, Missile, Tank, Vector2, G};

pub struct GameState {
    pub rng: ThreadRng,
    pub width: f32,
    pub height: f32,
    pub landscape: Landscape,
    pub wind_power: f32,
    pub tanks: Vec<Tank>,
    pub current_tank: usize,
    pub missile: Option<Missile>,
    pub explosion: Option<Explosion>,
}

impl GameState {
    pub fn new(width: f32, height: f32) -> Result<GameState, String> {
        let mut rng = rand::thread_rng();
        let mut landscape = Landscape::new(width as u16, height as u16)?;
        landscape.set_seed(rng.gen());
        landscape.dx = rng.gen_range(0, width as i32 / 2);
        landscape.generate();

        let tanks = vec![
            Tank::new([100., 50.], Color::from_rgb(245, 71, 32)),
            Tank::new([width - 100., 50.], Color::from_rgb(42, 219, 39)),
        ];

        let mut state = GameState {
            rng,
            width,
            height,
            landscape,
            wind_power: 0.0,
            tanks,
            current_tank: 0,
            missile: None,
            explosion: None,
        };
        state.change_wind();
        Ok(state)
    }

    #[inline]
    pub fn update_landscape_seed(&mut self) {
        self.landscape.set_seed(self.rng.gen());
    }

    #[inline]
    pub fn regenerate_landscape(&mut self) {
        self.landscape.generate();
        for tank in self.tanks.iter_mut() {
            tank.throw_down(Some(50.));
        }
        self.change_wind();
    }

    fn change_wind(&mut self) {
        self.wind_power = (self.rng.gen_range(-10.0_f32, 10.0_f32) * 10.0).round() / 10.0;
    }

    pub fn update(&mut self) {
        self.update_tanks();
        self.update_missile();
        self.update_explosion();
        self.update_landscape();
    }

    fn update_tanks(&mut self) {
        for tank in self.tanks.iter_mut() {
            tank.update(&mut self.landscape)
        }
    }

    fn update_missile(&mut self) {
        if let Some(missile) = self.missile.as_mut() {
            if let Some(pos) = missile.update(&self.landscape) {
                self.missile = None;
                self.explosion = Some(Explosion::new(pos, 50.0));
            }
        }
    }

    fn update_explosion(&mut self) {
        if let Some(explosion) = self.explosion.as_mut() {
            if explosion.update(&mut self.landscape) {
                self.explosion = None;
                self.landscape.subsidence();
            }
        }
    }

    fn update_landscape(&mut self) {
        if self.landscape.update() {
            self.current_tank += 1;
            if self.current_tank >= self.tanks.len() {
                self.current_tank = 0;
            }
            for tank in self.tanks.iter_mut() {
                tank.throw_down(None);
            }
            self.change_wind();
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

    #[inline]
    pub fn missile_speed(&self) -> f32 {
        if let Some(missile) = self.missile.as_ref() {
            let velocity = missile.cur_velocity();
            return velocity.magnitude();
        }
        0.0
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
        if self.missile.is_none() && self.explosion.is_none() && !self.landscape.is_subsidence() {
            if let Some(tank) = self.tanks.get(self.current_tank) {
                let acceleration = Vector2::new(self.wind_power, G);
                self.missile = Some(tank.shoot(acceleration));
            }
        }
    }
}
