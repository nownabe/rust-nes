/*
 * https://wiki.nesdev.com/w/index.php/INES#iNES_file_format
 */

const INES_HEADER_SIZE: usize = 16;
const INES_HEADER_CONSTANT: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];
const PROGRAM_UNIT_SIZE: usize = 0x4000;
const CHARACTER_ROM_UNIT_SIZE: usize = 0x2000;

#[derive(Debug)]
pub struct ROM {
    data: Vec<u8>,
    // TODO: Add program_rom and character_rom here
}

impl ROM {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
        }
    }

    pub fn is_ines(&self) -> bool {
        self.data[0..4] == INES_HEADER_CONSTANT
    }

    pub fn program_data(&self) -> Vec<u8> {
        let start = if self.has_trainer() {
            INES_HEADER_SIZE + 512
        } else {
            INES_HEADER_SIZE
        };
        debug!("Program data units = {}", self.data[4]);
        let end = start + PROGRAM_UNIT_SIZE * (self.data[4] as usize);
        debug!("Program ROM start address = 0x{:X}", start);
        debug!("Program ROM end address = 0x{:X}", end);
        self.data[start..end].to_vec()
    }

    pub fn character_rom(&self) -> Vec<u8> {
        let mut start = INES_HEADER_SIZE + PROGRAM_UNIT_SIZE * (self.data[4] as usize);
        if self.has_trainer() {
            start += 512
        }
        let end = start + CHARACTER_ROM_UNIT_SIZE * (self.data[5] as usize);

        debug!("Character ROM size = {} units, = {} bytes",
               self.data[5], self.data[5] as usize* CHARACTER_ROM_UNIT_SIZE);
        debug!("Character ROM start address = 0x{:X}", start);
        debug!("Character ROM end address = 0x{:X}", end);
        self.data[start..end].to_vec()
    }

    fn has_trainer(&self) -> bool {
        self.data[6] & 0b00000100 == 0b00000100
    }
}

