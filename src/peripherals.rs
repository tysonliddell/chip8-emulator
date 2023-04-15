// use std::cell::Cell;

use std::time::Duration;

use minifb::{Key, Window, WindowOptions};
use rodio::{source, OutputStream, OutputStreamHandle, Sink, Source};

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

impl HexKeyboard for Window {
    fn get_current_pressed_key(&self) -> Option<u8> {
        match self.get_keys()[..] {
            [Key::Key1, ..] => Some(0x1),
            [Key::Key2, ..] => Some(0x2),
            [Key::Key3, ..] => Some(0x3),
            [Key::Q, ..] => Some(0x4),
            [Key::W, ..] => Some(0x5),
            [Key::E, ..] => Some(0x6),
            [Key::A, ..] => Some(0x7),
            [Key::S, ..] => Some(0x8),
            [Key::D, ..] => Some(0x9),
            [Key::X, ..] => Some(0x0),
            [Key::Z, ..] => Some(0xA),
            [Key::C, ..] => Some(0xB),
            [Key::Key4, ..] => Some(0xC),
            [Key::R, ..] => Some(0xD),
            [Key::F, ..] => Some(0xE),
            [Key::V, ..] => Some(0xF),
            _ => None,
        }
    }
}

pub struct Beeper {
    stream: OutputStream,
    sink: rodio::Sink,
}

impl Beeper {
    pub fn new(freq_hz: u32) -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        sink.pause();

        let source = source::SineWave::new(freq_hz as f32)
            .take_duration(Duration::from_secs_f32(0.25))
            .repeat_infinite()
            .amplify(0.20);
        sink.append(source);

        Self { stream, sink }
    }
}

impl Tone for Beeper {
    fn is_tone_on(&self) -> bool {
        !self.sink.is_paused()
    }

    fn start_tone(&self) {
        self.sink.play();
    }

    fn stop_tone(&self) {
        self.sink.pause();
    }
}
