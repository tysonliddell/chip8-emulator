use chip8_emulator::memory::CosmacRAM;

fn main() {
    let config = cli::parse_args();
    let mut ram = CosmacRAM::new();

    if let Err(e) = io::load_chip8_program_into_cosmac(&config.chip8_program_path, &mut ram) {
        eprintln!("{}: {}", config.chip8_program_path, e);
        std::process::exit(1);
    }

    // Inspect the program in RAM.
    println!("{:?}", &ram.bytes()[0x0200..0x0300])
}

mod io {
    use std::{
        error::Error,
        fs::File,
        io::{BufReader, Read},
        path::Path,
    };

    use chip8_emulator::memory::CosmacRAM;

    pub fn load_chip8_program_into_cosmac(
        path: impl AsRef<Path>,
        ram: &mut CosmacRAM,
    ) -> Result<(), Box<dyn Error>> {
        let chip8_program_file = BufReader::new(File::open(path)?);
        let bytes: Result<Vec<u8>, _> = chip8_program_file.bytes().collect();
        ram.load_chip8_program(bytes?.as_slice())?;
        Ok(())
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
