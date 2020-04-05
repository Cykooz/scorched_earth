use std::time::Instant;

use ggez::graphics::Rect;
use itertools::Itertools;

use crate::geometry::Circle;
use crate::landscape::Landscape;
use crate::types::Point2;

const SPEED: f32 = 150.0;

#[derive(Debug, Clone, Copy)]
pub struct Explosion {
    created: Instant,
    pub pos: Point2,
    max_radius: f32,
    pub cur_radius: f32,
    pub cur_opacity: f32,
    landscape_updated: bool,
}

impl Explosion {
    pub fn new(pos: Point2, max_radius: f32) -> Self {
        Explosion {
            created: Instant::now(),
            pos,
            max_radius,
            cur_radius: 0.0,
            cur_opacity: 1.0,
            landscape_updated: false,
        }
    }

    #[inline]
    pub fn is_life(self) -> bool {
        self.cur_opacity > 0.0
    }

    /// Returns `true` if explosion has finished.
    pub fn update(&mut self, landscape: &mut Landscape) -> bool {
        if self.is_life() {
            let time = self.created.elapsed().as_secs_f32();
            let radius = time * SPEED;
            self.cur_opacity = if radius <= self.max_radius {
                1.0
            } else {
                0.0_f32.max((2.0 * self.max_radius - radius) / self.max_radius)
            };
            self.cur_radius = radius.min(self.max_radius);

            if !self.landscape_updated && radius >= self.max_radius {
                self.destroy_landscape(landscape);
            }
        }

        !self.is_life()
    }

    fn destroy_landscape(&mut self, landscape: &mut Landscape) {
        let circle = line_drawing::BresenhamCircle::new(
            self.pos.x as i32,
            self.pos.y as i32,
            self.max_radius as i32 - 1,
        );
        for points_iter in &circle.chunks(4) {
            let points: Vec<(i32, i32)> = points_iter.step_by(2).collect();
            if points.len() != 2 {
                break;
            }
            let (x1, y1) = points[0];
            let (x2, y2) = points[1];
            let x = x1.min(x2).max(0);
            let len = (x1.max(x2).max(0) - x) as u16;
            if len == 0 {
                continue;
            }
            for &y in [y1, y2].iter() {
                if let Some(pixels) = landscape.get_pixels_line_mut((x, y), len) {
                    pixels.iter_mut().for_each(|c| *c = 0);
                }
            }
        }
        landscape.set_changed();
        self.landscape_updated = true;
    }

    pub fn get_intersection_percents(&self, bound: Rect) -> u8 {
        let bound_area = bound.w * bound.h;
        if bound_area > 0.0 {
            let circle = Circle::new(self.pos, self.max_radius);
            let intersection_area = circle.area_of_rect_intersection(bound);
            if intersection_area > 0.0 {
                let percents = 100.0 * intersection_area / bound_area;
                return percents.min(100.0).max(0.0) as u8;
            }
        }
        0
    }
}
