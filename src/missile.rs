use crate::{Ballistics, Landscape, Point2, Vector2};
use std::f32::consts::PI;

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
            ballistics: Ballistics::new(pos, velocity, acceleration, TIME_SCALE),
        }
    }

    #[inline]
    pub fn cur_pos(&self) -> Point2 {
        self.ballistics.cur_pos()
    }

    #[inline]
    pub fn cur_velocity(&self) -> Vector2 {
        self.ballistics.pos_and_velocity().1
    }

    pub fn update(&mut self, landscape: &Landscape) -> bool {
        let size = landscape.size();
        let borders = (size.0 as i32, size.1 as i32);

        //        let mut destroy_missile = false;

        for (x, y) in self.ballistics.positions_iter(None, Some(borders)) {
            if landscape.is_not_empty(x, y) {
                return true;
            }
        }

        false
    }
}
