use std::f32::consts::PI;

use cgmath::{Basis2, Deg, Rotation, Rotation2};
use ggez::{self, graphics, GameResult};

use crate::ballistics::Ballistics;
use crate::geometry::Ellipse;
use crate::landscape::Landscape;
use crate::missile::Missile;
use crate::shaders;
use crate::types::{Point2, Vector2};
use crate::world::World;
use crate::G;

const TANK_SIZE: f32 = 41.;
const GUN_SIZE: f32 = 21.;
const POWER_SCALE: f32 = 300. / 100.;
const TIME_SCALE: f32 = 3.0;

#[derive(Debug, Clone)]
struct TankThrowing {
    start_height: f32,
    ballistics: Ballistics,
}

#[derive(Debug, Clone)]
pub struct Tank {
    pub player_number: u8,
    pub rect: graphics::Rect,
    body_bounds: Vec<Ellipse>,
    gun_bounds: Vec<Ellipse>,
    hue_offset: shaders::HueOffset,
    pub angle: f32,
    pub power: f32,
    pub health: u8,
    pub dead: bool,
    throwing: Option<TankThrowing>,
}

#[derive(Debug, Clone, Copy)]
pub enum TankState {
    Placed(f32),
    Dropped,
}

// impl TankState {
//     #[inline]
//     pub fn is_placed(self) -> bool {
//         self == TankState::Placed
//     }
// }

impl Tank {
    pub fn new<P, H>(player_number: u8, top_left: P, hue_offset: H) -> Tank
    where
        P: Into<Point2>,
        H: Into<f32>,
    {
        let top_left: Point2 = top_left.into();
        let rect = graphics::Rect::new(top_left.x, top_left.y, TANK_SIZE, TANK_SIZE);
        let body_bounds = vec![
            Ellipse::new((20.5, 26.), 9.5, 9.),    // top bound
            Ellipse::new((11., 33.5), 10., 6.5),   // left bound
            Ellipse::new((30., 33.5), 10., 6.5),   // right bound
            Ellipse::new((20.5, 33.5), 19.5, 7.5), // center bound
        ];
        let gun_bounds = vec![
            Ellipse::new((20.5, 6.5), 2.5, 5.),
            Ellipse::new((20.5, 15.5), 2., 8.),
        ];
        let mut tank = Tank {
            player_number,
            rect,
            body_bounds,
            gun_bounds,
            hue_offset: shaders::HueOffset::new(hue_offset),
            angle: 0.0,
            power: 40.0,
            health: 100,
            dead: false,
            throwing: None,
        };
        tank.throw_down(None);
        tank
    }

    //    #[inline]
    //    pub fn bottom_left(&self) -> Point2 {
    //        [self.rect.x, self.rect.bottom() - 1.].into()
    //    }

    #[inline]
    pub fn top_left(&self) -> Point2 {
        [self.rect.x, self.rect.y].into()
    }

    #[inline]
    pub fn center(&self) -> Point2 {
        Point2::new(
            self.rect.x + self.rect.w / 2.,
            self.rect.y + self.rect.h / 2.,
        )
    }

    pub fn gun_barrel_pos(&self) -> Point2 {
        let rad = self.angle * PI / 180.0;
        let gun_vec = Vector2::new(GUN_SIZE * rad.sin(), -GUN_SIZE * rad.cos());
        self.center() + gun_vec
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
        let mut start_height = self.rect.bottom() - 1.;

        if let Some(throwing) = self.throwing.as_mut() {
            start_height = throwing.start_height;
            let height = landscape.size().1 as i32;
            let tank_width = self.rect.w;
            let max_empty_count = (0.3 * tank_width).round() as usize;
            let mut offset: f32 = 0.0;

            for (x, y) in throwing.ballistics.positions_iter(None, None) {
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
            let cur_height = self.rect.bottom() - 1.;
            let path_len = cur_height - start_height;
            TankState::Placed(path_len)
        } else {
            TankState::Dropped
        }
    }

    pub fn throw_down(&mut self, top: Option<f32>) {
        if let Some(top) = top {
            self.rect.y = top;
        }

        let start_height = self.rect.bottom() - 1.;
        self.throwing = Some(TankThrowing {
            start_height,
            ballistics: Ballistics::new([self.rect.x, start_height], [0., 0.], [0., G])
                .time_scale(TIME_SCALE),
        });
    }

    pub fn draw(&self, ctx: &mut ggez::Context, world: &World) -> GameResult {
        let _lock = graphics::use_shader(ctx, &world.hue_shader);
        world.hue_shader.send(ctx, self.hue_offset.into())?;

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

    #[inline]
    pub fn damage(&mut self, v: u8) {
        self.health = self.health.saturating_sub(v);
    }

    /// Returns `true` if given point locates inside of tank's body or gun.
    pub fn has_collision<P: Into<Point2>>(&self, point: P) -> bool {
        let local_point = point.into() - Vector2::new(self.rect.x, self.rect.y);
        if self
            .body_bounds
            .iter()
            .any(|b| b.point_position(local_point) <= 0.)
        {
            return true;
        }

        let tank_center = Vector2::new(TANK_SIZE / 2., TANK_SIZE / 2.);
        let rotation: Basis2<_> = Rotation2::from_angle(Deg(-self.angle));
        let rotated_point = rotation.rotate_point(local_point - tank_center) + tank_center;
        self.gun_bounds
            .iter()
            .any(|b| b.point_position(rotated_point) <= 0.)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_collision() {
        let mut tank = Tank::new(1, (10.0, 20.0), 0.);

        let inner_points = [
            (20., 27.), // body center
            (4., 32.),  // body left
            (37., 32.), // body right
            (20., 40.), // body bottom
            (20., 18.), // body top
            (20., 2.),  // gun top
            (20., 13.), // gun middle
        ];
        for point in inner_points.iter() {
            assert!(
                tank.has_collision((10. + point.0, 20. + point.1)),
                format!("point=({}, {})", point.0, point.1)
            );
        }

        // Rotated gun
        tank.angle = 60.;
        let inner_points = [
            (34., 11.), // gun top
            (24., 18.), // gun middle
        ];
        for point in inner_points.iter() {
            assert!(
                tank.has_collision((10. + point.0, 20. + point.1)),
                format!("point=({}, {})", point.0, point.1)
            );
        }

        tank.angle = -45.;
        let inner_points = [
            (8., 8.),   // gun top
            (15., 15.), // gun middle
        ];
        for point in inner_points.iter() {
            assert!(
                tank.has_collision((10. + point.0, 20. + point.1)),
                format!("point=({}, {})", point.0, point.1)
            );
        }
    }
}
