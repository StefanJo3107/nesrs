mod opcodes;

use std::ops::BitAnd;
use bitflags::bitflags;
use crate::hw::cpu::opcodes::{Instruction, OPCODES};

bitflags! {
    // Status Register Flags (bit 7 to bit 0)
    // N V - B D I Z C
    // 7 6 5 4 3 2 1 0
    //
    // N	Negative
    // V	Overflow
    // -	ignored
    // B	Break
    // D	Decimal (use BCD for arithmetics)
    // I	Interrupt (IRQ disable)
    // Z	Zero
    // C	Carry

    pub struct CpuFlags: u8 {
        const CARRY         = 0b00000001;
        const ZERO          = 0b00000010;
        const INTERRUPT     = 0b00000100;
        const DECIMAL       = 0b00001000;
        const BREAK         = 0b00010000;
        const OVERFLOW      = 0b01000000;
        const NEGATIVE      = 0b10000000;
    }
}

pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: CpuFlags,
    pub stack_pointer: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF],
}

#[derive(Debug, Clone, Copy)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    IndirectX,
    IndirectY,
    Implicit,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: CpuFlags::empty(),
            stack_pointer: 0,
            program_counter: 0,
            memory: [0; 0xFFFF],
        }
    }

    fn mem_read(&self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.memory[addr as usize] = data;
    }

    fn mem_read_u16(&self, addr: u16) -> u16 {
        let lo = self.mem_read(addr) as u16;
        let hi = self.mem_read(addr + 1) as u16;
        (hi << 8) | lo
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xFF) as u8;
        self.mem_write(addr, lo);
        self.mem_write(addr + 1, hi);
    }

    fn get_operand_value(&mut self, mode: AddressingMode) -> u8 {
        let address = self.get_operand_address(mode);
        self.mem_read(address)
    }

    fn get_operand_address(&self, mode: AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,

            AddressingMode::ZeroPage => self.mem_read(self.program_counter) as u16,

            AddressingMode::Absolute => self.mem_read_u16(self.program_counter),

            AddressingMode::ZeroPageX => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.register_x) as u16
            }

            AddressingMode::ZeroPageY => {
                let pos = self.mem_read(self.program_counter);
                pos.wrapping_add(self.register_y) as u16
            }

            AddressingMode::AbsoluteX => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_x as u16)
            }

            AddressingMode::AbsoluteY => {
                let base = self.mem_read_u16(self.program_counter);
                base.wrapping_add(self.register_y as u16)
            }

            AddressingMode::IndirectX => {
                let base = self.mem_read(self.program_counter);

                let ptr: u8 = base.wrapping_add(self.register_x);
                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }

            AddressingMode::IndirectY => {
                let base = self.mem_read(self.program_counter);

                let lo = self.mem_read(base as u16);
                let hi = self.mem_read((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            AddressingMode::Implicit => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    fn update_z_and_n_flags(&mut self, result: u8) {
        if result == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }

        if result & 0b1000_0000 != 0 {
            self.status.insert(CpuFlags::NEGATIVE);
        } else {
            self.status.remove(CpuFlags::NEGATIVE);
        }
    }

    /* ------------ OPCODE EXECUTES ------------ */

    fn adc(&mut self, mode: AddressingMode) {
        todo!("")
    }

    fn lda(&mut self, mode: AddressingMode) {
        let param = self.get_operand_value(mode);
        self.register_a = param;
        self.update_z_and_n_flags(self.register_a);
    }

    fn ldx(&mut self, mode: AddressingMode) {
        let param = self.get_operand_value(mode);
        self.register_x = param;
        self.update_z_and_n_flags(self.register_x);
    }

    fn ldy(&mut self, mode: AddressingMode) {
        let param = self.get_operand_value(mode);
        self.register_y = param;
        self.update_z_and_n_flags(self.register_y);
    }

    fn tax(&mut self, _: AddressingMode) {
        self.register_x = self.register_a;
        self.update_z_and_n_flags(self.register_x);
    }

    fn tay(&mut self, _: AddressingMode) {
        self.register_y = self.register_a;
        self.update_z_and_n_flags(self.register_y);
    }

    fn tsx(&mut self, _: AddressingMode) {
        self.register_x = self.stack_pointer;
        self.update_z_and_n_flags(self.register_x);
    }

    fn txa(&mut self, _: AddressingMode) {
        self.register_a = self.register_x;
        self.update_z_and_n_flags(self.register_a);
    }

    fn txs(&mut self, _: AddressingMode) {
        self.stack_pointer = self.register_x;
        self.update_z_and_n_flags(self.stack_pointer);
    }

    fn tya(&mut self, _: AddressingMode) {
        self.register_a = self.register_y;
        self.update_z_and_n_flags(self.register_a);
    }

    fn inx(&mut self, _: AddressingMode) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_z_and_n_flags(self.register_x);
    }

    fn sta(&mut self, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        self.mem_write(address, self.register_a);
    }

    fn stx(&mut self, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        self.mem_write(address, self.register_x);
    }

    fn sty(&mut self, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        self.mem_write(address, self.register_y);
    }

    /* ----------------------------------------- */

    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000..(0x8000 + program.len())].copy_from_slice(&program);
        self.mem_write_u16(0xFFFC, 0x8000);
    }

    fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = CpuFlags::empty();

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    pub fn run(&mut self) {
        loop {
            let opcode_byte = self.mem_read(self.program_counter);
            self.program_counter += 1;

            if let Some(opcode) = OPCODES.get(&opcode_byte) {
                match opcode.instruction {
                    Instruction::ADC => {
                        self.adc(opcode.addressing_mode);
                    }
                    Instruction::BRK => {
                        return;
                    }
                    Instruction::TAX => {
                        self.tax(opcode.addressing_mode);
                    }
                    Instruction::TAY => {
                        self.tay(opcode.addressing_mode);
                    }
                    Instruction::TSX => {
                        self.tsx(opcode.addressing_mode);
                    }
                    Instruction::TXA => {
                        self.txa(opcode.addressing_mode);
                    }
                    Instruction::TXS => {
                        self.txs(opcode.addressing_mode);
                    }
                    Instruction::TYA => {
                        self.tya(opcode.addressing_mode);
                    }
                    Instruction::LDA => {
                        self.lda(opcode.addressing_mode);
                    }
                    Instruction::LDX => {
                        self.ldx(opcode.addressing_mode);
                    }
                    Instruction::LDY => {
                        self.ldy(opcode.addressing_mode);
                    }
                    Instruction::INX => {
                        self.inx(opcode.addressing_mode);
                    }
                    Instruction::STA => {
                        self.sta(opcode.addressing_mode);
                    }
                    Instruction::STX => {
                        self.stx(opcode.addressing_mode);
                    }
                    Instruction::STY => {
                        self.sty(opcode.addressing_mode);
                    }
                }

                self.program_counter += opcode.bytes - 1;
            } else {
                panic!("Illegal instruction: 0x{:02X}", opcode_byte);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert_eq!(cpu.status.bitand(CpuFlags::ZERO).bits(), 0b10);
    }

    #[test]
    fn test_0xa2_ldx_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x00, 0x00]);
        assert_eq!(cpu.status.bitand(CpuFlags::ZERO).bits(), 0b10);
    }

    #[test]
    fn test_0xa2_ldy_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x00, 0x00]);
        assert_eq!(cpu.status.bitand(CpuFlags::ZERO).bits(), 0b10);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xaa, 0x00]);

        assert_eq!(cpu.register_x, 0)
    }

    #[test]
    fn test_0xa8_tay_move_a_to_y() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa8, 0x00]);

        assert_eq!(cpu.register_y, 0)
    }

    #[test]
    fn test_0xba_tsx_move_sp_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xba, 0x00]);

        assert_eq!(cpu.register_x, 0)
    }

    #[test]
    fn test_0x8a_txa_move_x_to_a() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x8a, 0x00]);

        assert_eq!(cpu.register_a, 0)
    }

    #[test]
    fn test_0x9a_txs_move_x_to_sp() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x9a, 0x00]);

        assert_eq!(cpu.stack_pointer, 0)
    }

    #[test]
    fn test_0x98_tya_move_y_to_a() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x98, 0x00]);

        assert_eq!(cpu.register_a, 0)
    }

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 0x05);
        assert_eq!(cpu.status.bits() & CpuFlags::ZERO.bits(), 0b00);
        assert_eq!(cpu.status.bits() & CpuFlags::NEGATIVE.bits(), 0);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 2)
    }

    #[test]
    fn test_5_ops_working_together() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0xc1)
    }

    #[test]
    fn test_lda_from_memory() {
        let mut cpu = CPU::new();
        cpu.mem_write(0x10, 0x55);

        cpu.load_and_run(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.register_a, 0x55);
    }
}