use super::memory::Memory;
use super::memory::PpuAddrState;
use super::memory::PpuDataState;
use super::nes::Nes;
use super::cassette::Sprite;
use super::cassette::SPRITE_WIDTH;
use super::cassette::SPRITE_HEIGHT;

const VRAM_SIZE: usize = 0x0800;
const OAM_SIZE: usize = 0x0100;
const CHARACTER_ROM_SIZE: usize = 0x2000;

pub const VISIBLE_SCREEN_WIDTH: usize = 256;
pub const VISIBLE_SCREEN_HEIGHT: usize = 240;

// https://wiki.nesdev.com/w/index.php/PPU_rendering#Line-by-line_timing
const CYCLES_PER_SCANLINE: usize = 341;
const SCANLINES_PER_FRAME: usize = 262;
const CYCLES_PER_FRAME: usize = CYCLES_PER_SCANLINE * SCANLINES_PER_FRAME;

// 16ラインずつ処理
const RENDERING_BATCH_LINES: usize = 16;
const RENDERING_BATCH_NUM: usize = VISIBLE_SCREEN_HEIGHT / RENDERING_BATCH_LINES;

pub enum Register {
    PPUCTRL,
    PPUMASK,
    PPUSTATUS,
    OAMADDR,
    OAMDATA,
    PPUSCROLL,
    PPUADDR,
    PPUDATA,
    OAMDMA,
}

impl From<Register> for usize {
    fn from(r: Register) -> Self {
        match r {
            Register::PPUCTRL => 0x2000,
            Register::PPUMASK => 0x2001,
            Register::PPUSTATUS => 0x2002,
            Register::OAMADDR => 0x2003,
            Register::OAMDATA => 0x2004,
            Register::PPUSCROLL => 0x2005,
            Register::PPUADDR => 0x2006,
            Register::PPUDATA => 0x2007,
            Register::OAMDMA => 0x4014,
        }
    }
}


pub struct Ppu {
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    ppu_addr: u16,
    cycle_counter: usize,
    batch_counter: usize,
    pub screen: [[[u8; 3]; VISIBLE_SCREEN_WIDTH]; VISIBLE_SCREEN_HEIGHT],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            ppu_addr: 0,
            cycle_counter: 0,
            batch_counter: 0,
            screen: [[[0, 0, 0]; VISIBLE_SCREEN_WIDTH]; VISIBLE_SCREEN_HEIGHT],
        }
    }

    // https://wiki.nesdev.com/w/index.php/PPU_power_up_state
    pub fn init(&mut self, mem: &mut Memory) {
        self.write_register(mem, Register::PPUCTRL, 0x00);
        self.write_register(mem, Register::PPUMASK, 0x00);
        self.write_register(mem, Register::PPUSTATUS, 0b10100000);
        self.write_register(mem, Register::OAMADDR, 0x00);
        self.write_register(mem, Register::PPUSCROLL, 0x00);
        self.write_register(mem, Register::PPUADDR, 0x00);
        self.write_register(mem, Register::PPUDATA, 0x00);
    }

    pub fn step(&mut self, nes: &mut Nes, mem: &mut Memory, cpu_cycle: usize) {
        self.cycle_counter += cpu_cycle * 3;

        self.operate_vram(nes, mem);
        self.render(nes, mem);

        if self.cycle_counter >= CYCLES_PER_FRAME {
            self.cycle_counter -= CYCLES_PER_FRAME;
            self.batch_counter = 0;
        }
    }

    pub fn read_vram(&self, addr: u16) -> u8 {
        self.vram[addr as usize]
    }

    pub fn write_vram(&mut self, addr: u16, data: u8) {
        self.vram[addr as usize] = data;
    }

    fn read_register(&self, mem: &mut Memory, r: Register) -> u8 {
        mem.read(r.into())
    }

    fn write_register(&self, mem: &mut Memory, r: Register, data: u8) {
        mem.write(r.into(), data);
    }

    // ref. https://wiki.nesdev.com/w/index.php/PPU_memory_map
    fn read(&mut self, nes: &mut Nes, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => nes.read_chr_rom(addr),
            0x2000..=0x2FFF => {
                self.read_vram(addr)
            },
            0x3000..=0x3EFF => { // mirrors of 0x2000 - 0x2eff
                self.read_vram(addr)
            },
            _ => {
                panic!("Out of PPU's addressing range: 0x{:X}", addr)
            },
        }
    }

    // ref. https://wiki.nesdev.com/w/index.php/PPU_memory_map
    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => {
                //panic!("Write access is forbidden: PPU's 0x{:X}", addr),
            },
            0x2000..=0x2FFF => {
                self.write_vram(addr, data);
            },
            0x3000..=0x3EFF => { // mirrors of 0x2000 - 0x2eff
                self.write_vram(addr, data);
            },
            _ => {
                panic!("Out of PPU's addressing range: 0x{:X}", addr)
            },
        }
    }

    fn increment_ppu_address(&mut self, mem: &mut Memory) {
        // TODO: Consider PPUCTRL (bit 2 of 0x2000)
        let lower_addr = self.read_register(mem, Register::PPUADDR);
        self.write_register(mem, Register::PPUADDR, lower_addr.wrapping_add(1));
    }

    fn operate_vram(&mut self, nes: &mut Nes, mem: &mut Memory) {
        match mem.ppu_addr_state() {
            PpuAddrState::Higher => {
                self.ppu_addr = self.ppu_addr | (self.read_register(mem, Register::PPUADDR) as u16) << 8;
            },
            PpuAddrState::Lower => {
                self.ppu_addr = self.ppu_addr | self.read_register(mem, Register::PPUADDR) as u16
            },
            PpuAddrState::None => {},
        }

        match mem.ppu_data_state() {
            PpuDataState::Read => {
                let data = self.read(nes, self.ppu_addr);
                self.write_register(mem, Register::PPUDATA, data);
                self.increment_ppu_address(mem);
            },
            PpuDataState::Written => {
                self.write(self.ppu_addr, self.read_register(mem, Register::PPUDATA));
                self.increment_ppu_address(mem);
            },
            PpuDataState::None => {},
        }
    }

    fn render(&mut self, nes: &mut Nes, mem: &Memory) {
        if self.batch_counter >= RENDERING_BATCH_NUM {
            return
        }
        if self.cycle_counter < CYCLES_PER_SCANLINE * RENDERING_BATCH_LINES * (self.batch_counter + 1) {
            return
        }

        self.render_batch_lines(nes);
        self.batch_counter += 1;
    }

    fn render_batch_lines(&mut self, nes: &mut Nes) {
        const COLORS: [[u8; 3]; 4] = [[0, 0, 0], [63, 63, 63], [127, 127, 127], [255, 255, 255]];

        let offset = self.batch_counter * (RENDERING_BATCH_LINES/16);
        for i in 0..(RENDERING_BATCH_LINES/16) {
            let sprite_id = self.read(nes, (0x2000+offset+i) as u16);
            let sprite = nes.get_sprite(sprite_id);

            let offset_x = (i * SPRITE_WIDTH) % VISIBLE_SCREEN_WIDTH;
            let offset_y = i / (VISIBLE_SCREEN_WIDTH / SPRITE_WIDTH) * SPRITE_HEIGHT;

            for x in 0..SPRITE_WIDTH {
                for y in 0..SPRITE_HEIGHT {
                    self.screen[offset_y + y][offset_x + x] = COLORS[sprite.get(x, y) as usize];
                }
            }
        }
    }
}
