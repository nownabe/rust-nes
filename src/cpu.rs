use super::instruction::Instruction;
use super::instruction::Opcode;
use super::instruction::Addressing;
use super::memory::Memory;

const MEMORY_PROGRAM_OFFSET: usize = 0x8000;

#[derive(Debug)]
pub enum Flag {
    Carry,
    Zero,
    InterruptDisable,
    Decimal,
    Overflow,
    Negative,
}

impl From<Flag> for u8 {
    fn from(f: Flag) -> Self {
        match f {
            Flag::Carry => 0b00000001,
            Flag::Zero => 0b0000010,
            Flag::InterruptDisable => 0b0000100,
            Flag::Decimal => 0b00001000,
            Flag::Overflow => 0b01000000,
            Flag::Negative => 0b10000000,
        }
    }
}

pub struct Cpu {
    // Registers
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    s: u8,
    status: u8, // P

    // State
    instruction_cycle: usize,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: MEMORY_PROGRAM_OFFSET as u16,
            s: 0xfd,
            status: 0x34,
            instruction_cycle: 0,
        }
    }

    pub fn load_program(&mut self, mem: &mut Memory, data: Vec<u8>) {
        for i in 0..data.len() {
            mem.write(MEMORY_PROGRAM_OFFSET + i, data[i]);
        }
    }

    pub fn tick(&mut self, mem: &mut Memory) -> usize {
        // TODO: Refactor around instruction_cycle
        //       execute_instruction should return cycle directly.
        //       not to use struct field.
        self.instruction_cycle = 0;
        self.execute_instruction(mem);
        self.instruction_cycle
    }

    fn fetch_byte(&mut self, mem: &mut Memory) -> u8 {
        self.pc += 1;
        mem.read((self.pc-1) as usize)
    }


    fn fetch_word(&mut self, mem: &mut Memory) -> u16 {
        let l = self.fetch_byte(mem) as u16;
        let h = self.fetch_byte(mem) as u16;
        h << 8 | l
    }

    fn execute_instruction(&mut self, mem: &mut Memory) {
        let inst: Instruction = self.fetch_byte(mem).into();

        let Instruction(opcode, addressing, cycle) = inst;
        self.instruction_cycle = cycle;

        match opcode {
            Opcode::BNE => self.instruction_bne(mem, addressing),
            Opcode::DEC => self.instruction_dec(mem, addressing),
            Opcode::DEY => self.instruction_dey(mem, addressing),
            Opcode::INX => self.instruction_inx(mem, addressing),
            Opcode::JMP => self.instruction_jmp(mem, addressing),
            Opcode::LDA => self.instruction_lda(mem, addressing),
            Opcode::LDX => self.instruction_ldx(mem, addressing),
            Opcode::LDY => self.instruction_ldy(mem, addressing),
            Opcode::SEI => self.instruction_sei(mem, addressing),
            Opcode::STA => self.instruction_sta(mem, addressing),
            Opcode::TXS => self.instruction_txs(mem, addressing),
            _ => {
                self.dump();
                panic!("unknown opcode `{}` at 0x{:X}", opcode, self.pc-1)
            }
        }
    }

    pub fn read_flag(&self, f: Flag) -> bool {
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

    fn dump(&self) {
        println!("Cpu {{");
        println!("  a  = {}", self.a);
        println!("  x  = {}", self.x);
        println!("  y  = {}", self.y);
        println!("  pc = {}", self.pc);
        println!("  s  = {}", self.s);
        println!("  p  = {}", self.status);
        println!("}}");
    }

    fn instruction_bne(&mut self, mem: &mut Memory, addressing: Addressing) {
        if addressing != Addressing::Relative {
            panic!("Unknown BNE addressing mode: {:?}", addressing);
        }

        let val = self.fetch_byte(mem) as i8;

        if self.read_flag(Flag::Zero) {
            let addr = self.pc as i32 + val as i32;
            if (addr as u16 & 0xff00) != (self.pc & 0xff00) {
                self.instruction_cycle += 1;
            }

            self.pc = addr as u16;
            self.instruction_cycle += 1;
        }
    }

    fn instruction_dec(&mut self, mem: &mut Memory, addressing: Addressing) {
        let addr = match addressing {
            Addressing::ZeroPage => self.fetch_byte(mem) as usize,
            Addressing::ZeroPageX => self.fetch_byte(mem).wrapping_add(self.x) as usize,
            _ => panic!("Unknown DEC addressing mode: {:?}", addressing),
        };
        let val = mem.read(addr);
        let data = val.wrapping_add(!1+1);
        mem.write(addr, data);
        self.write_flag(Flag::Zero, data == 0);
        self.write_flag(Flag::Negative, is_negative(data));
    }

    fn instruction_dey(&mut self, _: &mut Memory, _: Addressing) {
        self.y = self.y.wrapping_add(!1+1);
        self.write_flag(Flag::Zero, self.y == 0);
        self.write_flag(Flag::Negative, is_negative(self.y));
    }

    fn instruction_inx(&mut self, _: &mut Memory, _: Addressing) {
        self.x = self.x.wrapping_add(1);
        self.write_flag(Flag::Zero, self.x == 0);
        self.write_flag(Flag::Negative, is_negative(self.x));
    }

    fn instruction_jmp(&mut self, mem: &mut Memory, _: Addressing) {
        let addr = self.fetch_word(mem);
        self.pc = addr;
    }

    fn instruction_lda(&mut self, mem: &mut Memory, addressing: Addressing) {
        let operand = match addressing {
            Addressing::Immediate => self.fetch_byte(mem),
            Addressing::AbsoluteX => {
                let word = self.fetch_word(mem);
                let addr = word.wrapping_add(self.x as u16);
                if (word & 0xff00) != (addr & 0xff00) {
                    self.instruction_cycle += 1;
                }
                mem.read(addr as usize)
            }
            _ => panic!("Unknown LDA addressing mode: {:?}", addressing),
        };
        self.a = operand;
        self.write_flag(Flag::Zero, self.a == 0);
        self.write_flag(Flag::Negative, is_negative(self.a));
    }

    fn instruction_ldx(&mut self, mem: &mut Memory, addressing: Addressing) {
        let operand = match addressing {
            Addressing::Immediate => self.fetch_byte(mem),
            _ => panic!("Unknown addressing mode: {:?}", addressing),
        };
        self.x = operand;
        self.write_flag(Flag::Zero, self.x == 0);
        self.write_flag(Flag::Negative, is_negative(self.x));
    }

    fn instruction_ldy(&mut self, mem: &mut Memory, addressing: Addressing) {
        let operand = match addressing {
            Addressing::Immediate => self.fetch_byte(mem),
            _ => panic!("Unknown addressing mode: {:?}", addressing),
        };
        self.y = operand;
        self.write_flag(Flag::Zero, self.y == 0);
        self.write_flag(Flag::Negative, is_negative(self.y));
    }

    fn instruction_sei(&mut self, _: &mut Memory, _: Addressing) {
        self.write_flag(Flag::InterruptDisable, true)
    }

    fn instruction_sta(&mut self, mem: &mut Memory, addressing: Addressing) {
        let addr = match addressing {
            Addressing::Absolute => self.fetch_word(mem) as usize,
            _ => panic!("Unknown addressing mode: {:?}", addressing),
        };
        mem.write(addr, self.a);
    }

    fn instruction_txs(&mut self, _: &mut Memory, _: Addressing) {
        self.s = self.x;
    }
}

fn is_negative(v: u8) -> bool {
    v & 0b10000000 == 0b10000000
}

#[cfg(test)]
mod tests {
    use super::MEMORY_PROGRAM_OFFSET;
    use super::Cpu;
    use super::Flag;
    use super::Memory;

    fn new_test_cpu() -> (Cpu, Memory) {
        (Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: MEMORY_PROGRAM_OFFSET as u16,
            s: 0,
            status: 0,
            instruction_cycle: 0,
        },
        Memory::new()
        )
    }

    #[test]
    fn instruction_bne() {
        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xD0, 0x03]);
        cpu.write_flag(Flag::Zero, true);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.instruction_cycle, 3);
        assert_eq!(cpu.pc, MEMORY_PROGRAM_OFFSET as u16 + 2 + 0x03);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xD0, 0x03]);
        cpu.write_flag(Flag::Zero, false);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.instruction_cycle, 2);
        assert_eq!(cpu.pc, MEMORY_PROGRAM_OFFSET as u16 + 2);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xD0, !0x03+1]);
        cpu.write_flag(Flag::Zero, true);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.instruction_cycle, 4);
        assert_eq!(cpu.pc, MEMORY_PROGRAM_OFFSET as u16 + 2 - 0x03);
    }

    #[test]
    fn instruction_dec() {
        // ZeroPage; Flag behavior
        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xC6, 0x10]);
        mem.write(0x0010, 0x03);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.instruction_cycle, 5);
        assert_eq!(mem.read(0x0010), 0x02);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xC6, 0x10]);
        mem.write(0x0010, 0x01);
        cpu.execute_instruction(&mut mem);
        assert_eq!(mem.read(0x0010), 0x00);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xC6, 0x10]);
        mem.write(0x0010, 0x00);
        cpu.execute_instruction(&mut mem);
        assert_eq!(mem.read(0x0010), !0x01+1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);

        // ZeroPage, X
        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xD6, 0x10]);
        mem.write(0x0011, 0x03);
        cpu.x = 0x01;
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.instruction_cycle, 6);
        assert_eq!(mem.read(0x0011), 0x02);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);
    }

    #[test]
    fn instruction_dey() {
        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0x88]);
        cpu.y = 0x03;
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.instruction_cycle, 2);
        assert_eq!(cpu.y, 0x02);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0x88]);
        cpu.y = 0x01;
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.y, 0x00);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0x88]);
        cpu.y = 0x00;
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.y, !1+1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_inx() {
        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xE8]);
        cpu.x = 0x03;
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.instruction_cycle, 2);
        assert_eq!(cpu.x, 0x04);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xE8]);
        cpu.x = !1 + 1;
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.x, 0x00);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xE8]);
        cpu.x = !3 + 1;
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.x, !2+1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_jmp() {
        // Absolute
        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0x4C, 0x03, 0x01]);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.instruction_cycle, 3);
        assert_eq!(cpu.pc, 0x0103);
    }

    #[test]
    fn instruction_lda() {
        // Test flag behavior
        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xA9, 3]);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.a, 3);
        assert_eq!(cpu.instruction_cycle, 2);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xA9, 0]);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xA9, !3 + 1]);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.a, !3 + 1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);

        // Immediate: Omission

        // Absolute X
        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xBD, 0x10, 0x10]);
        mem.write(0x1011, 3);
        cpu.x = 0x01;
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.a, 3);
        assert_eq!(cpu.instruction_cycle, 4);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0xBD, 0xFF, 0x10]);
        mem.write(0x1100, 3);
        cpu.x = 0x01;
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.a, 3);
        assert_eq!(cpu.instruction_cycle, 5);
    }

    #[test]
    fn instruction_ldx_immediate() {
        let opcode = 0xa2;

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![opcode, 3]);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.x, 3);
        assert_eq!(cpu.instruction_cycle, 2);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![opcode, 0]);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![opcode, !3 + 1]);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.x, !3 + 1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_ldy_immediate() {
        let opcode = 0xa0;

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![opcode, 3]);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.y, 3);
        assert_eq!(cpu.instruction_cycle, 2);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![opcode, 0]);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![opcode, !3 + 1]);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.y, !3 + 1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_sei_implied() {
        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0x78]);
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.instruction_cycle, 2);
        assert_eq!(cpu.read_flag(Flag::InterruptDisable), true);
    }

    #[test]
    fn instruction_sta_absolute() {
        let opcode = 0x8d;

        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![opcode, 0x11, 0x01]);
        cpu.a = 3;
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.instruction_cycle, 4);
        assert_eq!(mem.read(0x0111), 3);
    }

    #[test]
    fn instruction_txs_implied() {
        let (mut cpu, mut mem) = new_test_cpu();
        cpu.load_program(&mut mem, vec![0x9a]);
        cpu.x = 3;
        cpu.execute_instruction(&mut mem);
        assert_eq!(cpu.instruction_cycle, 2);
        assert_eq!(cpu.s, 3);
    }
}
