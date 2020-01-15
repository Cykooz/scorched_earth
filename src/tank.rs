use std::f32::consts::PI;

use ggez::{self, graphics, GameResult};

use crate::ballistics::Ballistics;
use crate::landscape::Landscape;
use crate::missile::Missile;
use crate::types::{Point2, Vector2};
use crate::world::World;
use crate::G;

const TANK_SIZE: f32 = 41.;
const GUN_SIZE: f32 = 21.;
const POWER_SCALE: f32 = 300. / 100.;
const TIME_SCALE: f32 = 3.0;

#[derive(Debug, Clone, Copy)]
pub struct Tank {
    pub rect: graphics::Rect,
    pub color: graphics::Color,
    pub angle: f32,
    pub power: f32,
    pub health: u8,
    throwing: Option<Ballistics>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TankState {
    Placed,
    Dropped,
}

impl TankState {
    #[inline]
    pub fn is_placed(self) -> bool {
        self == TankState::Placed
    }
}

impl Tank {
    pub fn new<P>(top_left: P, color: graphics::Color) -> Tank
    where
        P: Into<Point2>,
    {
        let top_left: Point2 = top_left.into();
        let rect = graphics::Rect::new(top_left.x, top_left.y, TANK_SIZE, TANK_SIZE);
        let mut tank = Tank {
            rect,
            color,
            angle: 0.0,
            power: 40.0,
            health: 100,
            throwing: None,
        };
        tank.throw_down(None);
        tank
    }

    #[inline]
    pub fn bottom_left(&self) -> Point2 {
        [self.rect.x, self.rect.bottom() - 1.].into()
    }

    #[inline]
    pub fn top_left(&self) -> Point2 {
        [self.rect.x, self.rect.y].into()
    }

    pub fn gun_barrel_pos(&self) -> Point2 {
        let center = Point2::new(
            self.rect.x + self.rect.w / 2.,
            self.rect.y + self.rect.h / 2.,
        );
        let rad = self.angle * PI / 180.0;
        let gun_vec = Vector2::new(GUN_SIZE * rad.sin(), -GUN_SIZE * rad.cos());
        center + gun_vec
    }

    pub fn shoot(&self, acceleration: Vector2) -> Missile {
        Missile::new(
            self.gun_barrel_pos(),
            self.angle,
            self.power * POWER_SCALE,
            acceleration,
        )
    }

    pub fn update(&mut self, landscape: &mut Landscape) -> TankState {
        if let Some(ballistics) = self.throwing.as_mut() {
            let height = landscape.size().1 as i32;
            let tank_width = self.rect.w;
            let max_empty_count = (0.3 * tank_width).round() as usize;
            let mut offset: f32 = 0.0;

            for (x, y) in ballistics.positions_iter(None, None) {
                if y >= height {
                    self.throwing = None;
                    break;
                }

                let pixels_under_tank = landscape.get_pixels_line_mut((x, y), tank_width as u16);
                if let Some(pixels) = pixels_under_tank {
                    let empty_count = bytecount::count(pixels, 0);
                    if empty_count > max_empty_count {
                        if empty_count < tank_width as usize {
                            // Landscape under tank is not empty - clear it
                            pixels.iter_mut().for_each(|c| *c = 0);
                            landscape.set_changed();
                        }
                        // Get down tank
                        offset += 1.0;
                    } else {
                        self.throwing = None;
                        break;
                    }
                }
            }

            if offset > 0. {
                self.rect.translate([0., offset]);
            }
        }

        if self.throwing.is_none() {
            TankState::Placed
        } else {
            TankState::Dropped
        }
    }

    pub fn throw_down(&mut self, top: Option<f32>) {
        if let Some(top) = top {
            self.rect.y = top;
        }

        self.throwing = Some(
            Ballistics::new([self.rect.x, self.rect.bottom() - 1.], [0., 0.], [0., G])
                .time_scale(TIME_SCALE),
        );
    }

    pub fn draw(&self, ctx: &mut ggez::Context, world: &World) -> GameResult {
        let pos = self.top_left();
        let gun_params = graphics::DrawParam::new()
            .dest(pos + Vector2::new(20.5, 20.5))
            .offset(Point2::new(0.5, 0.5))
            .rotation(std::f32::consts::PI * self.angle / 180.0);
        graphics::draw(ctx, &world.gun_image, gun_params)?;
        let tank_params = graphics::DrawParam::new().dest(pos);
        graphics::draw(ctx, &world.tank_image, tank_params)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottom_left() {
        let tank = Tank::new([0., 0.], (0, 0, 0).into());
        assert_eq!(tank.bottom_left(), [0., TANK_SIZE - 1.].into());
    }
}
