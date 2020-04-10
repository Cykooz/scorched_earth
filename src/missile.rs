use std::f32::consts::PI;

use crate::ballistics::Ballistics;
use crate::landscape::Landscape;
use crate::types::{Point2, Vector2};

const TIME_SCALE: f32 = 3.0;

#[derive(Debug, Clone, Copy)]
pub struct Missile {
    ballistics: Ballistics,
}

impl Missile {
    pub fn new(pos: Point2, angle: f32, power: f32, acceleration: Vector2) -> Missile {
        let rad = angle * PI / 180.;
        let velocity: Vector2 = Vector2::new(rad.sin(), -rad.cos()) * power;

        Missile {
            ballistics: Ballistics::new(pos, velocity, acceleration).time_scale(TIME_SCALE),
        }
    }

    #[inline]
    pub fn cur_pos(&self) -> Point2 {
        self.ballistics.cur_pos()
    }

    pub fn update<F>(&mut self, borders: (i32, i32), has_collision: F) -> Option<Point2>
    where
        F: Fn(i32, i32) -> bool,
    {
        for (x, y) in self.ballistics.positions_iter(None, Some(borders)) {
            if has_collision(x, y) || y >= borders.1 {
                return Some(Point2::new(x as f32, y as f32));
            }
        }

        None
    }
}
