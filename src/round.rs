use ggez::audio::SoundSource;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::explosion::Explosion;
use crate::landscape::Landscape;
use crate::missile::Missile;
use crate::tank::{Tank, TankState};
use crate::types::Vector2;
use crate::world::World;
use crate::{G, MAX_PLAYERS_COUNT};

/// A damage per one pixel of height with which tank was dropped.
const TANK_THROWING_DAMAGE_POWER: f32 = 0.1;

#[derive(Debug, Clone)]
pub enum GameState {
    TanksThrowing,
    Aiming,
    FlyingOfMissile(Missile),
    Exploding(Vec<Explosion>),
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
    pub number_of_iteration: usize,
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
                let hue_offset = (player_number as u16 - 1) * (360 / MAX_PLAYERS_COUNT as u16);
                Tank::new(player_number, [x, 50.], hue_offset)
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
            state: GameState::TanksThrowing,
            number_of_iteration: 0,
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

    pub fn update(&mut self, world: &mut World) -> &GameState {
        self.update_tanks(world);
        self.update_missile(world);
        self.update_explosions();
        self.update_landscape();
        &self.state
    }

    fn update_tanks(&mut self, world: &mut World) {
        if let GameState::TanksThrowing = self.state {
            let mut all_placed = true;
            let live_tanks = self.tanks.iter_mut().filter(|t| !t.dead);
            for tank in live_tanks {
                let tank_state = tank.update(&mut self.landscape);
                if let TankState::Placed(path_len) = tank_state {
                    if self.number_of_iteration > 0 {
                        let damage_value: u8 =
                            (path_len * TANK_THROWING_DAMAGE_POWER).min(255.).round() as u8;
                        tank.damage(damage_value);
                    }
                } else {
                    all_placed = false;
                }
            }

            if all_placed {
                let explosions = self.remove_destroyed_tanks(world);

                self.state = if !explosions.is_empty() {
                    world.explosion_sound.play().unwrap();
                    GameState::Exploding(explosions)
                } else if self.live_tanks_count() <= 1 {
                    GameState::Finish
                } else {
                    if self.number_of_iteration > 0 {
                        self.switch_current_tank();
                    }
                    // self.change_wind();
                    self.number_of_iteration = self.number_of_iteration.saturating_add(1);
                    GameState::Aiming
                };
            }
        }
    }

    fn update_missile(&mut self, world: &mut World) {
        if let GameState::FlyingOfMissile(ref mut missile) = self.state {
            let landscape = &self.landscape;
            let tanks = &self.tanks;
            let size = landscape.size();
            let borders = (size.0 as i32, size.1 as i32);
            let hit_point = missile.update(borders, |x, y| {
                landscape.is_not_empty(x, y)
                    || tanks
                        .iter()
                        .filter(|t| !t.dead)
                        .any(|t| t.has_collision((x as f32, y as f32)))
            });
            if let Some(pos) = hit_point {
                world.explosion_sound.play().unwrap();
                self.state = GameState::Exploding(vec![Explosion::new(pos, 50.0)]);
            }
        }
    }

    fn update_explosions(&mut self) {
        if let GameState::Exploding(ref mut explosions) = self.state {
            let landscape = &mut self.landscape;
            let count_not_finished_explosions = explosions
                .iter_mut()
                .filter_map(|e| {
                    if e.update(landscape) {
                        return None;
                    };
                    Some(())
                })
                .count();

            if count_not_finished_explosions == 0 {
                // Check intersection of explosion with tanks and decrease its health.
                let live_tanks = self.tanks.iter_mut().filter(|t| !t.dead);
                for tank in live_tanks {
                    for e in explosions.iter() {
                        let percents = e.get_intersection_percents(tank.rect);
                        tank.damage(percents);
                    }
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
                self.state = GameState::TanksThrowing;
            }
        }
    }

    fn change_wind(&mut self) {
        self.wind_power = (self.rng.gen_range(-10.0_f32, 10.0_f32) * 10.0).round() / 10.0;
    }

    /// Mark all destroyed tanks as "dead", add some money to current player
    /// and returns vector of tanks explosions.
    fn remove_destroyed_tanks(&mut self, world: &mut World) -> Vec<Explosion> {
        let explosions: Vec<Explosion> = self
            .tanks
            .iter_mut()
            .filter(|t| t.health == 0 && !t.dead)
            .map(|t| {
                t.dead = true;
                Explosion::new(t.center(), 50.0)
            })
            .collect();

        let current_player_number = self.player_number() as usize;
        let player = &mut world.players[current_player_number - 1];
        let count_of_destroyed = explosions.len() as u32;
        player.money = player.money.saturating_add(200 * count_of_destroyed);

        explosions
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

    #[inline]
    pub fn explosions(&self) -> Option<impl Iterator<Item = &Explosion>> {
        match self.state {
            GameState::Exploding(ref explosions) => Some(explosions.iter().filter(|e| e.is_life())),
            _ => None,
        }
    }

    /// Increment angle of gun of current tank
    pub fn inc_gun_angle(&mut self, delta: f32) {
        if let Some(tank) = self.tanks.get_mut(self.current_tank) {
            let angle = tank.angle + delta;
            tank.angle = angle.min(90.).max(-90.);
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
            let power = tank.power + delta;
            tank.power = power.min(100.).max(0.);
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
