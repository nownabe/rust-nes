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
    ISC,
    KIL,
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
            0x0A => Instruction(Opcode::ASL, Addressing::Accumulator, 2),
            0x06 => Instruction(Opcode::ASL, Addressing::ZeroPage, 5),
            0x16 => Instruction(Opcode::ASL, Addressing::ZeroPageX, 6),
            0x0E => Instruction(Opcode::ASL, Addressing::Absolute, 6),
            0x1E => Instruction(Opcode::ASL, Addressing::AbsoluteX, 7),

            0x30 => Instruction(Opcode::BMI, Addressing::Relative, 2),
            0xD0 => Instruction(Opcode::BNE, Addressing::Relative, 2),

            0x10 => Instruction(Opcode::BPL, Addressing::Relative, 2),

            0x00 => Instruction(Opcode::BRK, Addressing::Implied, 7),

            0x50 => Instruction(Opcode::BVC, Addressing::Relative, 2),

            // Clear flags
            0x18 => Instruction(Opcode::CLC, Addressing::Implied, 2),
            0xD8 => Instruction(Opcode::CLD, Addressing::Implied, 2),
            0x58 => Instruction(Opcode::CLI, Addressing::Implied, 2),
            0xB8 => Instruction(Opcode::CLV, Addressing::Implied, 2),

            // Compare
            0xC9 => Instruction(Opcode::CMP, Addressing::Immediate, 2),
            0xC5 => Instruction(Opcode::CMP, Addressing::ZeroPage, 3),
            0xD1 => Instruction(Opcode::CMP, Addressing::IndirectIndexed, 5),

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
            0xFF => Instruction(Opcode::ISC, Addressing::AbsoluteX, 7),

            0x02 => Instruction(Opcode::KIL, Addressing::Implied, 2),
            0x12 => Instruction(Opcode::KIL, Addressing::Implied, 2),
            0x22 => Instruction(Opcode::KIL, Addressing::Implied, 2),
            0x32 => Instruction(Opcode::KIL, Addressing::Implied, 2),
            0x42 => Instruction(Opcode::KIL, Addressing::Implied, 2),
            0x52 => Instruction(Opcode::KIL, Addressing::Implied, 2),
            0x62 => Instruction(Opcode::KIL, Addressing::Implied, 2),
            0x72 => Instruction(Opcode::KIL, Addressing::Implied, 2),
            0x92 => Instruction(Opcode::KIL, Addressing::Implied, 2),
            0xB2 => Instruction(Opcode::KIL, Addressing::Implied, 2),
            0xD2 => Instruction(Opcode::KIL, Addressing::Implied, 2),
            0xF2 => Instruction(Opcode::KIL, Addressing::Implied, 2),

            0x03 => Instruction(Opcode::SLO, Addressing::IndexedIndirect, 8),

            0x04 => Instruction(Opcode::NOP, Addressing::ZeroPage, 3),
            0x0C => Instruction(Opcode::NOP, Addressing::Absolute, 4),
            0x14 => Instruction(Opcode::NOP, Addressing::ZeroPageX, 4),
            0x1A => Instruction(Opcode::NOP, Addressing::Implied, 2),
            0x1C => Instruction(Opcode::NOP, Addressing::AbsoluteX, 4), //*
            0x34 => Instruction(Opcode::NOP, Addressing::ZeroPageX, 4),
            0x3A => Instruction(Opcode::NOP, Addressing::Implied, 2),
            0x3C => Instruction(Opcode::NOP, Addressing::AbsoluteX, 4), //*
            0x44 => Instruction(Opcode::NOP, Addressing::ZeroPage, 3),
            0x54 => Instruction(Opcode::NOP, Addressing::ZeroPageX, 4),
            0x5A => Instruction(Opcode::NOP, Addressing::Implied, 2),
            0x5C => Instruction(Opcode::NOP, Addressing::AbsoluteX, 4), //*
            0x64 => Instruction(Opcode::NOP, Addressing::ZeroPage, 3),
            0x74 => Instruction(Opcode::NOP, Addressing::ZeroPageX, 4),
            0x7A => Instruction(Opcode::NOP, Addressing::Implied, 2),
            0x7C => Instruction(Opcode::NOP, Addressing::AbsoluteX, 4), //*
            0x80 => Instruction(Opcode::NOP, Addressing::Immediate, 2),
            0x82 => Instruction(Opcode::NOP, Addressing::Immediate, 2),
            0x89 => Instruction(Opcode::NOP, Addressing::Immediate, 2),
            0xC2 => Instruction(Opcode::NOP, Addressing::Immediate, 2),
            0xD4 => Instruction(Opcode::NOP, Addressing::ZeroPageX, 4),
            0xDA => Instruction(Opcode::NOP, Addressing::Implied, 2),
            0xDC => Instruction(Opcode::NOP, Addressing::AbsoluteX, 4), //*
            0xE2 => Instruction(Opcode::NOP, Addressing::Immediate, 2),
            0xF4 => Instruction(Opcode::NOP, Addressing::ZeroPageX, 4),
            0xFA => Instruction(Opcode::NOP, Addressing::Implied, 2),
            0xFC => Instruction(Opcode::NOP, Addressing::AbsoluteX, 4), //*

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
    IndexedIndirect, // OPC ($LL, X); (Indirect, X)
    Indirect,
    IndirectIndexed, // OPC ($LL), Y; (Indirect), Y
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,

    UNKNOWN,
}

