fn main() {
    let config = cli::parse_args();
    match io::load_rom_from_path(&config.rom_path) {
        Err(e) => {
            eprintln!("{}: {}", config.rom_path, e);
            std::process::exit(1);
        }
        Ok(rom) => println!("{:?}", rom),
    }
}

mod io {
    use std::{
        error::Error,
        fs::File,
        io::{BufReader, Read},
        path::Path,
    };

    use chip8_emulator::rom::Rom;

    pub fn load_rom_from_path(path: impl AsRef<Path>) -> Result<Rom, Box<dyn Error>> {
        let rom_file = BufReader::new(File::open(path)?);
        let bytes: Result<Vec<u8>, _> = rom_file.bytes().collect();
        Ok(Rom::from_bytes("some_rom", bytes?.as_slice())?)
    }
}

mod cli {
    use clap::Parser;

    #[derive(Debug)]
    pub struct Config {
        pub rom_path: String,
    }

    #[derive(Parser)]
    #[command(author, version, about, long_about = None)]
    struct Args {
        /// Path to the rom to emulate
        #[arg(name = "rom_path", value_name = "ROM_PATH")]
        rom_path: String,
    }

    pub fn parse_args() -> Config {
        let args = Args::parse();
        Config {
            rom_path: args.rom_path,
        }
    }
}
