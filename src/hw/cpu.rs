mod opcodes;
mod tests;

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

    #[derive(Clone)]
    pub struct CpuFlags: u8 {
        const CARRY         = 0b00000001;
        const ZERO          = 0b00000010;
        const INTERRUPT     = 0b00000100;
        const DECIMAL       = 0b00001000;
        const BREAK         = 0b00010000;
        const BIT5          = 0b00100000;
        const OVERFLOW      = 0b01000000;
        const NEGATIVE      = 0b10000000;
    }
}

const STACK_PAGE: u16 = 0x0100;
const STACK_START: u8 = 0xff;

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

    fn stack_push(&mut self, data: u8) {
        self.mem_write(STACK_PAGE + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read(STACK_PAGE + self.stack_pointer as u16)
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

    /* ------------ OPCODE IMPLEMENTATIONS ------------ */

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

    fn pha(&mut self, _: AddressingMode) { self.stack_push(self.register_a); }

    fn php(&mut self, _: AddressingMode) {
        let mut flags = self.status.clone();
        flags.insert(CpuFlags::BREAK);
        flags.insert(CpuFlags::BIT5);
        self.stack_push(flags.bits());
    }

    fn pla(&mut self, _: AddressingMode) {
        self.register_a = self.stack_pop();
        self.update_z_and_n_flags(self.register_a)
    }

    fn plp(&mut self, _: AddressingMode) {
        self.status = CpuFlags::from_bits(self.stack_pop()).unwrap_or_else(|| panic!("invalid status register"));
        self.status.remove(CpuFlags::BIT5);
        self.status.remove(CpuFlags::BREAK);
    }

    fn dec(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);
        value = value.wrapping_sub(1);
        self.mem_write(addr, value);
        self.update_z_and_n_flags(value);
    }

    fn dex(&mut self, _: AddressingMode) {
        self.register_x = self.register_x.wrapping_sub(1);
        self.update_z_and_n_flags(self.register_x);
    }

    fn dey(&mut self, _: AddressingMode) {
        self.register_y = self.register_y.wrapping_sub(1);
        self.update_z_and_n_flags(self.register_y);
    }

    fn inc(&mut self, mode: AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mut value = self.mem_read(addr);
        value = value.wrapping_add(1);
        self.mem_write(addr, value);
        self.update_z_and_n_flags(value);
    }

    fn inx(&mut self, _: AddressingMode) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_z_and_n_flags(self.register_x);
    }

    fn iny(&mut self, _: AddressingMode) {
        self.register_y = self.register_y.wrapping_add(1);
        self.update_z_and_n_flags(self.register_y);
    }

    fn and(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        self.register_a &= value;
        self.update_z_and_n_flags(self.register_a);
    }

    fn eor(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        self.register_a ^= value;
        self.update_z_and_n_flags(self.register_a);
    }

    fn ora(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        self.register_a |= value;
        self.update_z_and_n_flags(self.register_a);
    }

    fn asl(&mut self, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        let mut value = self.get_operand_value(mode);
        if value & 0b10000000 == 0b10000000 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        value <<= 1;
        self.mem_write(address, value);
        self.update_z_and_n_flags(value);
    }

    fn asla(&mut self, _: AddressingMode) {
        if self.register_a & 0b10000000 == 0b10000000 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        self.register_a <<= 1;
        self.update_z_and_n_flags(self.register_a);
    }

    fn lsr(&mut self, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        let mut value = self.get_operand_value(mode);
        if value & 0b00000001 == 0b00000001 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        value >>= 1;
        self.mem_write(address, value);
        self.update_z_and_n_flags(value);
    }

    fn lsra(&mut self, _: AddressingMode) {
        if self.register_a & 0b00000001 == 0b00000001 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        self.register_a >>= 1;
        self.update_z_and_n_flags(self.register_a);
    }

    fn rol(&mut self, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        let mut value = self.get_operand_value(mode);
        let old_carry = self.status.contains(CpuFlags::CARRY);
        if value & 0b10000000 == 0b10000000 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        value <<= 1;
        if old_carry {
            value |= 1;
        }

        self.mem_write(address, value);
        self.update_z_and_n_flags(value);
    }

    fn rola(&mut self, _: AddressingMode) {
        let old_carry = self.status.contains(CpuFlags::CARRY);
        if self.register_a & 0b10000000 == 0b10000000 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        self.register_a <<= 1;
        if old_carry {
            self.register_a |= 1;
        }

        self.update_z_and_n_flags(self.register_a);
    }


    fn ror(&mut self, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        let mut value = self.get_operand_value(mode);
        let old_carry = self.status.contains(CpuFlags::CARRY);
        if value & 0b00000001 == 0b00000001 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        value >>= 1;
        if old_carry {
            value |= 0b10000000;
        }

        self.mem_write(address, value);
        self.update_z_and_n_flags(value);
    }

    fn rora(&mut self, _: AddressingMode) {
        let old_carry = self.status.contains(CpuFlags::CARRY);
        if self.register_a & 0b00000001 == 0b00000001 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        self.register_a >>= 1;
        if old_carry {
            self.register_a |= 0b10000000;
        }

        self.update_z_and_n_flags(self.register_a);
    }

    fn clc(&mut self, _: AddressingMode) {
        self.status.remove(CpuFlags::CARRY);
    }

    fn cld(&mut self, _: AddressingMode) {
        self.status.remove(CpuFlags::DECIMAL);
    }

    fn cli(&mut self, _: AddressingMode) {
        self.status.remove(CpuFlags::INTERRUPT);
    }

    fn clv(&mut self, _: AddressingMode) {
        self.status.remove(CpuFlags::OVERFLOW);
    }

    fn sec(&mut self, _: AddressingMode) {
        self.status.insert(CpuFlags::CARRY);
    }

    fn sed(&mut self, _: AddressingMode) {
        self.status.insert(CpuFlags::DECIMAL);
    }

    fn sei(&mut self, _: AddressingMode) {
        self.status.insert(CpuFlags::INTERRUPT);
    }

    fn cmp(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);

        if self.register_a < value {
            self.status.remove(CpuFlags::CARRY);
        } else {
            self.status.insert(CpuFlags::CARRY);
        }

        self.update_z_and_n_flags(self.register_a.wrapping_sub(value));
    }

    fn cpx(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);

        if self.register_x < value {
            self.status.remove(CpuFlags::CARRY);
        } else {
            self.status.insert(CpuFlags::CARRY);
        }

        self.update_z_and_n_flags(self.register_x.wrapping_sub(value));
    }

    fn cpy(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);

        if self.register_y < value {
            self.status.remove(CpuFlags::CARRY);
        } else {
            self.status.insert(CpuFlags::CARRY);
        }

        self.update_z_and_n_flags(self.register_y.wrapping_sub(value));
    }

    fn adc(&mut self, mode: AddressingMode) {
        todo!("")
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
        self.stack_pointer = STACK_START;

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
                    Instruction::STA => {
                        self.sta(opcode.addressing_mode);
                    }
                    Instruction::STX => {
                        self.stx(opcode.addressing_mode);
                    }
                    Instruction::STY => {
                        self.sty(opcode.addressing_mode);
                    }
                    Instruction::PHA => {
                        self.pha(opcode.addressing_mode);
                    }
                    Instruction::PHP => {
                        self.php(opcode.addressing_mode);
                    }
                    Instruction::PLA => {
                        self.pla(opcode.addressing_mode);
                    }
                    Instruction::PLP => {
                        self.plp(opcode.addressing_mode);
                    }
                    Instruction::DEC => {
                        self.dec(opcode.addressing_mode);
                    }
                    Instruction::DEX => {
                        self.dex(opcode.addressing_mode);
                    }
                    Instruction::DEY => {
                        self.dey(opcode.addressing_mode);
                    }
                    Instruction::INC => {
                        self.inc(opcode.addressing_mode);
                    }
                    Instruction::INX => {
                        self.inx(opcode.addressing_mode);
                    }
                    Instruction::INY => {
                        self.iny(opcode.addressing_mode);
                    }
                    Instruction::AND => {
                        self.and(opcode.addressing_mode);
                    }
                    Instruction::EOR => {
                        self.eor(opcode.addressing_mode);
                    }
                    Instruction::ORA => {
                        self.ora(opcode.addressing_mode);
                    }
                    Instruction::ASL => {
                        self.asl(opcode.addressing_mode);
                    }
                    Instruction::ASLA => {
                        self.asla(opcode.addressing_mode);
                    }
                    Instruction::LSR => {
                        self.lsr(opcode.addressing_mode);
                    }
                    Instruction::LSRA => {
                        self.lsra(opcode.addressing_mode);
                    }
                    Instruction::ROL => {
                        self.rol(opcode.addressing_mode);
                    }
                    Instruction::ROLA => {
                        self.rola(opcode.addressing_mode);
                    }
                    Instruction::ROR => {
                        self.ror(opcode.addressing_mode);
                    }
                    Instruction::RORA => {
                        self.rora(opcode.addressing_mode);
                    }
                    Instruction::CLC => {
                        self.clc(opcode.addressing_mode);
                    }
                    Instruction::CLD => {
                        self.cld(opcode.addressing_mode);
                    }
                    Instruction::CLI => {
                        self.cli(opcode.addressing_mode);
                    }
                    Instruction::CLV => {
                        self.clv(opcode.addressing_mode);
                    }
                    Instruction::SEC => {
                        self.sec(opcode.addressing_mode);
                    }
                    Instruction::SED => {
                        self.sed(opcode.addressing_mode);
                    }
                    Instruction::SEI => {
                        self.sei(opcode.addressing_mode);
                    }
                    Instruction::CMP => {
                        self.cmp(opcode.addressing_mode);
                    }
                    Instruction::CPX => {
                        self.cpx(opcode.addressing_mode);
                    }
                    Instruction::CPY => {
                        self.cpy(opcode.addressing_mode);
                    }
                }

                self.program_counter += opcode.bytes - 1;
            } else {
                panic!("Illegal instruction: 0x{:02X}", opcode_byte);
            }
        }
    }
}