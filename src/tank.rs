use std::f32::consts::PI;

use ggez::graphics::{Color, Rect};

use crate::{Ballistics, Landscape, Missile, Point2, Vector2, G};

const TANK_SIZE: f32 = 41.;
const GUN_SIZE: f32 = 21.;
const POWER_SCALE: f32 = 300. / 100.;
const TIME_SCALE: f32 = 3.0;

#[derive(Debug, Clone, Copy)]
pub struct Tank {
    pub rect: Rect,
    pub color: Color,
    pub angle: f32,
    pub power: f32,
    throwing: Option<Ballistics>,
}

impl Tank {
    pub fn new(top_left: Point2, color: Color) -> Tank {
        let rect = Rect::new(top_left.x, top_left.y, TANK_SIZE, TANK_SIZE);
        let mut tank = Tank {
            rect,
            color,
            angle: 0.0,
            power: 40.0,
            throwing: None,
        };
        tank.throw_down();
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

    pub fn update(&mut self, landscape: &mut Landscape) -> bool {
        let height = landscape.size().1 as i32;
        let tank_width = self.rect.w as u32;
        let mut res = false;
        let mut offset: f32 = 0.0;

        if let Some(ballistics) = self.throwing.as_mut() {
            for (x, y) in ballistics.positions_iter(None, None) {
                if y >= height {
                    self.throwing = None;
                    break;
                }

                let pixels_under_tank = landscape.get_pixels_line_mut((x, y + 1), tank_width);
                if let Some(pixels) = pixels_under_tank {
                    let empty_count = bytecount::count(pixels, 0);
                    if empty_count > 0 {
                        if empty_count < tank_width as usize {
                            // Landscape under tank is not empty - clear it
                            pixels.iter_mut().for_each(|c| *c = 0);
                            res = true;
                        }
                        // Get down tank
                        offset += 1.0;
                    } else {
                        self.throwing = None;
                        break;
                    }
                }
            }
        }
        if offset > 0. {
            self.rect.translate([0., offset]);
        }

        res
    }

    pub fn throw_down(&mut self) {
        if self.throwing.is_none() {
            self.throwing = Some(Ballistics::new(
                [self.rect.x, self.rect.bottom() - 1.],
                [0., 0.],
                [0., G],
                TIME_SCALE,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottom_left() {
        let tank = Tank::new([0., 0.].into(), (0, 0, 0).into());
        assert_eq!(tank.bottom_left(), [0., TANK_SIZE - 1.].into());
    }
}
