use std::{thread::sleep, time::Duration};

use crate::{
    interpreter::{Chip8Interpreter, DISPLAY_WIDTH_PIXELS},
    memory::CosmacRAM,
    peripherals::{HexKeyboard, Screen, Tone},
    Result,
};

type Chip8 = Chip8Interpreter<fastrand::Rng>;

const BYTES_PER_SCANLINE: usize = DISPLAY_WIDTH_PIXELS / 8;
const MICRO_SEC_PER_INSTRUCTION: Duration = Duration::from_micros(1_000_000 / 60);

pub fn run<T, U, V>(
    chip8_program: &[u8],
    tone: &T,
    display_renderer: &U,
    hex_keyboard: &V,
) -> Result<()>
where
    T: Tone,
    U: Screen,
    V: HexKeyboard,
{
    let mut ram = CosmacRAM::new();
    ram.load_chip8_program(chip8_program)?;

    let chip8 = Chip8::new(fastrand::Rng::new());
    chip8.reset(&mut ram);

    loop {
        // update display
        for row in ram.display_buffer().chunks(BYTES_PER_SCANLINE) {
            display_renderer.draw_monochrome_scanline(row);
        }

        // update tone
        let tone_should_be_sounding = Chip8::is_tone_sounding(&ram);
        if tone_should_be_sounding && !tone.is_tone_on() {
            tone.start_tone();
        } else if !tone_should_be_sounding && tone.is_tone_on() {
            tone.stop_tone();
        }

        // set hex key press state
        Chip8::set_current_key_press(&mut ram, hex_keyboard.get_current_pressed_key());

        chip8.step(&mut ram);
        sleep(MICRO_SEC_PER_INSTRUCTION);
    }
}
