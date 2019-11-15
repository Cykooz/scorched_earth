use crate::G;
use noise::{self, Fbm, MultiFractal, NoiseFn, Seedable};
use std::time::Instant;

const TIME_SCALE: f32 = 3.0;

pub struct Landscape {
    width: u32,
    height: u32,
    buffer: Vec<u8>,
    noise: Option<Fbm>,
    amplitude: f64,
    pub dx: i32,
    pub changed: bool,
    subsidence_started: Option<Instant>,
    subsidence_rows_count: u32,
}

impl Landscape {
    pub fn new(width: u32, height: u32) -> Result<Landscape, String> {
        if width.min(height) == 0 || width.max(height) > ::std::i32::MAX as u32 {
            return Err(format!(
                "'width' and 'height' must be greater than 0 and less or equal than {}",
                std::i32::MAX
            ));
        }

        let stride = width as usize;
        let res_size = stride * height as usize;
        let noise = Fbm::new()
            .set_seed(0)
            .set_octaves(4)
            .set_frequency(2. / f64::from(width));
        Ok(Landscape {
            width,
            height,
            buffer: vec![0; res_size],
            amplitude: f64::from(height) / 2.,
            dx: 0,
            noise: Some(noise),
            changed: true,
            subsidence_started: None,
            subsidence_rows_count: 0,
        })
    }

    pub fn set_seed(&mut self, seed: u32) {
        let noise = std::mem::replace(&mut self.noise, None);
        self.noise = Some(noise.unwrap().set_seed(seed));
    }

    pub fn seed(&self) -> u32 {
        self.noise.as_ref().unwrap().seed()
    }

    pub fn set_octaves(&mut self, octaves: usize) {
        let noise = std::mem::replace(&mut self.noise, None);
        self.noise = Some(noise.unwrap().set_octaves(octaves));
    }

    pub fn set_frequency(mut self, frequency: f64) {
        let noise = std::mem::replace(&mut self.noise, None);
        self.noise = Some(noise.unwrap().set_frequency(frequency));
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn generate(&mut self) {
        let stride = self.width as usize;
        let y_center: f64 = f64::from(self.height) / 2.;
        let noise = self.noise.as_mut().unwrap();

        for x in 0..self.width as i32 {
            let sx = f64::from(x + self.dx);
            let value = noise.get([sx, 0.]) * self.amplitude;
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
    pub fn get_pixels_line_mut(&mut self, point: (i32, i32), length: u32) -> Option<&mut [u8]> {
        let (x, y) = point;
        if x < 0 || y < 0 || x >= self.width as _ || y >= self.height as _ || length == 0 {
            return None;
        }
        let index = (y * self.width as i32 + x) as usize;
        let length = length.min(self.width - x as u32) as usize;
        Some(&mut self.buffer[index..index + length])
    }

    pub fn is_not_empty(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        let index = (y as u32 * self.width + x as u32) as usize;
        self.buffer[index] > 0
    }

    pub fn to_rgba(&self) -> Vec<u8> {
        let image_size = self.buffer.len() * 4;
        let mut rgba: Vec<u8> = Vec::with_capacity(image_size);
        let buf = unsafe {
            rgba.set_len(image_size);
            rgba.align_to_mut::<u32>().1
        };
        for (&v, d) in self.buffer.iter().zip(buf) {
            *d = if v == 0 { 0 } else { 0xff_40_71_9c } // 0xff_cf_bd_00
        }

        rgba
    }

    pub fn subsidence(&mut self) {
        if self.subsidence_started.is_none() {
            self.subsidence_started = Some(Instant::now());
            self.subsidence_rows_count = 0;
        }
    }

    pub fn is_subsidence(&self) -> bool {
        self.subsidence_started.is_some()
    }

    pub fn update(&mut self) -> bool {
        if let Some(subsidence_started) = self.subsidence_started {
            let time = subsidence_started.elapsed().as_secs_f32();
            let subsidence_rows_count = (G * time * time * TIME_SCALE).round() as u32;
            let delta = subsidence_rows_count - self.subsidence_rows_count;
            self.subsidence_rows_count = subsidence_rows_count;

            for _ in 0..delta {
                let mut changed = false;
                for y in (1..self.height).rev() {
                    let split_from = (y * self.width) as usize;
                    let (top_rows, current_row) = self.buffer.split_at_mut(split_from);

                    let top_row_iter = top_rows.iter_mut().skip(split_from - self.width as usize);
                    top_row_iter
                        .zip(current_row)
                        .filter(|(&mut top_pixel, &mut cur_pixel)| cur_pixel == 0 && top_pixel != 0)
                        .for_each(|(top_pixel, cur_pixel)| {
                            *cur_pixel = *top_pixel;
                            *top_pixel = 0;
                            changed = true;
                        });
                }

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

//impl Draw for Landscape {
//    fn draw(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
//        canvas.set_draw_color(Color::RGB(0, 189, 207));
//        for point in self.iter_filled_points() {
//            canvas.draw_point(point)?;
//        }
//        Ok(())
//    }
//}
