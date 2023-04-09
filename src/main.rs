use std::{
    fs::File,
    io::{BufReader, Read},
};

use chip8_emulator::{emulator, peripherals::DummyPeripherals};
use minifb::{Scale, Window, WindowOptions};

fn main() {
    let config = cli::parse_args();

    let chip8_program = File::open(&config.chip8_program_path)
        .and_then(|file| BufReader::new(file).bytes().collect());
    let chip8_program: Vec<u8> = match chip8_program {
        Err(e) => {
            eprintln!("{}: {}", config.chip8_program_path, e);
            std::process::exit(1);
        }
        Ok(bytes) => bytes,
    };

    let dummy_peripherals = DummyPeripherals {};
    let tone = &dummy_peripherals;
    let hex_keyboard = &dummy_peripherals;

    let window_opts = WindowOptions {
        scale: Scale::X8,
        ..WindowOptions::default()
    };
    let mut window = Window::new("CHIP-8 Emulator", 64, 32, window_opts)
        .expect("Expect window creation to succeed");

    if let Err(e) = emulator::run(&chip8_program, tone, &mut window, hex_keyboard) {
        eprintln!("emulator error: {}", e);
        std::process::exit(1);
    }
}

mod cli {
    use clap::Parser;

    #[derive(Debug)]
    pub struct Config {
        pub chip8_program_path: String,
    }

    #[derive(Parser)]
    #[command(author, version, about, long_about = None)]
    struct Args {
        /// Path to the rom to emulate
        #[arg(name = "chip8_program_path", value_name = "CHIP-8_PROGRAM_PATH")]
        chip8_program_path: String,
    }

    pub fn parse_args() -> Config {
        let args = Args::parse();
        Config {
            chip8_program_path: args.chip8_program_path,
        }
    }
}
