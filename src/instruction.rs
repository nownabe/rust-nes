use std::convert::From;
use std::fmt::{self, Formatter, Display};

/*
 * https://www.masswerk.at/6502/6502_instruction_set.html
 * http://obelisk.me.uk/6502/reference.html
 * http://www.oxyron.de/html/opcodes02.html - Unofficial opcodes
 */

#[derive(Debug)]
pub struct Instruction(pub Opcode, pub Addressing, pub usize);

#[derive(Debug)]
pub enum Opcode {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,

    // Unofficial
    SLO,

    UNKNOWN(u8),
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::UNKNOWN(byte) => write!(f, "UNKNWON(0x{:02X})", byte),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl From<u8> for Instruction {
    fn from(opcode: u8) -> Self {
        match opcode {
            0x06 => Instruction(Opcode::ASL, Addressing::ZeroPage, 5),

            0xD0 => Instruction(Opcode::BNE, Addressing::Relative, 2),

            0x00 => Instruction(Opcode::BRK, Addressing::Implied, 7),

            0xC6 => Instruction(Opcode::DEC, Addressing::ZeroPage, 5),
            0xD6 => Instruction(Opcode::DEC, Addressing::ZeroPageX, 6),

            0x88 => Instruction(Opcode::DEY, Addressing::Implied, 2),
            0xE8 => Instruction(Opcode::INX, Addressing::Implied, 2),

            0x4C => Instruction(Opcode::JMP, Addressing::Absolute, 3),
            0x20 => Instruction(Opcode::JSR, Addressing::Absolute, 6),

            0xA9 => Instruction(Opcode::LDA, Addressing::Immediate, 2),
            0xA5 => Instruction(Opcode::LDA, Addressing::ZeroPage, 3),
            0xB5 => Instruction(Opcode::LDA, Addressing::ZeroPageX, 4),
            0xAD => Instruction(Opcode::LDA, Addressing::Absolute, 4),
            0xBD => Instruction(Opcode::LDA, Addressing::AbsoluteX, 4),
            0xB9 => Instruction(Opcode::LDA, Addressing::AbsoluteY, 4),
            0xA1 => Instruction(Opcode::LDA, Addressing::IndexedIndirect, 6),
            0xB1 => Instruction(Opcode::LDA, Addressing::IndirectIndexed, 4),

            0xA2 => Instruction(Opcode::LDX, Addressing::Immediate, 2),
            0xA0 => Instruction(Opcode::LDY, Addressing::Immediate, 2),
            0x78 => Instruction(Opcode::SEI, Addressing::Implied, 2),
            0x8D => Instruction(Opcode::STA, Addressing::Absolute, 4),
            0x9A => Instruction(Opcode::TXS, Addressing::Implied, 2),

            // Unofficial instructions
            0x03 => Instruction(Opcode::SLO, Addressing::IndexedIndirect, 8),

            0x02 => Instruction(Opcode::NOP, Addressing::Implied, 2),
            0x80 => Instruction(Opcode::NOP, Addressing::Immediate, 2),

            _ => Instruction(Opcode::UNKNOWN(opcode), Addressing::UNKNOWN, 0),
        }
    }
}

#[derive(Debug,PartialEq)]
pub enum Addressing {
    Accumulator,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Immediate,
    Implied,
    IndexedIndirect, // OPC ($LL, X)
    Indirect,
    IndirectIndexed, // OPC ($LL), Y
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,

    UNKNOWN,
}

