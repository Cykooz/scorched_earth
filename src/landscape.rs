use std::time::Instant;

use ggez::graphics;
use itertools::Itertools;
use noise::{self, Fbm, MultiFractal, NoiseFn, Seedable};

use crate::G;

const TIME_SCALE: f32 = 3.0;

pub struct Landscape {
    width: i32,
    height: i32,
    buffer: Vec<u8>,
    rgba_buffer: Vec<u8>,
    noise: Fbm,
    amplitude: f64,
    pub dx: i32,
    changed: bool,
    subsidence_started: Option<Instant>,
    // Last position of virtual pixel of landscape on the way of it falling.
    // Used for calculate speed of fall.
    subsidence_last_pos: u32,
    // "Skip" and "take" used for optimize process of landscape subsidence.
    subsidence_skip: usize,
    subsidence_take: usize,
}

impl Landscape {
    pub fn new(width: u16, height: u16) -> Result<Landscape, String> {
        if width.min(height) == 0 {
            return Err("'width' and 'height' must be greater than 0".into());
        }

        let stride = width as usize;
        let res_size = stride * height as usize;
        Ok(Landscape {
            width: width as i32,
            height: height as i32,
            buffer: vec![0; res_size],
            rgba_buffer: vec![0; res_size * 4],
            amplitude: f64::from(height) / 2.,
            dx: 0,
            noise: Self::create_noise(width as i32, 0),
            changed: true,
            subsidence_started: None,
            subsidence_last_pos: 0,
            subsidence_skip: 0,
            subsidence_take: stride,
        })
    }

    fn create_noise(width: i32, seed: u32) -> Fbm {
        Fbm::new()
            .set_seed(seed)
            .set_octaves(4)
            .set_frequency(2. / f64::from(width))
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.noise = Self::create_noise(self.width, seed);
    }

    pub fn seed(&self) -> u32 {
        self.noise.seed()
    }

    #[inline]
    pub fn changed(&self) -> bool {
        self.changed
    }

    #[inline]
    pub fn set_changed(&mut self) {
        self.changed = true;
    }

    #[inline]
    pub fn size(&self) -> (u16, u16) {
        (self.width as u16, self.height as u16)
    }

    pub fn generate(&mut self) {
        let stride = self.width as usize;
        let y_center: f64 = f64::from(self.height) / 2.;

        for x in 0..self.width {
            let sx = f64::from(x + self.dx);
            let value = self.noise.get([sx, 0.]) * self.amplitude;
            let y = (y_center + value).round().max(0.) as usize;
            let y = y.min(self.height as usize);
            let index = y * stride + (x as usize);

            if y > 0 {
                self.buffer
                    .iter_mut()
                    .skip(x as usize)
                    .step_by(stride)
                    .take(y)
                    .for_each(|v| *v = 0);
            }

            self.buffer
                .iter_mut()
                .skip(index)
                .step_by(stride)
                .for_each(|v| *v = 1);
        }
    }

    /// Get mutable slice with row of pixels given length
    pub fn get_pixels_line_mut(&mut self, point: (i32, i32), length: u16) -> Option<&mut [u8]> {
        let (x, y) = point;
        if x < 0 || y < 0 || x >= self.width || y >= self.height || length == 0 {
            return None;
        }
        let index = (y * self.width + x) as usize;
        let length = (self.width - x).min(length as i32) as usize;
        Some(&mut self.buffer[index..index + length])
    }

    pub fn is_not_empty(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width || y >= self.height as i32 {
            return false;
        }
        let index = (y * self.width + x) as usize;
        self.buffer[index] > 0
    }

    pub fn create_image(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult<graphics::Image> {
        let buf = unsafe { self.rgba_buffer.align_to_mut::<u32>().1 };
        for (&v, d) in self.buffer.iter().zip(buf) {
            *d = if v == 0 { 0 } else { 0xff_40_71_9c } // 0xff_cf_bd_00
        }
        self.changed = false;

        graphics::Image::from_rgba8(
            ctx,
            self.width as u16,
            self.height as u16,
            &self.rgba_buffer,
        )
    }

    pub fn subsidence(&mut self) {
        if self.subsidence_started.is_none() {
            self.subsidence_started = Some(Instant::now());
            self.subsidence_last_pos = 0;
            self.subsidence_skip = 0;
            self.subsidence_take = self.width as usize;
        }
    }

    pub fn is_subsidence(&self) -> bool {
        self.subsidence_started.is_some()
    }

    /// Returns `true` if current subsidence has finished.
    pub fn update(&mut self) -> bool {
        if let Some(subsidence_started) = self.subsidence_started {
            let time = subsidence_started.elapsed().as_secs_f32();
            let subsidence_cur_pos = (G * time * time * TIME_SCALE).round() as u32;
            let delta = subsidence_cur_pos - self.subsidence_last_pos;
            self.subsidence_last_pos = subsidence_cur_pos;
            let stride = self.width as usize;

            for _ in 0..delta {
                let mut changed = false;
                let mut cur_row_index = stride * self.height as usize;
                let mut left_changed_pos: usize = self.subsidence_take;
                let mut right_changed_pos = 0;

                for _ in 1..self.height {
                    cur_row_index -= stride;
                    let (top_rows, current_row) = self.buffer.split_at_mut(cur_row_index);
                    let (_, top_row) = top_rows.split_at_mut(cur_row_index - stride);
                    let pixels_for_change = top_row
                        .iter_mut()
                        .zip(current_row)
                        .skip(self.subsidence_skip)
                        .take(self.subsidence_take)
                        .enumerate()
                        .filter(|(_, (&mut top_pixel, &mut cur_pixel))| {
                            cur_pixel == 0 && top_pixel != 0
                        });
                    let min_max = pixels_for_change
                        .map(|(i, (top_pixel, cur_pixel))| {
                            *cur_pixel = *top_pixel;
                            *top_pixel = 0;
                            i
                        })
                        .minmax();

                    if let Some((min, max)) = min_max.into_option() {
                        changed = true;
                        left_changed_pos = left_changed_pos.min(min);
                        right_changed_pos = right_changed_pos.max(max);
                    };
                }

                self.subsidence_skip += left_changed_pos;
                self.subsidence_take = right_changed_pos + 1;

                if changed {
                    self.changed = true;
                } else {
                    self.subsidence_started = None;
                    return true;
                }
            }
        }

        false
    }
}
