use std::convert::From;

// https://www.masswerk.at/6502/6502_instruction_set.html
// http://obelisk.me.uk/6502/reference.html
#[derive(Debug)]
pub struct Instruction(pub Opcode, pub Addressing, pub u8);

#[derive(Debug)]
pub enum Opcode {
    LDA,
    LDX,
    LDY,
    SEI,
    STA,
    TXS,

    UNKNOWN,
}

impl From<u8> for Instruction {
    fn from(opcode: u8) -> Self {
        match opcode {
            0xA9 => Instruction(Opcode::LDA, Addressing::Immediate, 2),
            0xA2 => Instruction(Opcode::LDX, Addressing::Immediate, 2),
            0xA0 => Instruction(Opcode::LDY, Addressing::Immediate, 2),
            0x78 => Instruction(Opcode::SEI, Addressing::Implied, 2),
            0x8D => Instruction(Opcode::STA, Addressing::Absolute, 4),
            0x9A => Instruction(Opcode::TXS, Addressing::Implied, 2),
            _ => Instruction(Opcode::UNKNOWN, Addressing::UNKNOWN, 0),
        }
    }
}

#[derive(Debug)]
pub enum Addressing {
    Accumulator,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Immediate,
    Implied,
    IndexedIndirect,
    Indirect,
    IndirectIndexed,
    Zeropage,
    ZeropageX,
    ZeropageY,

    UNKNOWN,
}

