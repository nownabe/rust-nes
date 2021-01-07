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

    #[allow(dead_code)]
    pub fn new_for_test(prg_rom: Vec<u8>) -> Self {
        let len = prg_rom.len();
        let mut data = [vec![0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], prg_rom].concat();
        for _ in 0..(0x4000-len) {
            data.push(0);
        }

        Self {
            cassette: Cassette::new(data),
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
