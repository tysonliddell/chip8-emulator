// use std::cell::Cell;

use minifb::{Window, WindowOptions};

use crate::interpreter::{DISPLAY_HEIGHT_PIXELS, DISPLAY_WIDTH_PIXELS};

pub trait Tone {
    fn start_tone(&self) {}
    fn stop_tone(&self) {}
    fn is_tone_on(&self) -> bool {
        false
    }
}

pub trait Screen {
    fn draw_buffer(&mut self, buffer: &[u8]) {}
}

pub trait HexKeyboard {
    fn get_current_pressed_key(&self) -> Option<u8> {
        None
    }
}

pub struct DummyPeripherals {
    // tone_is_on: Cell<bool>,
}

impl Tone for DummyPeripherals {}
impl Screen for DummyPeripherals {}
impl HexKeyboard for DummyPeripherals {}

impl Screen for Window {
    fn draw_buffer(&mut self, buffer: &[u8]) {
        let buffer: Vec<_> = buffer
            .iter()
            .flat_map(|pixel_byte| {
                let mut color_pixels = [0x00FFFFFFu32; 8]; // default to 8 white pixels
                for (i, rgb_pixel) in color_pixels.iter_mut().enumerate() {
                    if pixel_byte & (1 << (7 - i)) != 0 {
                        *rgb_pixel = 0; // make pixel black
                    }
                }
                color_pixels
            })
            .collect();
        self.update_with_buffer(&buffer, DISPLAY_WIDTH_PIXELS, DISPLAY_HEIGHT_PIXELS)
            .unwrap();
    }
}
