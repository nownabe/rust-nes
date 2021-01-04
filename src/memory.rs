const MEMORY_SIZE: usize = 0x10000;

pub struct Memory([u8; MEMORY_SIZE]);

impl Memory {
    pub fn new() -> Self {
        Self([0; MEMORY_SIZE])
    }

    pub fn write(&mut self, addr: usize, data: u8) {
        self.0[addr] = data
    }

    pub fn read(&self, addr: usize) -> u8 {
        self.0[addr]
    }
}
