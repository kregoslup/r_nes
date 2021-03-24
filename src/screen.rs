extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::render::WindowCanvas;
use std::fmt;
use self::sdl2::{Sdl, EventPump};

const SCREEN_WIDTH: u32 = 256;
const SCREEN_HEIGHT: u32 = 240;

pub struct Screen {
    canvas: WindowCanvas,
    sdl: Sdl
}

impl Screen {
    pub fn new() -> Screen {
        let sdl_context = sdl2::init().unwrap();
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys
            .window(
                "rust-sdl2_gfx: draw line & FPSManager",
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
            )
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();
        Screen {
            canvas,
            sdl: sdl_context
        }
    }

    pub fn events_stream(&mut self) -> EventPump {
        self.sdl.event_pump().unwrap()
    }

    fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
        let (r, g, b) = (r as u32, g as u32, b as u32);
        (r << 16) | (g << 8) | b
    }

    pub fn draw_pixels(&mut self, pixels: Vec<u8>) {
        let buffer_width = 100;
        let buffer_height = 150;
        let azure_blue = Screen::from_u8_rgb(0, 127, 255);
        let mut buffer: Vec<u32> = vec![azure_blue; buffer_width * buffer_height];
//        self.window.update_with_buffer(&buffer, buffer_width, buffer_height).unwrap();
    }
}

impl fmt::Debug for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Screen")
            .finish()
    }
}