use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{
    interpreter::{Chip8Interpreter, DISPLAY_WIDTH_PIXELS, PROGRAM_COUNTER_ADDRESS},
    memory::CosmacRAM,
    peripherals::{HexKeyboard, Screen, Tone},
    Result,
};

type Chip8 = Chip8Interpreter<fastrand::Rng>;

const INSTRUCTIONS_FREQ_HZ: u64 = 700; // number of CHIP-8 instructions performed per second
const INSTRUCTION_DURATION: Duration = Duration::from_micros(1_000_000 / INSTRUCTIONS_FREQ_HZ);

pub fn run<T>(chip8_program: &[u8], peripherals: &mut T) -> Result<()>
where
    T: Tone + Screen + HexKeyboard,
{
    let mut ram = CosmacRAM::new();
    ram.load_chip8_program(chip8_program)?;

    let mut chip8 = Chip8::new(fastrand::Rng::new());
    chip8.reset(&mut ram);
    peripherals.draw_buffer(ram.display_buffer());

    let mut frame_count = 0u32;
    let program_start_time = Instant::now();
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
        let pc = ram.get_u16_at(PROGRAM_COUNTER_ADDRESS);
        let instruction = ram.get_u16_at(pc as usize);
        if instruction & 0xD000 == 0xD000 {
            // display instruction
            peripherals.draw_buffer(ram.display_buffer());
        }

        // update tone
        let tone_should_be_sounding = Chip8::is_tone_sounding(&ram);
        if tone_should_be_sounding && !peripherals.is_tone_on() {
            peripherals.start_tone();
        } else if !tone_should_be_sounding && peripherals.is_tone_on() {
            peripherals.stop_tone();
        }

        // set hex key press state
        Chip8::set_current_key_press(&mut ram, peripherals.get_current_pressed_key());

        // sleep the required amount of time to maintain CLOCK_FREQ_HZ instructions per second
        frame_count += 1;
        let target_end_instruction_time = program_start_time + (frame_count * INSTRUCTION_DURATION);
        let sleep_for =
            target_end_instruction_time - Instant::now().min(target_end_instruction_time);
        sleep(sleep_for);
    }
}
