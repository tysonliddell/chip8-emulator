// use std::cell::Cell;

pub trait Tone {
    fn start_tone(&self);
    fn stop_tone(&self);
    fn is_tone_on(&self) -> bool;
}

pub trait Screen {
    fn draw_monochrome_scanline(&self, scanline_pixels: &[u8]);
}

pub trait HexKeyboard {
    fn get_current_pressed_key(&self) -> Option<u8>;
}

pub struct DummyPeripherals {
    // tone_is_on: Cell<bool>,
}

impl Tone for DummyPeripherals {
    fn is_tone_on(&self) -> bool {
        false
    }

    fn start_tone(&self) {}

    fn stop_tone(&self) {}
}

impl Screen for DummyPeripherals {
    fn draw_monochrome_scanline(&self, scanline_pixels: &[u8]) {}
}

impl HexKeyboard for DummyPeripherals {
    fn get_current_pressed_key(&self) -> Option<u8> {
        None
    }
}
