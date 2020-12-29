use super::instruction::Instruction;
use super::instruction::Opcode;
use super::instruction::Addressing;

const MEMORY_SIZE: usize = 0xffff;
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
            let instruction = self.fetch();
            self.execute_instruction(instruction.into());
        }
        self.instruction_cycle -= 1;
    }

    fn fetch(&mut self) -> u8 {
        self.pc += 1;
        self.memory[(self.pc-1) as usize]
    }


    fn fetch_word(&mut self) -> u16 {
        let h = self.fetch() as u16;
        let l = self.fetch() as u16;
        h << 8 | l
    }

    fn execute_instruction(&mut self, inst: Instruction) {
        match inst {
            Instruction(Opcode::LDA, Addressing::Immediate) => self.instruction_lda_immediate(),
            Instruction(Opcode::LDX, Addressing::Immediate) => self.instruction_ldx_immediate(),
            Instruction(Opcode::SEI, Addressing::Implied) => self.instruction_sei_implied(),
            Instruction(Opcode::STA, Addressing::Absolute) => self.instruction_sta_absolute(),
            Instruction(Opcode::TXS, Addressing::Implied) => self.instruction_txs_implied(),
            _ => panic!("unknown instruction {:?}", inst)
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

    // 0xa9
    fn instruction_lda_immediate(&mut self) {
        self.instruction_cycle = 2;
        self.a = self.fetch();
        self.write_flag(Flag::Zero, self.a == 0);
        self.write_flag(Flag::Negative, is_negative(self.a));
    }

    // 0xa2
    fn instruction_ldx_immediate(&mut self) {
        self.instruction_cycle = 2;
        self.x = self.fetch();
        self.write_flag(Flag::Zero, self.x == 0);
        self.write_flag(Flag::Negative, is_negative(self.x));
    }

    // 0x78
    fn instruction_sei_implied(&mut self) {
        self.instruction_cycle = 2;
        self.write_flag(Flag::InterruptDisable, true)
    }

    // 0x8d
    fn instruction_sta_absolute(&mut self) {
        self.instruction_cycle = 4;
        let addr = self.fetch_word() as usize;
        self.memory[addr] = self.a;
    }

    // 0x9a
    fn instruction_txs_implied(&mut self) {
        self.instruction_cycle = 2;
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
