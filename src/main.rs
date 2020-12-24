use std::env;
use std::fs::File;
use std::io::prelude::*;

#[macro_use]
extern crate log;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let rom_filename = &args[1];

    debug!("ROM file = {}", rom_filename);

    let mut f = File::open(rom_filename)?;
    let mut buf = Vec::new();
    let _ = f.read_to_end(&mut buf)?;
    println!("{:?}", &buf[0..4]);
    Ok(())
}
