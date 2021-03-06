use super::nes::Nes;
use super::cassette::SPRITE_WIDTH;
use super::cassette::SPRITE_HEIGHT;
use super::ppu_register_bus::PpuDataStatus;

const VRAM_SIZE: usize = 0x0800;
const OAM_SIZE: usize = 0x0100;

pub const VISIBLE_SCREEN_WIDTH: usize = 256;
pub const VISIBLE_SCREEN_HEIGHT: usize = 240;
const VISIBLE_SCREEN_SPRITES: usize = VISIBLE_SCREEN_WIDTH / SPRITE_WIDTH;

// https://wiki.nesdev.com/w/index.php/PPU_rendering#Line-by-line_timing
const CYCLES_PER_SCANLINE: usize = 341;
const SCANLINES_PER_FRAME: usize = 262;
const CYCLES_PER_FRAME: usize = CYCLES_PER_SCANLINE * SCANLINES_PER_FRAME;

// 16ラインずつ処理
const RENDERING_BATCH_SPRITES: usize = VISIBLE_SCREEN_SPRITES * 2;
const RENDERING_BATCH_LINES: usize = RENDERING_BATCH_SPRITES / VISIBLE_SCREEN_SPRITES * SPRITE_HEIGHT;
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

impl From<u16> for Register {
    fn from(a: u16) -> Self {
        match a {
            0x2000 => Register::PPUCTRL,
            0x2001 => Register::PPUMASK,
            0x2002 => Register::PPUSTATUS,
            0x2003 => Register::OAMADDR,
            0x2004 => Register::OAMDATA,
            0x2005 => Register::PPUSCROLL,
            0x2006 => Register::PPUADDR,
            0x2007 => Register::PPUDATA,
            0x4014 => Register::OAMDMA,
            _ => panic!("Invalid address for PPU Register: {:04X}", a),
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

    pub fn step(&mut self, nes: &mut Nes, cpu_cycle: usize) -> bool {
        self.cycle_counter += cpu_cycle * 3;

        self.handle_io(nes);
        let rendered = self.render(nes);

        if self.cycle_counter >= CYCLES_PER_FRAME {
            self.cycle_counter -= CYCLES_PER_FRAME;
            self.batch_counter = 0;
        }

        rendered
    }

    // ref. https://wiki.nesdev.com/w/index.php/PPU_memory_map
    fn read(&mut self, nes: &mut Nes, addr: u16) -> u8 {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => nes.read_chr_rom(addr as u16),
            0x2000..=0x2FFF => {
                self.vram[addr - 0x2000]
            },
            0x3000..=0x3EFF => { // mirrors of 0x2000 - 0x2eff
                self.vram[addr - 0x3000]
            },
            0x3F00..=0x3F1F => { todo!("Palette RAM is not implemented") },
            0x3F20..=0x3FFF => { todo!("Palette RAM is not implemented") },
            _ => {
                panic!("Out of PPU's addressing range: 0x{:X}", addr)
            },
        }
    }

    // ref. https://wiki.nesdev.com/w/index.php/PPU_memory_map
    fn write(&mut self, addr: u16, data: u8) {
        let addr = addr as usize;
        match addr {
            0x0000..=0x1FFF => {
                //panic!("Write access is forbidden: PPU's 0x{:X}", addr),
            },
            0x2000..=0x2FFF => {
                self.vram[addr - 0x2000] = data;
            },
            0x3000..=0x3EFF => { // mirrors of 0x2000 - 0x2eff
                self.vram[addr - 0x3000] = data;
            },
            0x3F00..=0x3F1F => { todo!("Palette RAM is not implemented") },
            0x3F20..=0x3FFF => { todo!("Palette RAM is not implemented") },
            _ => {
                panic!("Out of PPU's addressing range: 0x{:X}", addr)
            },
        }
    }

    fn increment_ppu_addr(&mut self) {
        // TODO: Consider PPUCTRL (bit 2 of 0x2000)
        self.ppu_addr = self.ppu_addr.wrapping_add(1);
    }

    fn handle_io(&mut self, nes: &mut Nes) {
        if let Some(addr) = nes.ppu_register_bus.ppu_read(Register::PPUADDR) {
            self.ppu_addr = addr;
        }

        match nes.ppu_register_bus.ppu_data_status() {
            PpuDataStatus::Read => {
                let data = self.read(nes, self.ppu_addr);
                nes.ppu_register_bus.ppu_write(Register::PPUDATA, data);
                debug!("PPU copied {:02X} into PPUDATA from VRAM[{:04X}]", data, self.ppu_addr);
                self.increment_ppu_addr();
            },
            PpuDataStatus::Written => {
                if let Some(data) = nes.ppu_register_bus.ppu_read(Register::PPUDATA) {
                    self.write(self.ppu_addr, data as u8);
                    debug!("PPU copied {:02X} from PPUDATA into VRAM[{:04X}]", data as u8, self.ppu_addr);
                }
                self.increment_ppu_addr();
            },
            PpuDataStatus::None => {},
        }
    }

    fn render(&mut self, nes: &mut Nes) -> bool {
        if self.batch_counter >= RENDERING_BATCH_NUM {
            return false
        }
        if self.cycle_counter < CYCLES_PER_SCANLINE * RENDERING_BATCH_LINES * (self.batch_counter + 1) {
            return false
        }

        self.render_batch_lines(nes);
        self.batch_counter += 1;

        true
    }

    fn render_batch_lines(&mut self, nes: &mut Nes) {
        const COLORS: [[u8; 3]; 4] = [[0, 0, 0], [63, 63, 63], [127, 127, 127], [255, 255, 255]];

        let sprite_offset = self.batch_counter * RENDERING_BATCH_SPRITES;
        for i in 0..RENDERING_BATCH_SPRITES {
            let sprite_id = self.read(nes, (0x2000+sprite_offset+i) as u16);
            let sprite = nes.get_sprite(sprite_id);

            let offset_x = i % VISIBLE_SCREEN_SPRITES * SPRITE_WIDTH;
            let offset_y = i / VISIBLE_SCREEN_SPRITES * SPRITE_HEIGHT + self.batch_counter * RENDERING_BATCH_LINES;

            for x in 0..SPRITE_WIDTH {
                for y in 0..SPRITE_HEIGHT {
                    self.screen[offset_y + y][offset_x + x] = COLORS[sprite.get(x, y) as usize];
                }
            }
        }
    }
}
