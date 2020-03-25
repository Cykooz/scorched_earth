use ggez::audio::SoundSource;
use ggez::graphics::Color;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::explosion::Explosion;
use crate::landscape::Landscape;
use crate::missile::Missile;
use crate::tank::Tank;
use crate::types::Vector2;
use crate::world::World;
use crate::G;

#[derive(Debug, Clone, Copy)]
pub enum ThrowingNumber {
    First,
    NotFirst,
}

#[derive(Debug, Clone, Copy)]
pub enum GameState {
    TanksThrowing(ThrowingNumber),
    Aiming,
    FlyingOfMissile(Missile),
    Exploding(Explosion),
    Subsidence,
    Finish,
}

pub struct Round {
    pub rng: ThreadRng,
    pub width: f32,
    pub height: f32,
    pub landscape: Landscape,
    pub wind_power: f32,
    pub tanks: Vec<Tank>,
    pub current_tank: usize,
    pub state: GameState,
}

impl Round {
    pub fn new(width: u16, height: u16, count_of_tanks: u8) -> Result<Round, String> {
        let mut rng = rand::thread_rng();
        let mut landscape = Landscape::new(width, height)?;
        landscape.set_seed(rng.gen());
        landscape.dx = rng.gen_range(0, width as i32 / 2);
        landscape.generate();

        let size_between_tanks = (width as f32 - 200.) / (count_of_tanks - 1) as f32;

        let mut player_numbers: Vec<u8> = (1..=count_of_tanks).collect();
        player_numbers.shuffle(&mut rng);

        let tanks: Vec<Tank> = player_numbers
            .iter()
            .enumerate()
            .map(|(i, &player_number)| {
                let x = 100. + size_between_tanks * i as f32;
                Tank::new(player_number, [x, 50.], Color::from_rgb(245, 71, 32))
            })
            .collect();

        let mut round = Round {
            rng,
            width: width as f32,
            height: height as f32,
            landscape,
            wind_power: 0.0,
            tanks,
            current_tank: 0,
            state: GameState::TanksThrowing(ThrowingNumber::First),
        };
        round.change_wind();
        Ok(round)
    }

    //    #[inline]
    //    pub fn update_landscape_seed(&mut self) {
    //        self.landscape.set_seed(self.rng.gen());
    //    }
    //
    //    #[inline]
    //    pub fn regenerate_landscape(&mut self) {
    //        self.landscape.generate();
    //        for tank in self.tanks.iter_mut() {
    //            tank.health = 100;
    //            tank.throw_down(Some(50.));
    //        }
    //        self.change_wind();
    //        self.state = GameState::TanksThrowing;
    //    }

    fn change_wind(&mut self) {
        self.wind_power = (self.rng.gen_range(-10.0_f32, 10.0_f32) * 10.0).round() / 10.0;
    }

    /// Mark all destroyed tanks as "dead" and add some money to current player.
    fn remove_destroyed_tanks(&mut self, world: &mut World) {
        let current_player_number = self.player_number();
        let mut count_of_destroyed: u32 = 0;
        self.tanks
            .iter_mut()
            .filter(|t| t.health == 0 && !t.dead)
            .for_each(|t| {
                t.dead = true;
                count_of_destroyed += 1;
            });

        let player = &mut world.players[current_player_number as usize - 1];
        player.money = player.money.saturating_add(200 * count_of_destroyed);
    }

    fn switch_current_tank(&mut self) {
        let mut current_tank = self.current_tank;
        for _ in 0..self.tanks.len() {
            current_tank += 1;
            if current_tank >= self.tanks.len() {
                current_tank = 0;
            }
            if !self.tanks[current_tank].dead {
                self.current_tank = current_tank;
                return;
            }
        }
    }

    #[inline]
    pub fn live_tanks(&self) -> impl Iterator<Item = &Tank> {
        self.tanks.iter().filter(|t| !t.dead)
    }

    #[inline]
    pub fn live_tanks_count(&self) -> usize {
        self.live_tanks().count()
    }

    pub fn update(&mut self, world: &mut World) -> GameState {
        self.update_tanks(world);
        self.update_missile(world);
        self.update_explosion();
        self.update_landscape();
        self.state
    }

    fn update_tanks(&mut self, world: &mut World) {
        if let GameState::TanksThrowing(number) = self.state {
            let mut all_placed = true;
            let live_tanks = self.tanks.iter_mut().filter(|t| !t.dead);
            for tank in live_tanks {
                all_placed &= tank.update(&mut self.landscape).is_placed();
            }

            if all_placed {
                if let ThrowingNumber::NotFirst = number {
                    self.remove_destroyed_tanks(world);
                    self.switch_current_tank();
                }

                self.state = if self.live_tanks_count() > 1 {
                    GameState::Aiming
                } else {
                    GameState::Finish
                };
            }
        }
    }

    fn update_missile(&mut self, world: &mut World) {
        if let GameState::FlyingOfMissile(ref mut missile) = self.state {
            if let Some(pos) = missile.update(&self.landscape) {
                world.explosion_sound.play().unwrap();
                self.state = GameState::Exploding(Explosion::new(pos, 50.0));
            }
        }
    }

    fn update_explosion(&mut self) {
        if let GameState::Exploding(ref mut explosion) = self.state {
            if explosion.update(&mut self.landscape) {
                // Check intersection of explosion with tanks and decrease its health.
                let live_tanks = self.tanks.iter_mut().filter(|t| !t.dead);
                for tank in live_tanks {
                    let percents = explosion.get_intersection_percents(tank.rect);
                    tank.health = tank.health.saturating_sub(percents);
                }

                self.landscape.subsidence();
                self.state = GameState::Subsidence;
            }
        }
    }

    fn update_landscape(&mut self) {
        if let GameState::Subsidence = self.state {
            if self.landscape.update() {
                let live_tanks = self.tanks.iter_mut().filter(|t| !t.dead);
                for tank in live_tanks {
                    tank.throw_down(None);
                }
                self.change_wind();
                self.state = GameState::TanksThrowing(ThrowingNumber::NotFirst);
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
    pub fn player_number(&self) -> u8 {
        self.tanks
            .get(self.current_tank)
            .map_or_else(|| 1, |tank| tank.player_number)
    }

    #[inline]
    pub fn gun_angle(&self) -> f32 {
        self.tanks
            .get(self.current_tank)
            .map_or_else(|| 90.0, |tank| tank.angle)
    }

    #[inline]
    pub fn gun_power(&self) -> f32 {
        self.tanks
            .get(self.current_tank)
            .map_or_else(|| 0.0, |tank| tank.power)
    }

    #[inline]
    pub fn health(&self) -> u8 {
        self.tanks
            .get(self.current_tank)
            .map_or_else(|| 0, |tank| tank.health)
    }

    //    #[inline]
    //    pub fn missile_speed(&self) -> f32 {
    //        if let GameState::FlyingOfMissile(ref missile) = self.state {
    //            let velocity = missile.cur_velocity();
    //            return velocity.magnitude();
    //        }
    //        0.0
    //    }

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

    pub fn shoot(&mut self, world: &mut World) {
        if let GameState::Aiming = self.state {
            if let Some(tank) = self.tanks.get(self.current_tank) {
                world.tank_fire_sound.play().unwrap();
                let acceleration = Vector2::new(self.wind_power, G);
                self.state = GameState::FlyingOfMissile(tank.shoot(acceleration));
            }
        }
    }
}
