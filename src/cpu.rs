use super::instruction::Instruction;
use super::instruction::Opcode;
use super::instruction::Addressing;

const MEMORY_SIZE: usize = 0x10000;
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

    // Memory
    memory: [u8; MEMORY_SIZE],

    // State
    instruction_cycle: u8,
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
            memory: [0; MEMORY_SIZE],
            instruction_cycle: 0,
        }
    }

    pub fn load_program(&mut self, data: Vec<u8>) {
        for i in 0..data.len() {
            self.memory[MEMORY_PROGRAM_OFFSET + i] = data[i];
        }
    }

    pub fn tick(&mut self) {
        if self.instruction_cycle == 0 {
            let inst: Instruction = self.fetch_byte().into();
            let Instruction(opcode, addr, cycle) = inst;
            self.instruction_cycle = cycle;
            self.execute_instruction(opcode, addr);
        }
        self.instruction_cycle -= 1;
    }

    fn fetch_byte(&mut self) -> u8 {
        self.pc += 1;
        self.memory[(self.pc-1) as usize]
    }


    fn fetch_word(&mut self) -> u16 {
        let l = self.fetch_byte() as u16;
        let h = self.fetch_byte() as u16;
        h << 8 | l
    }

    fn execute_instruction(&mut self, op: Opcode, addressing: Addressing) {
        match op {
            Opcode::LDA => self.instruction_lda(addressing),
            Opcode::LDX => self.instruction_ldx(addressing),
            Opcode::LDY => self.instruction_ldy(addressing),
            Opcode::SEI => self.instruction_sei(addressing),
            Opcode::STA => self.instruction_sta(addressing),
            Opcode::TXS => self.instruction_txs(addressing),
            _ => {
                self.dump();
                panic!("unknown opcode: {}", op)
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

    fn instruction_lda(&mut self, addressing: Addressing) {
        let operand = match addressing {
            Addressing::Immediate => self.fetch_byte(),
            _ => panic!("Unknown addressing mode: {:?}", addressing),
        };
        self.a = operand;
        self.write_flag(Flag::Zero, self.a == 0);
        self.write_flag(Flag::Negative, is_negative(self.a));
    }

    fn instruction_ldx(&mut self, addressing: Addressing) {
        let operand = match addressing {
            Addressing::Immediate => self.fetch_byte(),
            _ => panic!("Unknown addressing mode: {:?}", addressing),
        };
        self.x = operand;
        self.write_flag(Flag::Zero, self.x == 0);
        self.write_flag(Flag::Negative, is_negative(self.x));
    }

    fn instruction_ldy(&mut self, addressing: Addressing) {
        let operand = match addressing {
            Addressing::Immediate => self.fetch_byte(),
            _ => panic!("Unknown addressing mode: {:?}", addressing),
        };
        self.y = operand;
        self.write_flag(Flag::Zero, self.y == 0);
        self.write_flag(Flag::Negative, is_negative(self.y));
    }

    fn instruction_sei(&mut self, _: Addressing) {
        self.write_flag(Flag::InterruptDisable, true)
    }

    fn instruction_sta(&mut self, addressing: Addressing) {
        let addr = match addressing {
            Addressing::Absolute => self.fetch_word() as usize,
            _ => panic!("Unknown addressing mode: {:?}", addressing),
        };
        self.memory[addr] = self.a;
    }

    fn instruction_txs(&mut self, _: Addressing) {
        self.s = self.x;
    }
}

fn is_negative(v: u8) -> bool {
    v & 0b10000000 == 0b10000000
}

#[cfg(test)]
mod tests {
    use super::MEMORY_SIZE;
    use super::MEMORY_PROGRAM_OFFSET;
    use super::Cpu;
    use super::Flag;

    fn new_test_cpu() -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: MEMORY_PROGRAM_OFFSET as u16,
            s: 0,
            status: 0,
            memory: [0; MEMORY_SIZE],
            instruction_cycle: 0,
        }
    }

    #[test]
    fn instruction_lda_immediate() {
        let opcode = 0xa9;

        let mut cpu = new_test_cpu();
        cpu.load_program(vec![opcode, 3]);
        cpu.tick();
        assert_eq!(cpu.a, 3);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let mut cpu = new_test_cpu();
        cpu.load_program(vec![opcode, 0]);
        cpu.tick();
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let mut cpu = new_test_cpu();
        cpu.load_program(vec![opcode, !3 + 1]);
        cpu.tick();
        assert_eq!(cpu.a, !3 + 1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_ldx_immediate() {
        let opcode = 0xa2;

        let mut cpu = new_test_cpu();
        cpu.load_program(vec![opcode, 3]);
        cpu.tick();
        assert_eq!(cpu.x, 3);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let mut cpu = new_test_cpu();
        cpu.load_program(vec![opcode, 0]);
        cpu.tick();
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let mut cpu = new_test_cpu();
        cpu.load_program(vec![opcode, !3 + 1]);
        cpu.tick();
        assert_eq!(cpu.x, !3 + 1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_ldy_immediate() {
        let opcode = 0xa0;

        let mut cpu = new_test_cpu();
        cpu.load_program(vec![opcode, 3]);
        cpu.tick();
        assert_eq!(cpu.y, 3);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let mut cpu = new_test_cpu();
        cpu.load_program(vec![opcode, 0]);
        cpu.tick();
        assert_eq!(cpu.y, 0);
        assert_eq!(cpu.read_flag(Flag::Zero), true);
        assert_eq!(cpu.read_flag(Flag::Negative), false);

        let mut cpu = new_test_cpu();
        cpu.load_program(vec![opcode, !3 + 1]);
        cpu.tick();
        assert_eq!(cpu.y, !3 + 1);
        assert_eq!(cpu.read_flag(Flag::Zero), false);
        assert_eq!(cpu.read_flag(Flag::Negative), true);
    }

    #[test]
    fn instruction_sei_implied() {
        let mut cpu = new_test_cpu();
        cpu.load_program(vec![0x78]);
        cpu.tick();
        assert_eq!(cpu.read_flag(Flag::InterruptDisable), true);
    }

    #[test]
    fn instruction_sta_absolute() {
        let opcode = 0x8d;

        let mut cpu = new_test_cpu();
        cpu.load_program(vec![opcode, 0x01, 0x11]);
        cpu.a = 3;
        cpu.tick();
        assert_eq!(cpu.memory[0x0111], 3);
    }

    #[test]
    fn instruction_txs_implied() {
        let mut cpu = new_test_cpu();
        cpu.load_program(vec![0x9a]);
        cpu.x = 3;
        cpu.tick();
        assert_eq!(cpu.s, 3);
    }
}
