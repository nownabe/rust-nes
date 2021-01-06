use super::memory::Memory;
use super::memory::PpuAddrState;
use super::memory::PpuDataState;

const VRAM_SIZE: usize = 0x0800;
const OAM_SIZE: usize = 0x0100;
const CHARACTER_ROM_SIZE: usize = 0x2000;

const VISIBLE_SCREEN_WIDTH: usize = 256;
const VISIBLE_SCREEN_HEIGHT: usize = 240;

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

#[derive(Clone, Copy, Debug)]
pub struct Sprite {
    data: [[u8; 8]; 8],
}

impl Sprite {
    pub fn get(&self, x: usize, y: usize) -> u8 {
        self.data[y][x]
    }
}

pub struct Ppu {
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    ppu_addr: u16,
    cycle_counter: usize,
    batch_counter: usize,
    pub character_rom: [u8; CHARACTER_ROM_SIZE],
    screen: [[[u8; 3]; VISIBLE_SCREEN_WIDTH]; VISIBLE_SCREEN_HEIGHT],
    sprites: [Sprite; 0x200],
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            ppu_addr: 0,
            cycle_counter: 0,
            batch_counter: 0,
            character_rom: [0; CHARACTER_ROM_SIZE],
            screen: [[[0; 3]; VISIBLE_SCREEN_WIDTH]; VISIBLE_SCREEN_HEIGHT],
            sprites: [Sprite{data:[[0;8]; 8]}; 0x200],
        }
    }

    // https://wiki.nesdev.com/w/index.php/PPU_power_up_state
    pub fn init(&mut self, mem: &mut Memory, character_rom: Vec<u8>) {
        self.write_register(mem, Register::PPUCTRL, 0x00);
        self.write_register(mem, Register::PPUMASK, 0x00);
        self.write_register(mem, Register::PPUSTATUS, 0b10100000);
        self.write_register(mem, Register::OAMADDR, 0x00);
        self.write_register(mem, Register::PPUSCROLL, 0x00);
        self.write_register(mem, Register::PPUADDR, 0x00);
        self.write_register(mem, Register::PPUDATA, 0x00);

        for i in 0..character_rom.len() {
            self.character_rom[i] = character_rom[i];
        }

        for i in 0..(character_rom.len()/16) {
            for j in 0..8 {
                for k in 0..8 {
                    self.sprites[i].data[j][k] = (character_rom[i*16+j] & (1<<(7-k))) >> (7-k);
                    self.sprites[i].data[j][k] += (character_rom[i*16+j+8] & (1<<(7-k))) >> (7-k) << 1;
                }
            }
        }

    }

    pub fn get_sprite(&self, id: u8) -> Sprite {
        self.sprites[id as usize]
    }

    pub fn step(&mut self, mem: &mut Memory, cpu_cycle: usize) {
        self.cycle_counter += cpu_cycle * 3;

        self.operate_vram(mem);
        self.render(mem);

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
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.character_rom[addr as usize],
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
            0x0000..=0x1FFF => panic!("Write access is forbidden: PPU's 0x{:X}", addr),
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
        self.write_register(mem, Register::PPUADDR, lower_addr + 1);
    }

    fn operate_vram(&mut self, mem: &mut Memory) {
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
                let data = self.read(self.ppu_addr);
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

    fn render(&mut self, mem: &Memory) {
        if self.batch_counter >= RENDERING_BATCH_NUM {
            return
        }
        if self.cycle_counter < CYCLES_PER_SCANLINE * RENDERING_BATCH_LINES * (self.batch_counter + 1) {
            return
        }

        self.render_batch_lines(mem);
        self.batch_counter += 1;
    }

    fn render_batch_lines(&mut self, mem: &Memory) {
        let offset = self.batch_counter * (RENDERING_BATCH_LINES/16);
        for i in 0..(RENDERING_BATCH_LINES/16) {
            let sprite_id = self.read((0x2000+offset+i) as u16);
        }
    }
}
