use super::cassette::Cassette;
use super::cassette::Sprite;
use super::ppu_register_bus::PpuRegisterBus;

/*
 * Container for sharable hardwares, such as PPU registers and cassette.
 */
pub struct Nes {
    cassette: Cassette,
    pub ppu_register_bus: PpuRegisterBus,
}

impl Nes {
    pub fn new(cassette: Cassette) -> Self {
        Self {
            cassette,
            ppu_register_bus: PpuRegisterBus::new(),
        }
    }

    pub fn read_program(&self, addr: u16) -> u8 {
        self.cassette.prg_rom[addr as usize]
    }

    pub fn read_chr_rom(&self, addr: u16) -> u8 {
        self.cassette.chr_rom[addr as usize]
    }

    pub fn get_sprite(&self, id: u8) -> &Sprite {
        &self.cassette.sprites[id as usize]
    }
}
