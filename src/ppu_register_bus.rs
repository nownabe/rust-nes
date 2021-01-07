use super::ppu::Register;

pub enum PpuDataStatus {
    None,
    Read,
    Written,
}

pub struct PpuRegisterBus {
    ppu_addr_higher: Option<u8>,
    ppu_addr: Option<u16>,
    ppu_data: u8,
    ppu_data_status: PpuDataStatus,
}

impl PpuRegisterBus {
    pub fn new() -> Self {
        Self {
            ppu_addr_higher: None,
            ppu_addr: None,
            ppu_data: 0,
            ppu_data_status: PpuDataStatus::None,
        }
    }

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr.into() {
            Register::PPUCTRL => { todo!("Setting PPUCTRL is not implemented"); },
            Register::PPUMASK => { todo!("Setting PPUMASK is not implemented"); },
            Register::PPUSTATUS => { todo!("Setting PPUSTATUS is not implemented"); },
            Register::OAMADDR => { todo!("Setting OAMADDR is not implemented"); },
            Register::OAMDATA => { todo!("Setting OAMDATA is not implemented"); },
            Register::PPUSCROLL => { todo!("Setting PPUSCROLL is not implemented"); },
            Register::PPUADDR => panic!("Forbidden to read PPUADDR from CPU"),
            Register::PPUDATA => {
                self.ppu_data_status = PpuDataStatus::Read;
                self.ppu_data
            },
            Register::OAMDMA => { todo!("Setting OAMDMA is not implemented"); },
        }
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr.into() {
            Register::PPUCTRL => { todo!("Setting PPUCTRL is not implemented"); },
            Register::PPUMASK => { todo!("Setting PPUMASK is not implemented"); },
            Register::PPUSTATUS => { todo!("Setting PPUSTATUS is not implemented"); },
            Register::OAMADDR => { todo!("Setting OAMADDR is not implemented"); },
            Register::OAMDATA => { todo!("Setting OAMDATA is not implemented"); },
            Register::PPUSCROLL => { todo!("Setting PPUSCROLL is not implemented"); },
            Register::PPUADDR => {
                match (self.ppu_addr_higher, self.ppu_addr) {
                    (None, _) => self.ppu_addr_higher = Some(data),
                    (Some(higher), _) => {
                        self.ppu_addr = Some((higher as u16) << 8 | data as u16);
                        self.ppu_addr_higher = None;
                    },
                }
            },
            Register::PPUDATA => {
                self.ppu_data = data;
                self.ppu_data_status = PpuDataStatus::Written;
            },
            Register::OAMDMA => { todo!("Setting OAMDMA is not implemented"); },
        }
    }

    pub fn ppu_read(&mut self, r: Register) -> Option<u16> {
        match r {
            Register::PPUADDR => {
                let addr = self.ppu_addr;
                self.ppu_addr = None;
                addr
            },
            Register::PPUDATA => {
                self.ppu_data_status = PpuDataStatus::None;
                Some(self.ppu_data as u16)
            },
            _ => {None},
        }
    }

    pub fn ppu_write(&mut self, r: Register, data: u8) {
        match r {
            Register::PPUDATA => {
                self.ppu_data = data;
                self.ppu_data_status = PpuDataStatus::None;
            },
            _ => panic!("Forbidden to write data into {:04X} from PPU", usize::from(r)),
        }
    }

    pub fn ppu_data_status(&self) -> &PpuDataStatus {
        &self.ppu_data_status
    }
}
