extern crate sdl2;

use log::{info, warn};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use sdl2::render::WindowCanvas;
use std::fmt;
use self::sdl2::{Sdl, EventPump};
use self::sdl2::pixels::PixelFormatEnum;
use self::sdl2::rect::Point;

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
                "NES emulator",
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
            )
            .position_centered()
            .resizable()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .accelerated()
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

    pub fn draw_pixels(&mut self, mut pixels: &Vec<(u16, u16)>) {
        let creator = self.canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGB24,SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
        self.canvas.set_draw_color(pixels::Color::RGB(250, 250, 250));

        pixels.iter().map(
            |x| Point::new(x.0 as i32, x.1 as i32)
        ).map(
            |y| self.canvas.draw_point(y)
        ).collect::<Vec<_>>();
        self.canvas.present();
        texture.update()
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();
    }

    pub fn draw_pixel(&mut self, cor_x: u16, cor_y: u16) {
        let creator = self.canvas.texture_creator();
        let mut texture = creator
            .create_texture_target(PixelFormatEnum::RGB24,SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
        self.canvas.set_draw_color(pixels::Color::RGB(250, 250, 250));
        self.canvas.draw_point(Point::new(cor_x as i32, cor_y as i32));
        self.canvas.present();
    }

}

impl fmt::Debug for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Screen")
            .finish()
    }
}