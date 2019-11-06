use crate::{Point2, Vector2};
use std::f32::consts::PI;
use std::time::Instant;

pub const G: f32 = 9.8;
const TIME_SCALE: f32 = 3.0;

pub struct Missile {
    start_pos: Point2,
    start_velocity: Vector2,
    pub cur_pos: Point2,
    created: Instant,
    last_updated: f32,
    acceleration: Vector2,
}

impl Missile {
    pub fn new(pos: Point2, angle: f32, power: f32, acceleration: Vector2) -> Missile {
        let rad = angle * PI / 180.;
        let velocity: Vector2 = Vector2::new(rad.sin(), -rad.cos()) * power;

        Missile {
            start_pos: pos,
            start_velocity: velocity,
            cur_pos: pos,
            created: Instant::now(),
            last_updated: 0.0,
            acceleration,
        }
    }

    #[inline]
    fn velocity(&self, time: f32) -> Vector2 {
        self.start_velocity + self.acceleration * time
    }

    #[inline]
    fn pos(&self, time: f32) -> Point2 {
        let velocity = self.velocity(time);
        self.start_pos + velocity * time
    }

    #[inline]
    pub fn pos_i32(&self, time: f32) -> (i32, i32) {
        let pos = self.pos(time * TIME_SCALE);
        (pos.x.floor() as i32, pos.y.floor() as i32)
    }

    #[inline]
    pub fn pos_and_velocity(&self, time: f32) -> (Point2, Vector2) {
        let velocity = self.velocity(time);
        (self.start_pos + velocity * time, velocity)
    }

    //    pub fn update(&mut self) -> bool {
    //        let time = self.created.elapsed().as_secs_f32() * TIME_SCALE;
    //        self.last_updated = time;
    //        let cur_pos = self.pos(time);
    //        let cur_pos = (cur_pos.x as i32, cur_pos.y as i32);
    //        if self.cur_pos != cur_pos {
    //            self.cur_pos = cur_pos;
    //            return true;
    //        }
    //        false
    //    }

    pub fn positions_iter(&mut self, end_time: Option<f32>) -> MissilePosIterator {
        let start_time = self.last_updated;
        let end_time =
            end_time.unwrap_or_else(|| self.created.elapsed().as_secs_f32()) * TIME_SCALE;

        let (_, start_velocity) = self.pos_and_velocity(start_time);
        let (_, end_velocity) = self.pos_and_velocity(end_time);
        let max_velocity = start_velocity
            .x
            .abs()
            .max(start_velocity.y.abs())
            .max(end_velocity.x.abs())
            .max(end_velocity.y.abs());

        let time_period = end_time - start_time;

        let time_step = if max_velocity == 0.0 {
            time_period
        } else {
            1.0 / (2.0 * max_velocity)
        };
        let last_pos = (self.cur_pos.x.floor() as i32, self.cur_pos.y.floor() as i32);

        MissilePosIterator {
            missile: self,
            end_time,
            time_step,
            last_time: start_time,
            last_pos,
        }
    }
}

pub struct MissilePosIterator<'a> {
    missile: &'a mut Missile,
    end_time: f32,
    time_step: f32,
    last_time: f32,
    last_pos: (i32, i32),
}

impl<'a> Iterator for MissilePosIterator<'a> {
    type Item = (i32, i32);

    fn next(&mut self) -> Option<Self::Item> {
        let mut next_time = self.last_time;

        while next_time <= self.end_time {
            next_time += self.time_step;
            let clamped_time = self.end_time.min(next_time);

            let pos = self.missile.pos(clamped_time);
            let pos_i32 = (pos.x.floor() as i32, pos.y.floor() as i32);
            if pos_i32 != self.last_pos {
                self.last_time = next_time;
                self.last_pos = pos_i32;
                self.missile.last_updated = clamped_time;
                self.missile.cur_pos = pos;
                return Some(pos_i32);
            }
        }

        self.missile.cur_pos = self.missile.pos(self.end_time);
        self.missile.last_updated = self.end_time;
        None
    }
}

//#[inline]
//fn get_time_step(velocity: f32, pos: f32, left_time: f32) -> f32 {
//    if velocity == 0.0 {
//        left_time
//    } else if velocity.is_sign_positive() ^ pos.is_sign_positive() {
//        (pos.fract().abs() + 0.5) / velocity
//    } else {
//        (1.5 - pos.fract().abs()) / velocity
//    }
//    .abs()
//}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_positions_iter_horizontal() {
        let acceleration = Vector2::new(0.0, 0.0);
        let angle = 90.0;
        let power = 100.0;
        let mut missile = Missile::new([0., 0.].into(), angle, power, acceleration);

        assert_eq!(missile.pos_i32(10.0), (3000, 0));

        let mut pos_iterator = missile.positions_iter(Some(10.0));
        for x in 1..=3000 {
            assert_eq!(pos_iterator.next(), Some((x, 0)));
        }
        assert_eq!(pos_iterator.next(), None);
        assert_eq!(missile.last_updated, 10.0 * TIME_SCALE);
        assert_eq!(missile.cur_pos.x, 3000.0);

        let mut pos_iterator = missile.positions_iter(Some(20.0));
        for x in 3001..=6000 {
            assert_eq!(pos_iterator.next(), Some((x, 0)));
        }
        assert_eq!(pos_iterator.next(), None);
        assert_eq!(missile.last_updated, 20.0 * TIME_SCALE);
        assert_eq!(missile.cur_pos.x, 6000.0);
    }

    #[test]
    fn test_positions_iter_vertical() {
        let acceleration = Vector2::new(0.0, 0.0);
        let angle = 0.0;
        let power = 100.0;
        let mut missile = Missile::new([0., 0.].into(), angle, power, acceleration);

        assert_eq!(missile.pos(10.0 * TIME_SCALE).y, -3000.0);

        let mut pos_iterator = missile.positions_iter(Some(10.0));
        for y in 1..=2999 {
            assert_eq!(pos_iterator.next(), Some((0, -y)));
        }
        assert_eq!(pos_iterator.next(), Some((0, -3000)));
        assert_eq!(pos_iterator.next(), None);
        assert_eq!(missile.last_updated, 10.0 * TIME_SCALE);
        assert_eq!(missile.cur_pos.y, -3000.0);
    }
}
