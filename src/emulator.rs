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

pub fn run<T>(chip8_program: &[u8], peripherals: &mut T) -> Result<()>
where
    T: Tone + Screen + HexKeyboard,
{
    let mut ram = CosmacRAM::new();
    ram.load_chip8_program(chip8_program)?;

    let chip8 = Chip8::new(fastrand::Rng::new());
    chip8.reset(&mut ram);

    loop {
        #[cfg(debug_assertions)]
        {
            eprintln!("Before instruction");
            dbg!(Chip8::get_state(&ram));
        }

        chip8.step(&mut ram);

        #[cfg(debug_assertions)]
        {
            eprintln!("After instruction");
            dbg!(Chip8::get_state(&ram));
        }

        // update display
        // FIXME: Probably don't have to update the display on every cycle.
        peripherals.draw_buffer(ram.display_buffer());

        // update tone
        let tone_should_be_sounding = Chip8::is_tone_sounding(&ram);
        if tone_should_be_sounding && !peripherals.is_tone_on() {
            peripherals.start_tone();
        } else if !tone_should_be_sounding && peripherals.is_tone_on() {
            peripherals.stop_tone();
        }

        // set hex key press state
        Chip8::set_current_key_press(&mut ram, peripherals.get_current_pressed_key());

        sleep(MICRO_SEC_PER_INSTRUCTION);
    }
}
