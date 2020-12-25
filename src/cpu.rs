#[derive(Debug)]
pub enum CpuFlag {
    InterruptDisable,
}

impl From<CpuFlag> for u8 {
    fn from(f: CpuFlag) -> Self {
        match f {
            CpuFlag::InterruptDisable => 0b0000100,
        }
    }
}

#[derive(Debug)]
pub struct Cpu {
    // Registers
    a: u8,
    x: u8,
    y: u8,
    pc: u16,
    sp: u8,
    status: u8,

    // Memory
    memory: Vec<u8>,

    // State
    instruction_cycle: u8,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            pc: 0x34,
            sp: 0xfd,
            status: 0x34,
            memory: vec![],
            instruction_cycle: 0,
        }
    }

    pub fn load(&mut self, data: Vec<u8>) {
        self.memory = data;
    }

    pub fn tick(&mut self) {
        if self.instruction_cycle == 0 {
            let instruction = self.fetch();
            self.execute_instruction(instruction);
        }
        self.instruction_cycle -= 1;
    }

    fn fetch(&mut self) -> u8 {
        self.pc += 1;
        self.memory[(self.pc-1) as usize]
    }

    fn execute_instruction(&mut self, instruction: u8) {
        match instruction {
            0x78 => self.instruction_sei_implied(),
            _ => panic!("unknown instruction {:?}", instruction)
        }
    }

    pub fn read_flag(self, f: CpuFlag) -> bool {
        let bit: u8 = f.into();
        self.status & bit == bit
    }

    fn write_flag(&mut self, f: CpuFlag, v: bool) {
        let bit: u8 = f.into();
        if v {
            self.status |= bit
        } else {
            self.status &= !bit
        }
    }

    // 0x78
    fn instruction_sei_implied(&mut self) {
        self.instruction_cycle = 2;
        self.write_flag(CpuFlag::InterruptDisable, true)
    }
}

#[cfg(test)]
mod tests {
    use super::Cpu;
    use super::CpuFlag;

    fn new_test_cpu() -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            sp: 0,
            status: 0,
            memory: vec![],
            instruction_cycle: 0,
        }
    }

    #[test]
    fn instruction_sei_implied() {
        let mut cpu = new_test_cpu();
        println!("{:?}", cpu);
        cpu.load(vec![0x78]);
        cpu.tick();
        assert_eq!(cpu.read_flag(CpuFlag::InterruptDisable), true);
    }
}
