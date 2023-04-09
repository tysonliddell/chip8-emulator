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
