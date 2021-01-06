use super::ppu::Register;

const MEMORY_SIZE: usize = 0x10000;

// https://wiki.nesdev.com/w/index.php/PPU_registers#PPUADDR
#[derive(Copy, Clone)]
pub enum PpuAddrState {
    Higher, // 1回目
    Lower,  // 2回目
    None,
}

#[derive(Copy, Clone)]
pub enum PpuDataState {
    Read,
    Written,
    None,
}

pub struct Memory {
    data: [u8; MEMORY_SIZE],
    ppu_addr_state: PpuAddrState,
    ppu_data_state: PpuDataState,
}

impl Memory {
    pub fn new() -> Self {
        Self{
            data: [0; MEMORY_SIZE],
            ppu_addr_state: PpuAddrState::None,
            ppu_data_state: PpuDataState::None,
        }
    }

    pub fn write(&mut self, addr: usize, data: u8) {
        self.data[addr] = data;

        if addr == Register::PPUADDR.into() {
            self.ppu_addr_state = match self.ppu_addr_state {
                PpuAddrState::Higher => PpuAddrState::Lower,
                PpuAddrState::None | PpuAddrState::Lower => PpuAddrState::Lower,
            };
        }

        if addr == Register::PPUDATA.into() {
            self.ppu_data_state = PpuDataState::Written;
        }
    }

    pub fn read(&mut self, addr: usize) -> u8 {
        if addr == Register::PPUDATA.into() {
            self.ppu_data_state = PpuDataState::Read;
        }

        self.data[addr]
    }

    pub fn ppu_addr_state(&self) -> PpuAddrState {
        self.ppu_addr_state
    }

    pub fn ppu_data_state(&self) -> PpuDataState {
        self.ppu_data_state
    }
}
