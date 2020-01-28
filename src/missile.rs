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

    //    #[inline]
    //    pub fn cur_velocity(&self) -> Vector2 {
    //        self.ballistics.pos_and_velocity().1
    //    }

    pub fn update(&mut self, landscape: &Landscape) -> Option<Point2> {
        let size = landscape.size();
        let borders = (size.0 as i32, size.1 as i32);

        for (x, y) in self.ballistics.positions_iter(None, Some(borders)) {
            if landscape.is_not_empty(x, y) || y >= borders.1 {
                return Some(Point2::new(x as f32, y as f32));
            }
        }

        None
    }
}
