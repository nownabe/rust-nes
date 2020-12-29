use std::env;
use std::fs::File;
use std::io::prelude::*;

#[macro_use]
extern crate log;

mod rom;
mod cpu;
mod instruction;

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

    let rom = rom::ROM::new(buf);

    if !rom.is_ines() {
        error!("ROM must be iNES format");
        return Err("ROM must be iNES format".into());
    }

    let program_data = rom.program_data();
    debug!("program data length = {} bytes", program_data.len());
    let mut cpu = cpu::Cpu::new();
    cpu.load_program(program_data);
    loop {
        cpu.tick()
    }


    // let program_data_size = &buf[4];
    // debug!("program data size = {}", program_data_size);
    // let character_data_size = &buf[5];
    // debug!("character data size = {}", character_data_size);

    Ok(())
}
