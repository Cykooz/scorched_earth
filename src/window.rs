use sdl2;
use sdl2::image::InitFlag;
use sdl2::{event::Event, keyboard::Keycode, render::WindowCanvas, EventPump};

pub struct Window {
    pub sdl_context: sdl2::Sdl,
    pub canvas: WindowCanvas,
}

pub struct EventsIter {
    event_pump: EventPump,
}

impl Window {
    pub fn new(width: u32, height: u32, title: &str) -> Window {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let _image_context = sdl2::image::init(InitFlag::PNG | InitFlag::JPG).unwrap();
        let window = video_subsystem
            .window(title, width, height)
            .position_centered()
            .build()
            .unwrap();
        let canvas = window.into_canvas().accelerated().build().unwrap();

        Window {
            sdl_context,
            canvas,
        }
    }

    pub fn get_events_iter(&self) -> Result<EventsIter, String> {
        Ok(EventsIter {
            event_pump: self.sdl_context.event_pump()?,
        })
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.canvas.logical_size()
    }

    //    pub fn clear(&mut self, color: Color) {
    //        self.canvas.set_draw_color(color);
    //        self.canvas.clear();
    //    }
    //
    //    pub fn set_pixel(&mut self, x: i32, y: i32, color: Color) {
    //        self.canvas.set_draw_color(color);
    //        self.canvas.draw_point(Point::new(x, y)).unwrap();
    //    }

    //    pub fn load_texture<P: AsRef<Path>>(&self, file_name: P) -> Result<Texture, String> {
    //        self.texture_creator.load_texture(file_name)
    //    }

    //    #[inline]
    //    pub fn present(&mut self) {
    //        self.canvas.present();
    //    }
}

impl Iterator for EventsIter {
    type Item = Vec<Event>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut events: Vec<Event> = Vec::new();

        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return None;
                }
                _ => {
                    events.push(event);
                }
            }
        }
        Some(events)
    }
}
