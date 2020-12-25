const INES_HEADER: [u8; 4] = [0x4e, 0x45, 0x53, 0x1a];

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
        self.data[0..4] == INES_HEADER
    }
}

