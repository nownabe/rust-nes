use std::convert::From;

// https://www.masswerk.at/6502/6502_instruction_set.html
#[derive(Debug)]
pub struct Instruction(pub Opcode, pub Addressing);

#[derive(Debug)]
pub enum Opcode {
    LDA,
    LDX,
    SEI,
    STA,
    TXS,

    UNKNOWN,
}

impl From<u8> for Instruction {
    fn from(opcode: u8) -> Self {
        match opcode {
            0xA9 => Instruction(Opcode::LDA, Addressing::Immediate),
            0xA2 => Instruction(Opcode::LDX, Addressing::Immediate),
            0x78 => Instruction(Opcode::SEI, Addressing::Implied),
            0x8D => Instruction(Opcode::STA, Addressing::Absolute),
            0x9A => Instruction(Opcode::TXS, Addressing::Implied),
            _ => Instruction(Opcode::UNKNOWN, Addressing::UNKNOWN),
        }
    }
}

#[derive(Debug)]
pub enum Addressing {
    Implied,
    Immediate,
    Absolute,

    UNKNOWN,
}

