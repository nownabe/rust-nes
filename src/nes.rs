use super::cassette::Cassette;
use super::cassette::Sprite;

/*
 * Container for sharable hardwares, such as PPU registers and cassette.
 */
pub struct Nes {
    // TODO: Make cassette private
    pub cassette: Cassette,
}

impl Nes {
    pub fn new(cassette: Cassette) -> Self {
        Self {
            cassette,
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
