use std::env;
use std::fs::File;
use std::io::prelude::*;

#[macro_use]
extern crate log;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        error!("ROM filename is required");
        return Err("ROM filename is required".into());
    }
    let rom_filename = &args[1];

    debug!("ROM file = {}", rom_filename);

    let mut f = File::open(rom_filename)?;
    let mut buf = Vec::new();
    let _ = f.read_to_end(&mut buf)?;

    let head = &buf[0..4];
    if head != [0x4e, 0x45, 0x53, 0x1a] {
        error!("ROM must be iNES format");
        return Err("ROM must be iNES format".into());
    }

    let program_data_size = &buf[4];
    debug!("program data size = {}", program_data_size);
    let character_data_size = &buf[5];
    debug!("character data size = {}", character_data_size);

    Ok(())
}
