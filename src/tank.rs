use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::Draw;
use sdl2::rect::{Point, Rect};

const TANK_SIZE: u32 = 42;

pub struct Tank {
    rect: Rect,
    color: Color,
}

impl Tank {
    pub fn new(top_left: Point, color: Color) -> Tank {
        let rect = Rect::new(top_left.x(), top_left.y(), TANK_SIZE, TANK_SIZE);
        Tank { rect, color }
    }

    #[inline]
    pub fn bottom_left(&self) -> Point {
        Point::new(self.rect.left(), self.rect.bottom() - 1)
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.rect.width()
    }

    /// Move this tank and clamp the positions to prevent over/underflow.
    #[inline]
    pub fn offset<P>(&mut self, point: P)
    where
        P: Into<(i32, i32)>,
    {
        let (x, y): (i32, i32) = point.into();
        self.rect.offset(x, y);
    }
}

impl Draw for Tank {
    fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(self.color);
        canvas.fill_rect(self.rect)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottom_left() {
        let tank = Tank::new((0, 0).into(), Color::RGB(0, 0, 0));
        let height = TANK_SIZE as i32;
        assert_eq!(tank.bottom_left(), Point::new(0, height - 1));
    }
}
