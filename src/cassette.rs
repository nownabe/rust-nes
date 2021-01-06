/*
 * https://wiki.nesdev.com/w/index.php/INES#iNES_file_format
 */

const INES_HEADER_SIZE: usize = 16;
const INES_HEADER_CONSTANT: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];
const TRAINER_SIZE: usize = 0x0200; // 512 bytes
const PRG_ROM_UNIT_SIZE: usize = 0x4000; // 16384 bytes
const CHR_ROM_UNIT_SIZE: usize = 0x2000; // 8192 bytes

pub struct Cassette {
    header: [u8; INES_HEADER_SIZE],
    // trainer: Option<[u8; TRAINER_SIZE]>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub sprites: Vec<Sprite>,
}

impl Cassette {
    pub fn new(data: Vec<u8>) -> Self {
        // Parse header
        let mut header = [0; INES_HEADER_SIZE];
        for i in 0..INES_HEADER_SIZE {
            header[i] = data[i];
        }

        // Trainer, if present
        if data[6] & 0b00000100 == 0b00000100 {
            panic!("Trainer is not implement yet");
        }

        // Parse PRG ROM data
        let prg_start = INES_HEADER_SIZE;
        let prg_end = prg_start + PRG_ROM_UNIT_SIZE * (header[4] as usize);
        debug!("PRG ROM size = {} units ({} bytes)", header[4], prg_end - prg_start);
        debug!("PRG ROM start address = 0x{:X}", prg_start);
        debug!("PRG ROM end address = 0x{:X}", prg_end);

        let chr_start = prg_end;
        let chr_end = chr_start + CHR_ROM_UNIT_SIZE * (header[5] as usize);
        debug!("CHR ROM size = {} units ({} bytes)", header[5], chr_end - chr_start);
        debug!("CHR ROM start address = 0x{:X}", chr_start);
        debug!("CHR ROM end address = 0x{:X}", chr_end);
        let chr_rom = data[chr_start..chr_end].to_vec();

        let sprites = Self::parse_sprites(&chr_rom);

        Self {
            header,
            prg_rom: data[prg_start..prg_end].to_vec(),
            chr_rom,
            sprites,
        }
    }

    fn parse_sprites(chr_rom: &Vec<u8>) -> Vec<Sprite> {
        let mut sprites = Vec::new();
        for i in 0..(chr_rom.len()/16) {
            sprites.push(Sprite::new(&chr_rom[i*16..i*16+16]));
        }
        sprites
    }

    pub fn is_ines(&self) -> bool {
        self.header[0..4] == INES_HEADER_CONSTANT
    }
}

pub const SPRITE_WIDTH: usize = 8;
pub const SPRITE_HEIGHT: usize = 8;

#[derive(Debug)]
pub struct Sprite {
    data: [[u8; SPRITE_WIDTH]; SPRITE_HEIGHT],
}

impl Sprite {
    pub fn new(data: &[u8]) -> Self {
        let mut sprite = [[0; 8]; 8];
        for y in 0..SPRITE_HEIGHT {
            for x in 0..SPRITE_WIDTH {
                sprite[y][x] = (data[y] & 1 << (7-x)) >> (7-x);
                sprite[y][x] += (data[y+8] & 1 << (7-x)) >> (7-x) << 1;
            }
        }
        Self { data: sprite }
    }

    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.data[y][x]
    }
}
