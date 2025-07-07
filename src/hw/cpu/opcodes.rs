use std::collections::HashMap;
use crate::hw::cpu::AddressingMode;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    /* ----- Transfer instructions ----- */
    // LDA - load value into accumulator
    LDA,
    // LDX - load value into register X
    LDX,
    // LDX - load value into register Y
    LDY,
    // STA - copy value from register A into memory
    STA,
    // STX - copy value from register X into memory
    STX,
    // STY - copy value from register Y into memory
    STY,
    // TAX - transfer accumulator to X
    TAX,
    // TAY - transfer accumulator to Y
    TAY,
    // TSX - transfer stack pointer to X
    TSX,
    // TXA - transfer X to accumulator
    TXA,
    // TXS - transfer X to stack pointer
    TXS,
    // TYA - transfer Y to accumulator
    TYA,

    /* ----- Stack instructions ----- */
    // PHA - push accumulator on stack
    PHA,
    // PHP - push processor status register (with break flag set)
    PHP,
    // PLA - pull accumulator
    PLA,
    // PLP - pull processor status register
    PLP,

    /* ----- Decrements and increments ----- */
    // DEC - decrement (memory)
    DEC,
    // DEX - decrement X
    DEX,
    // DEY - decrement Y
    DEY,
    // INC - increment (memory)
    INC,
    // INX - increment value in X register
    INX,
    // INY - increment value in Y register
    INY,

    /* ----- Logical operations ----- */
    // AND - and with accumulator
    AND,
    // EOR - exclusive or with accumulator
    EOR,
    // inclusive or with accumulator
    ORA,

    /* ----- Shift and rotate instructions ----- */
    // ASL - arithmetic shift left (shifts in a zero bit on the right)
    ASL,
    // ASLA - arithmetic shift left accumulator (shifts in a zero bit on the right)
    ASLA,
    // LSR - logical shift right (shifts in a zero bit on the left)
    LSR,
    // LSRA - logical shift right accumulator (shifts in a zero bit on the left)
    LSRA,
    // ROL - rotate left (shifts in carry bit on the right)
    ROL,
    // ROLA - rotate left accumulator (shifts in carry bit on the right)
    ROLA,
    // ROR - rotate right (shifts in zero bit on the left)
    ROR,
    // RORA - rotate right accumulator (shifts in zero bit on the left)
    RORA,

    // ADC - add memory to accumulator with carry
    ADC,
    // BRK - return from program
    BRK,
}

#[derive(Debug, Clone, Copy)]
pub struct OpCode {
    pub instruction: Instruction,
    pub bytes: u16,
    pub cycles: u8,
    pub addressing_mode: AddressingMode,
}

impl OpCode {
    pub fn new(instruction: Instruction, bytes: u16, cycles: u8, addressing_mode: AddressingMode) -> Self {
        OpCode {
            instruction,
            bytes,
            cycles,
            addressing_mode,
        }
    }
}

lazy_static::lazy_static! {
    pub static ref OPCODES: HashMap<u8, OpCode> = {
        let mut map = HashMap::new();

        // ADC
        map.insert(0x69, OpCode::new(Instruction::ADC, 2, 2, AddressingMode::Immediate));
        map.insert(0x65, OpCode::new(Instruction::ADC, 2, 3, AddressingMode::ZeroPage));
        map.insert(0x75, OpCode::new(Instruction::ADC, 2, 4, AddressingMode::ZeroPageX));
        map.insert(0x6D, OpCode::new(Instruction::ADC, 3, 4, AddressingMode::Absolute));
        map.insert(0x7D, OpCode::new(Instruction::ADC, 3, 4, AddressingMode::AbsoluteX));
        map.insert(0x79, OpCode::new(Instruction::ADC, 3, 4, AddressingMode::AbsoluteY));
        map.insert(0x61, OpCode::new(Instruction::ADC, 2, 6, AddressingMode::IndirectX));
        map.insert(0x71, OpCode::new(Instruction::ADC, 2, 5, AddressingMode::IndirectY));

        // BRK
        map.insert(0x00, OpCode::new(Instruction::BRK, 1, 7, AddressingMode::Implicit));

        // LDA variants
        map.insert(0xA9, OpCode::new(Instruction::LDA, 2, 2, AddressingMode::Immediate));
        map.insert(0xA5, OpCode::new(Instruction::LDA, 2, 3, AddressingMode::ZeroPage));
        map.insert(0xB5, OpCode::new(Instruction::LDA, 2, 4, AddressingMode::ZeroPageX));
        map.insert(0xAD, OpCode::new(Instruction::LDA, 3, 4, AddressingMode::Absolute));
        map.insert(0xBD, OpCode::new(Instruction::LDA, 3, 4, AddressingMode::AbsoluteX));
        map.insert(0xB9, OpCode::new(Instruction::LDA, 3, 4, AddressingMode::AbsoluteY));
        map.insert(0xA1, OpCode::new(Instruction::LDA, 2, 6, AddressingMode::IndirectX));
        map.insert(0xB1, OpCode::new(Instruction::LDA, 2, 5, AddressingMode::IndirectY));

        // LDX variants
        map.insert(0xA2, OpCode::new(Instruction::LDX, 2, 2, AddressingMode::Immediate));
        map.insert(0xA6, OpCode::new(Instruction::LDX, 2, 3, AddressingMode::ZeroPage));
        map.insert(0xB6, OpCode::new(Instruction::LDX, 2, 4, AddressingMode::ZeroPageY));
        map.insert(0xAE, OpCode::new(Instruction::LDX, 3, 4, AddressingMode::Absolute));
        map.insert(0xBE, OpCode::new(Instruction::LDX, 3, 4, AddressingMode::AbsoluteY));

        // LDY variants
        map.insert(0xA0, OpCode::new(Instruction::LDY, 2, 2, AddressingMode::Immediate));
        map.insert(0xA4, OpCode::new(Instruction::LDY, 2, 3, AddressingMode::ZeroPage));
        map.insert(0xB4, OpCode::new(Instruction::LDY, 2, 4, AddressingMode::ZeroPageX));
        map.insert(0xAC, OpCode::new(Instruction::LDY, 3, 4, AddressingMode::Absolute));
        map.insert(0xBC, OpCode::new(Instruction::LDY, 3, 4, AddressingMode::AbsoluteX));

        // STA variants
        map.insert(0x85, OpCode::new(Instruction::STA, 2, 3, AddressingMode::ZeroPage));
        map.insert(0x95, OpCode::new(Instruction::STA, 2, 4, AddressingMode::ZeroPageX));
        map.insert(0x8D, OpCode::new(Instruction::STA, 3, 4, AddressingMode::Absolute));
        map.insert(0x9D, OpCode::new(Instruction::STA, 3, 5, AddressingMode::AbsoluteX));
        map.insert(0x99, OpCode::new(Instruction::STA, 3, 5, AddressingMode::AbsoluteY));
        map.insert(0x81, OpCode::new(Instruction::STA, 2, 6, AddressingMode::IndirectX));
        map.insert(0x91, OpCode::new(Instruction::STA, 2, 6, AddressingMode::IndirectY));

        // STX variants
        map.insert(0x86, OpCode::new(Instruction::STX, 2, 3, AddressingMode::ZeroPage));
        map.insert(0x96, OpCode::new(Instruction::STX, 2, 4, AddressingMode::ZeroPageY));
        map.insert(0x8E, OpCode::new(Instruction::STX, 3, 4, AddressingMode::Absolute));

        // STY variants
        map.insert(0x84, OpCode::new(Instruction::STY, 2, 3, AddressingMode::ZeroPage));
        map.insert(0x94, OpCode::new(Instruction::STY, 2, 4, AddressingMode::ZeroPageX));
        map.insert(0x8C, OpCode::new(Instruction::STY, 3, 4, AddressingMode::Absolute));

        // TAX
        map.insert(0xAA, OpCode::new(Instruction::TAX, 1, 2, AddressingMode::Implicit));

        // TAY
        map.insert(0xA8, OpCode::new(Instruction::TAY, 1, 2, AddressingMode::Implicit));

        // TSX
        map.insert(0xBA, OpCode::new(Instruction::TSX, 1, 2, AddressingMode::Implicit));

        // TXA
        map.insert(0x8A, OpCode::new(Instruction::TXA, 1, 2, AddressingMode::Implicit));

        // TXS
        map.insert(0x9A, OpCode::new(Instruction::TXS, 1, 2, AddressingMode::Implicit));

        // TYA
        map.insert(0x98, OpCode::new(Instruction::TYA, 1, 2, AddressingMode::Implicit));

        // PHA
        map.insert(0x48, OpCode::new(Instruction::PHA, 1, 3, AddressingMode::Implicit));

        // PHP
        map.insert(0x08, OpCode::new(Instruction::PHP, 1, 3, AddressingMode::Implicit));

        // PLA
        map.insert(0x68, OpCode::new(Instruction::PLA, 1, 4, AddressingMode::Implicit));

        // PLP
        map.insert(0x28, OpCode::new(Instruction::PLP, 1, 4, AddressingMode::Implicit));

        // DEC
        map.insert(0xC6, OpCode::new(Instruction::DEC, 2, 5, AddressingMode::ZeroPage));
        map.insert(0xD6, OpCode::new(Instruction::DEC, 2, 6, AddressingMode::ZeroPageX));
        map.insert(0xCE, OpCode::new(Instruction::DEC, 3, 6, AddressingMode::Absolute));
        map.insert(0xDE, OpCode::new(Instruction::DEC, 3, 7, AddressingMode::AbsoluteX));

        // DEX
        map.insert(0xCA, OpCode::new(Instruction::DEX, 1, 2, AddressingMode::Implicit));

        // DEY
        map.insert(0x88, OpCode::new(Instruction::DEY, 1, 2, AddressingMode::Implicit));

        // INC
        map.insert(0xE6, OpCode::new(Instruction::INC, 2, 5, AddressingMode::ZeroPage));
        map.insert(0xF6, OpCode::new(Instruction::INC, 2, 6, AddressingMode::ZeroPageX));
        map.insert(0xEE, OpCode::new(Instruction::INC, 3, 6, AddressingMode::Absolute));
        map.insert(0xFE, OpCode::new(Instruction::INC, 3, 7, AddressingMode::AbsoluteX));

        // INX
        map.insert(0xE8, OpCode::new(Instruction::INX, 1, 2, AddressingMode::Implicit));

        // INY
        map.insert(0xC8, OpCode::new(Instruction::INY, 1, 2, AddressingMode::Implicit));

        // AND
        map.insert(0x29, OpCode::new(Instruction::AND, 2, 2, AddressingMode::Immediate));
        map.insert(0x25, OpCode::new(Instruction::AND, 2, 3, AddressingMode::ZeroPage));
        map.insert(0x35, OpCode::new(Instruction::AND, 2, 4, AddressingMode::ZeroPageX));
        map.insert(0x2D, OpCode::new(Instruction::AND, 3, 4, AddressingMode::Absolute));
        map.insert(0x3D, OpCode::new(Instruction::AND, 3, 4, AddressingMode::AbsoluteX));
        map.insert(0x39, OpCode::new(Instruction::AND, 3, 4, AddressingMode::AbsoluteY));
        map.insert(0x21, OpCode::new(Instruction::AND, 2, 6, AddressingMode::IndirectX));
        map.insert(0x31, OpCode::new(Instruction::AND, 2, 5, AddressingMode::IndirectY));

        // EOR
        map.insert(0x49, OpCode::new(Instruction::EOR, 2, 2, AddressingMode::Immediate));
        map.insert(0x45, OpCode::new(Instruction::EOR, 2, 3, AddressingMode::ZeroPage));
        map.insert(0x55, OpCode::new(Instruction::EOR, 2, 4, AddressingMode::ZeroPageX));
        map.insert(0x4D, OpCode::new(Instruction::EOR, 3, 4, AddressingMode::Absolute));
        map.insert(0x5D, OpCode::new(Instruction::EOR, 3, 4, AddressingMode::AbsoluteX));
        map.insert(0x59, OpCode::new(Instruction::EOR, 3, 4, AddressingMode::AbsoluteY));
        map.insert(0x41, OpCode::new(Instruction::EOR, 2, 6, AddressingMode::IndirectX));
        map.insert(0x51, OpCode::new(Instruction::EOR, 2, 5, AddressingMode::IndirectY));

        // ORA
        map.insert(0x09, OpCode::new(Instruction::ORA, 2, 2, AddressingMode::Immediate));
        map.insert(0x05, OpCode::new(Instruction::ORA, 2, 3, AddressingMode::ZeroPage));
        map.insert(0x15, OpCode::new(Instruction::ORA, 2, 4, AddressingMode::ZeroPageX));
        map.insert(0x0D, OpCode::new(Instruction::ORA, 3, 4, AddressingMode::Absolute));
        map.insert(0x1D, OpCode::new(Instruction::ORA, 3, 4, AddressingMode::AbsoluteX));
        map.insert(0x19, OpCode::new(Instruction::ORA, 3, 4, AddressingMode::AbsoluteY));
        map.insert(0x01, OpCode::new(Instruction::ORA, 2, 6, AddressingMode::IndirectX));
        map.insert(0x11, OpCode::new(Instruction::ORA, 2, 5, AddressingMode::IndirectY));

        // ASLA
        map.insert(0x0A, OpCode::new(Instruction::ASLA, 1, 2, AddressingMode::Implicit));

        // ASL
        map.insert(0x06, OpCode::new(Instruction::ASL, 2, 5, AddressingMode::ZeroPage));
        map.insert(0x16, OpCode::new(Instruction::ASL, 2, 6, AddressingMode::ZeroPageX));
        map.insert(0x0E, OpCode::new(Instruction::ASL, 3, 6, AddressingMode::Absolute));
        map.insert(0x1E, OpCode::new(Instruction::ASL, 3, 7, AddressingMode::AbsoluteX));

        // LSRA
        map.insert(0x4A, OpCode::new(Instruction::LSRA, 1, 2, AddressingMode::Implicit));

        // LSR
        map.insert(0x46, OpCode::new(Instruction::LSR, 2, 5, AddressingMode::ZeroPage));
        map.insert(0x56, OpCode::new(Instruction::LSR, 2, 6, AddressingMode::ZeroPageX));
        map.insert(0x4E, OpCode::new(Instruction::LSR, 3, 6, AddressingMode::Absolute));
        map.insert(0x5E, OpCode::new(Instruction::LSR, 3, 7, AddressingMode::AbsoluteX));

        // ROLA
        map.insert(0x2A, OpCode::new(Instruction::ROLA, 1, 2, AddressingMode::Implicit));

        // ROL
        map.insert(0x26, OpCode::new(Instruction::ROL, 2, 5, AddressingMode::ZeroPage));
        map.insert(0x36, OpCode::new(Instruction::ROL, 2, 6, AddressingMode::ZeroPageX));
        map.insert(0x2E, OpCode::new(Instruction::ROL, 3, 6, AddressingMode::Absolute));
        map.insert(0x3E, OpCode::new(Instruction::ROL, 3, 7, AddressingMode::AbsoluteX));

        // RORA
        map.insert(0x6A, OpCode::new(Instruction::RORA, 1, 2, AddressingMode::Implicit));

        // ROL
        map.insert(0x66, OpCode::new(Instruction::ROR, 2, 5, AddressingMode::ZeroPage));
        map.insert(0x76, OpCode::new(Instruction::ROR, 2, 6, AddressingMode::ZeroPageX));
        map.insert(0x6E, OpCode::new(Instruction::ROR, 3, 6, AddressingMode::Absolute));
        map.insert(0x7E, OpCode::new(Instruction::ROR, 3, 7, AddressingMode::AbsoluteX));

        map
    };
}
