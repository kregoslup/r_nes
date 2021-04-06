extern crate winit;
extern crate pixels;


use log::{info, warn};
use core::fmt;
use self::winit::event_loop::EventLoop;
use self::winit::dpi::{PhysicalSize, LogicalSize, LogicalPosition};
use self::pixels::{SurfaceTexture, Pixels};
use self::winit::window::Window;
use winit_input_helper::WinitInputHelper;
use crate::ppu::Colour;

const SCREEN_WIDTH: u32 = 256;
const SCREEN_HEIGHT: u32 = 240;

pub struct Screen {
    pixels: Pixels<Window>,
    window: Window
}

impl Screen {
    pub fn new() -> Screen {
        let event_loop = EventLoop::new();
        let mut input = WinitInputHelper::new();
        let (window, p_width, p_height, mut _hidpi_factor) =
            Screen::create_window("NES", &event_loop);

        let surface_texture = SurfaceTexture::new(p_width, p_height, &window);
        let mut pixels = Pixels::new(SCREEN_WIDTH, SCREEN_HEIGHT, surface_texture)
            .unwrap();
        pixels.render();
        Screen {
            pixels,
            window
        }
    }

    pub fn draw_pixels(&mut self, frame: &Vec<(u16, u16, Colour)>) {
        let screen = self.pixels.get_frame();
        for pixel in frame.iter() {
            let (x, y, colour) = pixel;
            let address = (((*y as u32 * SCREEN_WIDTH) + *x as u32) * 4) as usize;
            screen[address] = colour.r;
            screen[address + 1] = colour.g;
            screen[address + 2] = colour.b;
            screen[address + 3] = 0;
        }
        self.pixels.render();
        self.window.request_redraw();
    }

    pub fn clear(&mut self) {
        let frame = self.pixels.get_frame();
        for pixel in frame.chunks_exact_mut(4) {
            pixel[0] = 0x00; // R
            pixel[1] = 0x00; // G
            pixel[2] = 0x00; // B
            pixel[3] = 0xff; // A
        }
    }

    fn create_window(
        title: &str,
        event_loop: &EventLoop<()>,
    ) -> (winit::window::Window, u32, u32, f64) {
        // Create a hidden window so we can estimate a good default window size
        let window = winit::window::WindowBuilder::new()
            .with_visible(false)
            .with_title(title)
            .build(&event_loop)
            .unwrap();
        let hidpi_factor = window.scale_factor();

        // Get dimensions
        let width = SCREEN_WIDTH as f64;
        let height = SCREEN_HEIGHT as f64;
        let (monitor_width, monitor_height) = {
            if let Some(monitor) = window.current_monitor() {
                let size = monitor.size().to_logical(hidpi_factor);
                (size.width, size.height)
            } else {
                (width, height)
            }
        };
        let scale = (monitor_height / height * 2.0 / 3.0).round().max(1.0);

        // Resize, center, and display the window
        let min_size: winit::dpi::LogicalSize<f64> =
            PhysicalSize::new(width, height).to_logical(hidpi_factor);
        let default_size = LogicalSize::new(width * scale, height * scale);
        let center = LogicalPosition::new(
            (monitor_width - width * scale) / 2.0,
            (monitor_height - height * scale) / 2.0,
        );
        window.set_inner_size(default_size);
        window.set_min_inner_size(Some(min_size));
        window.set_outer_position(center);
        window.set_visible(true);

        let size = default_size.to_physical::<f64>(hidpi_factor);
        (
            window,
            size.width.round() as u32,
            size.height.round() as u32,
            hidpi_factor,
        )
    }

}

impl fmt::Debug for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Screen")
            .finish()
    }
}