/*
 * https://wiki.nesdev.com/w/index.php/INES#iNES_file_format
 */

const INES_HEADER_SIZE: usize = 16;
const INES_HEADER_CONSTANT: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];
const PROGRAM_UNIT_SIZE: usize = 0x4000;

#[derive(Debug)]
pub struct ROM {
    data: Vec<u8>,
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
        debug!("Program start byte = {}", start);
        debug!("Program end byte = {}", end);
        self.data[start..end].to_vec()
    }

    fn has_trainer(&self) -> bool {
        self.data[6] & 0b00000100 == 0b00000100
    }
}

