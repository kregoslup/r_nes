extern crate minifb;

use minifb::{Key, Window, WindowOptions};
use std::fmt;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

pub struct Screen {
    window: Window
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            window: Window::new(
                "Test - ESC to exit",
                WIDTH,
                HEIGHT,
                WindowOptions::default(),
            ).unwrap_or_else(|e| {
                panic!("{}", e);
            })
        }
    }
}

impl fmt::Debug for Screen {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Screen")
            .finish()
    }
}