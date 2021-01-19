use super::instruction::Instruction;
use super::instruction::Opcode;
use super::instruction::Addressing;
use super::nes::Nes;

const RAM_SIZE: usize = 0x0800;
const PRG_ROM_BASE: u16 = 0x8000;

#[derive(Debug)]
pub enum Flag {
    Carry,
    Zero,
    InterruptDisable,
    Decimal,
    Overflow,
    Negative,
    Break,
}

impl From<Flag> for u8 {
    fn from(f: Flag) -> Self {
        match f {
            Flag::Carry             => 0b00000001,
            Flag::Zero              => 0b00000010,
            Flag::InterruptDisable  => 0b00000100,
            Flag::Decimal           => 0b00001000,
            Flag::Break             => 0b00010000,
            Flag::Overflow          => 0b01000000,
            Flag::Negative          => 0b10000000,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Interruption {
    RESET,
    IRQ,
    BRK,
    NMI,
    None,
}

pub struct Cpu {
    // Registers
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    s: u16,
    status: u8, // P

    ram: [u8; RAM_SIZE],
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: PRG_ROM_BASE,
            s: 0x00fd,
            status: 0x34,
            ram: [0; RAM_SIZE],
        }
    }

    pub fn tick(&mut self, nes: &mut Nes) -> usize {
        let cycle = self.execute_instruction(nes);
        self.interrupt(nes);
        cycle
    }

    fn interrupt(&mut self, nes: &mut Nes) {
        match nes.cpu_interruption {
            Interruption::RESET => { debug!("CPU RESET interruption is not implemented yet") },
            Interruption::IRQ => { debug!("CPU IRQ interruption is not implemented yet") },
            Interruption::BRK => {
                if self.read_flag(Flag::InterruptDisable) {
                    return
                }
                debug!("BRK interruption: Jump to 0x{:02X}{:02X}",
                       self.read(nes, 0xFFFF), self.read(nes,0xFFFE));
                self.push_word(self.pc);
                self.push_byte(self.status);
                self.status = self.status | u8::from(Flag::InterruptDisable);

                self.pc = (self.read(nes, 0xFFFF) as u16) << 8 | self.read(nes, 0xFFFE) as u16;
            },
            Interruption::NMI => { debug!("CPU NMI interruption is not implemented yet") },
            Interruption::None => {},
        }
        nes.cpu_interruption = Interruption::None;
    }

    fn execute_instruction(&mut self, nes: &mut Nes) -> usize {
        let inst: Instruction = self.fetch_byte(nes).into();

        let Instruction(opcode, addressing, cycle) = inst;

        let additional_cycle = match opcode {
            Opcode::ASL => self.instruction_asl(nes, addressing),
            Opcode::BNE => self.instruction_bne(nes, addressing),
            Opcode::BRK => self.instruction_brk(nes, addressing),
            Opcode::BVC => self.instruction_bvc(nes, addressing),
            Opcode::CLD => self.instruction_cld(nes, addressing),
            Opcode::DEC => self.instruction_dec(nes, addressing),
            Opcode::DEY => self.instruction_dey(nes, addressing),
            Opcode::INX => self.instruction_inx(nes, addressing),
            Opcode::ISC => self.instruction_isc(nes, addressing),
            Opcode::JMP => self.instruction_jmp(nes, addressing),
            Opcode::JSR => self.instruction_jsr(nes, addressing),
            Opcode::LDA => self.instruction_lda(nes, addressing),
            Opcode::LDX => self.instruction_ldx(nes, addressing),
            Opcode::LDY => self.instruction_ldy(nes, addressing),
            Opcode::NOP => self.instruction_nop(nes, addressing),
            Opcode::SEI => self.instruction_sei(nes, addressing),
            Opcode::STA => self.instruction_sta(nes, addressing),
            Opcode::TXS => self.instruction_txs(nes, addressing),

            // Unofficial instructions
            Opcode::SLO => self.instruction_slo(nes, addressing),
            _ => {
                self.dump();
                panic!("unknown opcode `{}` at 0x{:X}", opcode, self.pc-1)
            }
        };

        cycle + additional_cycle
    }

    fn dump(&self) {
        println!("Cpu {{");
        println!("  a  = {:02X}", self.a);
        println!("  x  = {:02X}", self.x);
        println!("  y  = {:02X}", self.y);
        println!("  pc = {:04X}", self.pc);
        println!("  s  = {:04X}", self.s);
        println!("  p  = {:08b}", self.status);
        println!("}}");
    }

    // https://wiki.nesdev.com/w/index.php/CPU_memory_map
    fn read(&mut self, nes: &mut Nes, addr: u16) -> u8 {
        match addr {
            0x0000..=0x07FF => self.read_ram(addr),
            0x0800..=0x0FFF => self.read_ram(addr - 0x0800),
            0x1000..=0x17FF => self.read_ram(addr - 0x1000),
            0x1800..=0x1FFF => self.read_ram(addr - 0x1800),
            0x2000..=0x2007 => nes.ppu_register_bus.cpu_read(addr),
            0x2008..=0x401F => { warn!("Reading CPU address 0x2008-0x401F is not implemented"); 0 },
            0x4020..=0x7FFF => { warn!("Reading CPU address 0x4020-0x7FFF is not implemented"); 0 }, // 拡張ROM, 拡張RAM
            PRG_ROM_BASE..=0xFFFF => nes.read_program(addr-PRG_ROM_BASE),
        }
    }

    fn write(&mut self, nes: &mut Nes, addr: u16, data: u8) {
        match addr {
            0x0000..=0x07FF => self.write_ram(addr, data),
            0x0800..=0x0FFF => self.write_ram(addr - 0x0800, data),
            0x1000..=0x17FF => self.write_ram(addr - 0x1000, data),
            0x1800..=0x1FFF => self.write_ram(addr - 0x1800, data),
            0x2000..=0x2007 => nes.ppu_register_bus.cpu_write(addr, data),
            0x2008..=0x401F => warn!("Writing CPU address 0x2008-0x401F is not implemented"),
            0x4020..=0xFFFF => panic!("Cartridge space is read only: 0x{:X}", addr),
        }
    }

    fn read_ram(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    fn write_ram(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }

    fn read_flag(&self, f: Flag) -> bool {
        let bit: u8 = f.into();
        self.status & bit == bit
    }

    fn write_flag(&mut self, f: Flag, v: bool) {
        let bit: u8 = f.into();
        if v {
            self.status |= bit
        } else {
            self.status &= !bit
        }
    }

    fn push_byte(&mut self, data: u8) {
        self.write_ram(self.s, data);
        self.s = self.s.wrapping_sub(1);
    }

    fn push_word(&mut self, data: u16) {
        self.push_byte((data >> 8) as u8);
        self.push_byte((data & 0x00ff) as u8);
    }

    fn pop(&mut self) -> u8 {
        self.s += 1;
        self.read_ram(self.s - 1)
    }

    fn fetch_byte(&mut self, nes: &mut Nes) -> u8 {
        self.pc += 1;
        self.read(nes, self.pc-1)
    }

    fn fetch_word(&mut self, nes: &mut Nes) -> u16 {
        let l = self.fetch_byte(nes) as u16;
        let h = self.fetch_byte(nes) as u16;
        h << 8 | l
    }

    fn instruction_asl(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        let addr = match addressing {
            Addressing::ZeroPage => self.fetch_byte(nes) as u16,
            _ => panic!("Unknown ASL addressing mode: {:?}", addressing),
        };

        let data = self.read(nes, addr);
        let val = data.wrapping_shl(1);
        self.write(nes, addr, val);
        self.write_flag(Flag::Carry, data & 0b10000000 == 0b10000000);
        self.write_flag(Flag::Zero, self.a == 0);
        self.write_flag(Flag::Negative, is_negative(val));

        0
    }

    fn instruction_bne(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        if addressing != Addressing::Relative {
            panic!("Unknown BNE addressing mode: {:?}", addressing);
        }

        let val = self.fetch_byte(nes) as i8;

        let mut additional_cycle = 0;

        if !self.read_flag(Flag::Zero) {
            let addr = self.pc as i32 + val as i32;
            if (addr as u16 & 0xff00) != (self.pc & 0xff00) {
                additional_cycle += 1;
            }

            self.pc = addr as u16;
            additional_cycle += 1;
        }

        additional_cycle
    }

    fn instruction_brk(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        if addressing != Addressing::Implied {
            panic!("Unknown BRK addressing mode: {:?}", addressing);
        }

        nes.cpu_interruption = Interruption::BRK;
        self.write_flag(Flag::Break, true);

        0
    }

    fn instruction_cld(&mut self, _: &mut Nes, addressing: Addressing) -> usize {
        if addressing != Addressing::Implied {
            panic!("Invalid CLD addressing mode: {:?}", addressing);
        }

        self.write_flag(Flag::Decimal, false);

        0
    }

    fn instruction_bvc(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        if addressing != Addressing::Relative {
            panic!("Unknown BVC addressing mode: {:?}", addressing);
        }

        let val = self.fetch_byte(nes) as i8;
        let mut additional_cycle = 0;

        if !self.read_flag(Flag::Overflow) {
            let addr = self.pc as i32 + val as i32;
            if (addr as u16 & 0xff00) != (self.pc & 0xff00) {
                additional_cycle += 1;
            }

            self.pc = addr as u16;
            additional_cycle += 1;
        }

        additional_cycle
    }

    fn instruction_dec(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        let addr = match addressing {
            Addressing::ZeroPage => self.fetch_byte(nes) as u16,
            Addressing::ZeroPageX => self.fetch_byte(nes).wrapping_add(self.x) as u16,
            _ => panic!("Unknown DEC addressing mode: {:?}", addressing),
        };
        let val = self.read(nes, addr);
        let data = val.wrapping_add(!1+1);
        self.write(nes, addr, data);
        self.write_flag(Flag::Zero, data == 0);
        self.write_flag(Flag::Negative, is_negative(data));

        0
    }

    fn instruction_dey(&mut self, _: &mut Nes, _: Addressing) -> usize {
        self.y = self.y.wrapping_add(!1+1);
        self.write_flag(Flag::Zero, self.y == 0);
        self.write_flag(Flag::Negative, is_negative(self.y));

        0
    }

    fn instruction_inx(&mut self, _: &mut Nes, _: Addressing) -> usize {
        self.x = self.x.wrapping_add(1);
        self.write_flag(Flag::Zero, self.x == 0);
        self.write_flag(Flag::Negative, is_negative(self.x));

        0
    }

    // ISC = INC + SBC
    fn instruction_isc(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        let addr = match addressing {
            Addressing::AbsoluteX => self.fetch_word(nes) + self.x as u16,
            _ => panic!("Unknown ISC addressing mode: {:?}", addressing),
        };

        // INC
        let data = self.read(nes, addr);
        let incremented_val = data.wrapping_add(1);
        self.write(nes, addr, incremented_val);

        // SBC
        let c = if self.read_flag(Flag::Carry) { 0 } else { 1 };
        let (next_val, overflowed1) = self.a.overflowing_sub(incremented_val);
        let (result, overflowed2) = next_val.overflowing_sub(c);

        self.write_flag(Flag::Carry, !(is_carried(self.a, incremented_val) || is_carried(next_val, c)));
        self.write_flag(Flag::Zero, result == 0);
        self.write_flag(Flag::Overflow, overflowed1 || overflowed2);
        self.write_flag(Flag::Negative, is_negative(result));

        self.a = result;

        0
    }

    fn instruction_jmp(&mut self, nes: &mut Nes, _: Addressing) -> usize {
        let addr = self.fetch_word(nes);
        self.pc = addr;

        0
    }

    fn instruction_jsr(&mut self, nes: &mut Nes, _: Addressing) -> usize {
        let addr = self.fetch_word(nes);
        self.push_word(self.pc);
        self.pc = addr;

        0
    }

    fn instruction_lda(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        let mut additional_cycle = 0;
        let operand = match addressing {
            Addressing::Immediate => self.fetch_byte(nes),
            Addressing::Absolute => {
                let addr = self.fetch_word(nes);
                self.read(nes, addr)
            },
            Addressing::AbsoluteX => {
                let word = self.fetch_word(nes);
                let addr = word.wrapping_add(self.x as u16);
                if (word & 0xff00) != (addr & 0xff00) {
                    additional_cycle += 1;
                }
                self.read(nes, addr)
            }
            _ => panic!("Unknown LDA addressing mode: {:?}", addressing),
        };
        self.a = operand;
        self.write_flag(Flag::Zero, self.a == 0);
        self.write_flag(Flag::Negative, is_negative(self.a));

        additional_cycle
    }

    fn instruction_ldx(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        let operand = match addressing {
            Addressing::Immediate => self.fetch_byte(nes),
            _ => panic!("Unknown addressing mode: {:?}", addressing),
        };
        self.x = operand;
        self.write_flag(Flag::Zero, self.x == 0);
        self.write_flag(Flag::Negative, is_negative(self.x));

        0
    }

    fn instruction_ldy(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        let operand = match addressing {
            Addressing::Immediate => self.fetch_byte(nes),
            _ => panic!("Unknown addressing mode: {:?}", addressing),
        };
        self.y = operand;
        self.write_flag(Flag::Zero, self.y == 0);
        self.write_flag(Flag::Negative, is_negative(self.y));

        0
    }

    fn instruction_nop(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        match addressing {
            Addressing::Implied => {},
            Addressing::Immediate => { self.fetch_byte(nes); },
            _ => panic!("Unknown addressing mode: {:?}", addressing),
        }

        0
    }

    fn instruction_sei(&mut self, _: &mut Nes, _: Addressing) -> usize {
        self.write_flag(Flag::InterruptDisable, true);

        0
    }

    // ASL + ORA
    fn instruction_slo(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        let addr = match addressing {
            IndexedIndirect => {
                let addr = (self.fetch_byte(nes) + self.x) as u16;
                let l = self.read(nes, addr) as u16;
                let h = (self.read(nes, addr + 1) as u16) << 8;
                l + h
            },
            _ => panic!("Unknown SLO addressing mode: {:?}", addressing),
        };

        // ASL
        let data = self.read(nes, addr);
        let val = data.wrapping_shl(1);
        self.write_flag(Flag::Carry, data & 0b10000000 == 0b10000000);

        // ORA
        self.a = val | self.a;
        self.write_flag(Flag::Zero, self.a == 0);
        self.write_flag(Flag::Negative, is_negative(self.a));

        0

    }

    fn instruction_sta(&mut self, nes: &mut Nes, addressing: Addressing) -> usize {
        let addr = match addressing {
            Addressing::Absolute => self.fetch_word(nes),
            _ => panic!("Unknown addressing mode: {:?}", addressing),
        };
        debug!("STA {:04X} (A = {:02X})", addr, self.a);
        self.write(nes, addr, self.a);

        0
    }

    fn instruction_txs(&mut self, _: &mut Nes, _: Addressing) -> usize {
        self.s = self.x as u16;

        0
    }
}

fn is_negative(v: u8) -> bool {
    v & 0b10000000 == 0b10000000
}

fn is_carried(v1: u8, v2: u8) -> bool {
    let result = v1 as u16 + v2 as u16;
    result & 0x0100 == 0x0100
}

#[cfg(test)]
mod tests {
    use super::RAM_SIZE;
    use super::PRG_ROM_BASE;
    use super::Cpu;
    use super::Flag;
    use super::Nes;
    use super::Interruption;

    fn new_test_cpu(prg_rom: Vec<u8>) -> (Cpu, Nes) {
        (
            Cpu {
                a: 0,
                x: 0,
                y: 0,
                pc: PRG_ROM_BASE,
                s: 0x00fd,
                status: 0,
                ram: [0; RAM_SIZE],
            },
            Nes::new_for_test(prg_rom)
        )
    }

    #[test]
    fn instruction_asl() {
        // ZeroPage; Flag behavior
        let (mut cpu, mut nes) = new_test_cpu(vec![0x06, 0x10]);
        cpu.write(&mut nes, 0x10, 2);
        assert_eq!(cpu.execute_instruction(&mut nes), 5);
        assert_eq!(cpu.read(&mut nes, 0x10), 4);
        assert_eq!(cpu.read_flag(Flag::Carry), false);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![0x06, 0x10]);
        cpu.write(&mut nes, 0x10, 0b10000000);
        assert_eq!(cpu.execute_instruction(&mut nes), 5);
        assert_eq!(cpu.read(&mut nes, 0x10), 0);
        assert_eq!(cpu.read_flag(Flag::Carry), true);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![0x06, 0x10]);
        cpu.write(&mut nes, 0x10, 0b01000000);
        assert_eq!(cpu.execute_instruction(&mut nes), 5);
        assert_eq!(cpu.read(&mut nes, 0x10), 0b10000000);
        assert_eq!(cpu.read_flag(Flag::Carry), false);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_bne() {
        let (mut cpu, mut nes) = new_test_cpu(vec![0xD0, 0x03]);
        cpu.write_flag(Flag::Zero, false);
        assert_eq!(cpu.execute_instruction(&mut nes), 3);
        assert_eq!(cpu.pc, PRG_ROM_BASE + 2 + 0x03);

        let (mut cpu, mut nes) = new_test_cpu(vec![0xD0, 0x03]);
        cpu.write_flag(Flag::Zero, true);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.pc, PRG_ROM_BASE + 2);

        let (mut cpu, mut nes) = new_test_cpu(vec![0xD0, !0x03+1]);
        cpu.write_flag(Flag::Zero, false);
        assert_eq!(cpu.execute_instruction(&mut nes), 4);
        assert_eq!(cpu.pc, PRG_ROM_BASE + 2 - 0x03);
    }

    #[test]
    fn instruction_brk() {
        let (mut cpu, mut nes) = new_test_cpu(vec![0x00]);
        assert_eq!(cpu.execute_instruction(&mut nes), 7);
        assert_eq!(nes.cpu_interruption, Interruption::BRK);
        assert_eq!(cpu.read_flag(Flag::Break), true);
    }

    #[test]
    fn instruction_bvc() {
        let (mut cpu, mut nes) = new_test_cpu(vec![0x50, 0x03]);
        cpu.write_flag(Flag::Overflow, false);
        assert_eq!(cpu.execute_instruction(&mut nes), 3);
        assert_eq!(cpu.pc, PRG_ROM_BASE + 2 + 0x03);

        let (mut cpu, mut nes) = new_test_cpu(vec![0x50, 0x03]);
        cpu.write_flag(Flag::Overflow, true);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.pc, PRG_ROM_BASE + 2);

        let (mut cpu, mut nes) = new_test_cpu(vec![0x50, !0x03+1]);
        cpu.write_flag(Flag::Overflow, false);
        assert_eq!(cpu.execute_instruction(&mut nes), 4);
        assert_eq!(cpu.pc, PRG_ROM_BASE + 2 - 0x03);
    }

    #[test]
    fn instruction_cld() {
        let (mut cpu, mut nes) = new_test_cpu(vec![0xD8]);
        cpu.write_flag(Flag::Decimal, true);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.read_flag(Flag::Decimal), false);
    }

    #[test]
    fn instruction_dec() {
        // ZeroPage; Flag behavior
        let (mut cpu, mut nes) = new_test_cpu(vec![0xC6, 0x10]);
        cpu.write(&mut nes, 0x0010, 0x03);
        assert_eq!(cpu.execute_instruction(&mut nes), 5);
        assert_eq!(cpu.read(&mut nes, 0x0010), 0x02);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![0xC6, 0x10]);
        cpu.write(&mut nes, 0x0010, 0x01);
        assert_eq!(cpu.execute_instruction(&mut nes), 5);
        assert_eq!(cpu.read(&mut nes, 0x0010), 0x00);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![0xC6, 0x10]);
        cpu.write(&mut nes, 0x0010, 0x00);
        assert_eq!(cpu.execute_instruction(&mut nes), 5);
        assert_eq!(cpu.read(&mut nes, 0x0010), !0x01+1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);

        // ZeroPage, X
        let (mut cpu, mut nes) = new_test_cpu(vec![0xD6, 0x10]);
        cpu.write(&mut nes, 0x0011, 0x03);
        cpu.x = 0x01;
        assert_eq!(cpu.execute_instruction(&mut nes), 6);
        assert_eq!(cpu.read(&mut nes, 0x0011), 0x02);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);
    }

    #[test]
    fn instruction_dey() {
        let (mut cpu, mut nes) = new_test_cpu(vec![0x88]);
        cpu.y = 0x03;
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.y, 0x02);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![0x88]);
        cpu.y = 0x01;
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![0x88]);
        cpu.y = 0x00;
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.y, !1+1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_inx() {
        let (mut cpu, mut nes) = new_test_cpu(vec![0xE8]);
        cpu.x = 0x03;
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.x, 0x04);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![0xE8]);
        cpu.x = !1 + 1;
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![0xE8]);
        cpu.x = !3 + 1;
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.x, !2+1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_jmp() {
        // Absolute
        let (mut cpu, mut nes) = new_test_cpu(vec![0x4C, 0x03, 0x01]);
        assert_eq!(cpu.execute_instruction(&mut nes), 3);
        assert_eq!(cpu.pc, 0x0103);
    }

    #[test]
    fn instruction_jsr() {
        let (mut cpu, mut nes) = new_test_cpu(vec![0x20, 0x09, 0x90]);
        assert_eq!(cpu.execute_instruction(&mut nes), 6);
        assert_eq!(cpu.pc, 0x9009);
        assert_eq!(cpu.read(&mut nes, cpu.s+2), (PRG_ROM_BASE >> 8) as u8);
        assert_eq!(cpu.read(&mut nes, cpu.s+1), (PRG_ROM_BASE & 0x00ff) as u8 + 3);
    }

    #[test]
    fn instruction_lda() {
        // Test flag behavior
        let (mut cpu, mut nes) = new_test_cpu(vec![0xA9, 3]);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.a, 3);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![0xA9, 0]);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![0xA9, !3 + 1]);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.a, !3 + 1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);

        // Immediate: Omission

        // Absolute
        let (mut cpu, mut nes) = new_test_cpu(vec![0xAD, 0x01, 0x10]);
        cpu.write(&mut nes, 0x1001, 3);
        assert_eq!(cpu.execute_instruction(&mut nes), 4);
        assert_eq!(cpu.a, 3);

        // Absolute X
        let (mut cpu, mut nes) = new_test_cpu(vec![0xBD, 0x10, 0x10]);
        cpu.write(&mut nes, 0x1011, 3);
        cpu.x = 0x01;
        assert_eq!(cpu.execute_instruction(&mut nes), 4);
        assert_eq!(cpu.a, 3);

        let (mut cpu, mut nes) = new_test_cpu(vec![0xBD, 0xFF, 0x10]);
        cpu.write(&mut nes, 0x1100, 3);
        cpu.x = 0x01;
        assert_eq!(cpu.execute_instruction(&mut nes), 5);
        assert_eq!(cpu.a, 3);
    }

    #[test]
    fn instruction_ldx_immediate() {
        let opcode = 0xa2;

        let (mut cpu, mut nes) = new_test_cpu(vec![opcode, 3]);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.x, 3);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![opcode, 0]);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![opcode, !3 + 1]);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.x, !3 + 1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_ldy_immediate() {
        let opcode = 0xa0;

        let (mut cpu, mut nes) = new_test_cpu(vec![opcode, 3]);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.y, 3);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![opcode, 0]);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut nes) = new_test_cpu(vec![opcode, !3 + 1]);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.y, !3 + 1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_sei_implied() {
        let (mut cpu, mut nes) = new_test_cpu(vec![0x78]);
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.read_flag(Flag::InterruptDisable), true);
    }

    #[test]
    fn instruction_slo() {
        // TODO
    }

    #[test]
    fn instruction_sta_absolute() {
        let opcode = 0x8d;

        let (mut cpu, mut nes) = new_test_cpu(vec![opcode, 0x11, 0x01]);
        cpu.a = 3;
        assert_eq!(cpu.execute_instruction(&mut nes), 4);
        assert_eq!(cpu.read(&mut nes, 0x0111), 3);
    }

    #[test]
    fn instruction_txs_implied() {
        let (mut cpu, mut nes) = new_test_cpu(vec![0x9a]);
        cpu.x = 3;
        assert_eq!(cpu.execute_instruction(&mut nes), 2);
        assert_eq!(cpu.s, 3);
    }
}
