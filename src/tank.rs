use std::f32::consts::PI;

use ggez::graphics::{Color, Rect};

use crate::{Missile, Point2, Vector2};

const TANK_SIZE: f32 = 41.;
const GUN_SIZE: f32 = 21.;
const POWER_SCALE: f32 = 300. / 100.;

pub struct Tank {
    pub rect: Rect,
    pub color: Color,
    pub angle: f32,
    pub power: f32,
}

impl Tank {
    pub fn new(top_left: Point2, color: Color) -> Tank {
        let rect = Rect::new(top_left.x, top_left.y, TANK_SIZE, TANK_SIZE);
        Tank {
            rect,
            color,
            angle: 0.0,
            power: 40.0,
        }
    }

    #[inline]
    pub fn bottom_left(&self) -> Point2 {
        [self.rect.x, self.rect.bottom() - 1.].into()
    }

    #[inline]
    pub fn top_left(&self) -> Point2 {
        [self.rect.x, self.rect.y].into()
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.rect.w
    }

    /// Move this tank and clamp the positions to prevent over/underflow.
    #[inline]
    pub fn offset(&mut self, x: f32, y: f32) {
        self.rect.translate([x, y]);
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
}

//impl Draw for Tank {
//    fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
//        canvas.set_draw_color(self.color);
//        canvas.fill_rect(self.rect)
//    }
//}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottom_left() {
        let tank = Tank::new([0., 0.].into(), (0, 0, 0).into());
        assert_eq!(tank.bottom_left(), [0., TANK_SIZE - 1.].into());
    }
}
