mod opcodes;
mod tests;
pub mod tracer;
mod interrupt;

use std::cmp::PartialEq;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use crate::hw::bus::Bus;
use crate::hw::cpu::opcodes::{Instruction, OPCODES};
use crate::hw::memory::Memory;

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

    #[derive(Clone, Serialize, Deserialize)]
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
const STACK_START: u8 = 0xfd;

#[derive(Serialize, Deserialize)]
pub struct CPU<'a> {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub status: CpuFlags,
    pub stack_pointer: u8,
    pub program_counter: u16,
    pub bus: Bus<'a>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AddressingMode {
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Implicit,
    Relative,
}

impl<'a> Memory for CPU<'a> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        self.bus.mem_read(addr)
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.bus.mem_write(addr, data);
    }

    fn mem_read_u16(&mut self, addr: u16) -> u16 {
        self.bus.mem_read_u16(addr)
    }

    fn mem_write_u16(&mut self, addr: u16, data: u16) {
        self.bus.mem_write_u16(addr, data);
    }
}

impl<'a> CPU<'a> {
    pub fn new(bus: Bus) -> CPU {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            status: CpuFlags::from_bits_truncate(0b100100),
            stack_pointer: STACK_START,
            program_counter: 0,
            bus,
        }
    }

    fn stack_push(&mut self, data: u8) {
        self.mem_write(STACK_PAGE + self.stack_pointer as u16, data);
        self.stack_pointer = self.stack_pointer.wrapping_sub(1);
    }

    fn stack_pop(&mut self) -> u8 {
        self.stack_pointer = self.stack_pointer.wrapping_add(1);
        self.mem_read(STACK_PAGE + self.stack_pointer as u16)
    }

    fn stack_push_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.stack_push(hi);
        self.stack_push(lo);
    }

    fn stack_pop_u16(&mut self) -> u16 {
        let lo = self.stack_pop() as u16;
        let hi = self.stack_pop() as u16;

        hi << 8 | lo
    }

    fn add_to_register_a(&mut self, value: u8) {
        let sum = value as u16 + self.register_a as u16 + if self.status.contains(CpuFlags::CARRY) { 1 } else { 0 } as u16;

        let carry = sum > 0xFF;
        if carry {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        let result = sum as u8;

        if (value ^ result) & (self.register_a ^ result) & 0b1000_0000 != 0 {
            self.status.insert(CpuFlags::OVERFLOW);
        } else {
            self.status.remove(CpuFlags::OVERFLOW);
        }

        self.register_a = result;
        self.update_z_and_n_flags(self.register_a);
    }

    fn get_operand_value(&mut self, mode: AddressingMode) -> u8 {
        let address = self.get_operand_address(mode);
        self.mem_read(address)
    }

    fn get_operand_address(&mut self, mode: AddressingMode) -> u16 {
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

            AddressingMode::Indirect => {
                let ptr = self.mem_read(self.program_counter);

                let lo = self.mem_read(ptr as u16);
                let hi = self.mem_read(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
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
                let hi = self.mem_read(base.wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.register_y as u16);
                deref
            }

            AddressingMode::Implicit | AddressingMode::Relative => {
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
        self.status.insert(CpuFlags::BIT5);
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

    fn adc(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        self.add_to_register_a(value);
    }

    fn sbc(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        self.add_to_register_a(value.wrapping_neg().wrapping_sub(1));
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

    fn branch(&mut self, condition: bool) {
        if condition {
            let offset: i8 = self.mem_read(self.program_counter) as i8;
            let jump_addr = self.program_counter.wrapping_add(1).wrapping_add(offset as u16);
            self.program_counter = jump_addr;
        }
    }
    fn bcc(&mut self, _: AddressingMode) {
        self.branch(!self.status.contains(CpuFlags::CARRY));
    }

    fn bcs(&mut self, _: AddressingMode) {
        self.branch(self.status.contains(CpuFlags::CARRY));
    }

    fn beq(&mut self, _: AddressingMode) {
        self.branch(self.status.contains(CpuFlags::ZERO));
    }

    fn bmi(&mut self, _: AddressingMode) {
        self.branch(self.status.contains(CpuFlags::NEGATIVE));
    }

    fn bne(&mut self, _: AddressingMode) {
        self.branch(!self.status.contains(CpuFlags::ZERO));
    }

    fn bpl(&mut self, _: AddressingMode) {
        self.branch(!self.status.contains(CpuFlags::NEGATIVE));
    }

    fn bvc(&mut self, _: AddressingMode) {
        self.branch(!self.status.contains(CpuFlags::OVERFLOW));
    }

    fn bvs(&mut self, _: AddressingMode) {
        self.branch(self.status.contains(CpuFlags::OVERFLOW));
    }

    fn jmp(&mut self, mode: AddressingMode) {
        if mode == AddressingMode::Absolute {
            let jump = self.mem_read_u16(self.program_counter);
            self.program_counter = jump;
        } else if mode == AddressingMode::Indirect {
            let mem_address = self.mem_read_u16(self.program_counter);

            // let indirect_ref = self.mem_read_u16(mem_address);
            // 6502 bug with page boundary (http://www.6502.org/tutorials/6502opcodes.html#JMP)
            let indirect_ref = if mem_address & 0x00FF == 0x00FF {
                let lo = self.mem_read(mem_address);
                let hi = self.mem_read(mem_address & 0xFF00);
                (hi as u16) << 8 | (lo as u16)
            } else {
                self.mem_read_u16(mem_address)
            };

            self.program_counter = indirect_ref;
        }
    }

    fn jsr(&mut self, mode: AddressingMode) {
        self.stack_push_u16(self.program_counter + 2 - 1);
        let address = self.get_operand_address(mode);
        self.program_counter = address
    }

    fn rts(&mut self, _: AddressingMode) {
        self.program_counter = self.stack_pop_u16() + 1;
    }

    fn rti(&mut self, _: AddressingMode) {
        self.status = CpuFlags::from_bits(self.stack_pop()).unwrap_or_else(|| panic!("invalid status register"));
        self.status.insert(CpuFlags::BIT5);
        self.program_counter = self.stack_pop_u16();
    }

    fn bit(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        let negative = (value & 0b1000_0000) == 0b1000_0000;
        if negative {
            self.status.insert(CpuFlags::NEGATIVE);
        } else {
            self.status.remove(CpuFlags::NEGATIVE);
        }

        let overflow = (value & 0b0100_0000) == 0b0100_0000;
        if overflow {
            self.status.insert(CpuFlags::OVERFLOW);
        } else {
            self.status.remove(CpuFlags::OVERFLOW);
        }

        if self.register_a & value == 0 {
            self.status.insert(CpuFlags::ZERO);
        } else {
            self.status.remove(CpuFlags::ZERO);
        }
    }

    fn lax(&mut self, mode: AddressingMode) {
        let param = self.get_operand_value(mode);
        self.register_a = param;
        self.register_x = param;
        self.update_z_and_n_flags(self.register_x);
    }

    fn aax(&mut self, mode: AddressingMode) {
        let address = self.get_operand_address(mode);
        self.mem_write(address, self.register_a & self.register_x);
    }

    fn dcp(&mut self, mode: AddressingMode) {
        self.dec(mode);
        self.cmp(mode);
    }

    fn isc(&mut self, mode: AddressingMode) {
        self.inc(mode);
        self.sbc(mode);
    }

    fn slo(&mut self, mode: AddressingMode) {
        self.asl(mode);
        self.ora(mode);
    }

    fn rla(&mut self, mode: AddressingMode) {
        self.rol(mode);
        self.and(mode);
    }

    fn sre(&mut self, mode: AddressingMode) {
        self.lsr(mode);
        self.eor(mode);
    }

    fn rra(&mut self, mode: AddressingMode) {
        self.ror(mode);
        self.adc(mode);
    }

    fn anc(&mut self, mode: AddressingMode) {
        let value = self.get_operand_value(mode);
        self.register_a = self.register_a & value;
        self.update_z_and_n_flags(self.register_a);
        if self.status.contains(CpuFlags::NEGATIVE) {
            self.status.insert(CpuFlags::CARRY);
        }
    }

    fn arr(&mut self, mode: AddressingMode) {
        self.and(mode);
        self.rora(mode);
        let bit6 = (self.register_a & 0b0100_0000) >> 6;
        let bit5 = (self.register_a & 0b0010_0000) >> 5;
        if bit6 != 0 {
            self.status.insert(CpuFlags::CARRY);
        } else {
            self.status.remove(CpuFlags::CARRY);
        }

        if bit6 ^ bit5 != 0 {
            self.status.insert(CpuFlags::OVERFLOW);
        } else {
            self.status.remove(CpuFlags::OVERFLOW);
        }
    }

    fn asr(&mut self, mode: AddressingMode) {
        self.and(mode);
        self.lsra(mode);
    }

    fn atx(&mut self, mode: AddressingMode) {
        self.and(mode);
        self.txa(mode);
    }
    /* ----------------------------------------- */

    pub fn reset_and_run(&mut self) {
        self.reset();
        self.run();
    }

    pub fn load(&mut self, program: Vec<u8>) {
        for i in 0..(program.len() as u16) {
            self.mem_write(0x0000 + i, program[i as usize]);
        }
        self.mem_write_u16(0xFFFC, 0x0000);
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.register_y = 0;
        self.status = CpuFlags::from_bits_truncate(0b100100);
        self.stack_pointer = STACK_START;

        self.program_counter = self.mem_read_u16(0xFFFC);
    }

    fn interrupt(&mut self, interrupt: interrupt::Interrupt) {
        self.stack_push_u16(self.program_counter);
        let mut flag = self.status.clone();
        flag.set(CpuFlags::BREAK, interrupt.b_flag_mask & 0b010000 == 1);
        flag.set(CpuFlags::BIT5, interrupt.b_flag_mask & 0b100000 == 1);

        self.stack_push(flag.bits());
        self.status.insert(CpuFlags::INTERRUPT);

        self.bus.tick(interrupt.cpu_cycles);
        self.program_counter = self.mem_read_u16(interrupt.vector_addr);
    }

    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            self.step(&mut callback);
        }
    }

    pub fn step<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        if let Some(_nmi) = self.bus.poll_nmi_status() {
            self.interrupt(interrupt::NMI);
        }

        callback(self);
        let opcode_byte = self.mem_read(self.program_counter);
        self.program_counter += 1;
        let old_counter = self.program_counter;

        if let Some(opcode) = OPCODES.get(&opcode_byte) {
            match opcode.instruction {
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
                Instruction::ADC => {
                    self.adc(opcode.addressing_mode);
                }
                Instruction::SBC => {
                    self.sbc(opcode.addressing_mode);
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
                Instruction::BCC => {
                    self.bcc(opcode.addressing_mode);
                }
                Instruction::BCS => {
                    self.bcs(opcode.addressing_mode);
                }
                Instruction::BEQ => {
                    self.beq(opcode.addressing_mode);
                }
                Instruction::BMI => {
                    self.bmi(opcode.addressing_mode);
                }
                Instruction::BNE => {
                    self.bne(opcode.addressing_mode);
                }
                Instruction::BPL => {
                    self.bpl(opcode.addressing_mode);
                }
                Instruction::BVC => {
                    self.bvc(opcode.addressing_mode);
                }
                Instruction::BVS => {
                    self.bvs(opcode.addressing_mode);
                }
                Instruction::JMP => {
                    self.jmp(opcode.addressing_mode);
                }
                Instruction::JSR => {
                    self.jsr(opcode.addressing_mode);
                }
                Instruction::RTS => {
                    self.rts(opcode.addressing_mode);
                }
                Instruction::BRK => {
                    return;
                }
                Instruction::RTI => {
                    self.rti(opcode.addressing_mode);
                }
                Instruction::BIT => {
                    self.bit(opcode.addressing_mode);
                }
                Instruction::NOP | Instruction::DOP | Instruction::TOP => {}
                Instruction::LAX => {
                    self.lax(opcode.addressing_mode);
                }
                Instruction::AAX => {
                    self.aax(opcode.addressing_mode);
                }
                Instruction::DCP => {
                    self.dcp(opcode.addressing_mode);
                }
                Instruction::ISC => {
                    self.isc(opcode.addressing_mode);
                }
                Instruction::SLO => {
                    self.slo(opcode.addressing_mode);
                }
                Instruction::RLA => {
                    self.rla(opcode.addressing_mode);
                }
                Instruction::SRE => {
                    self.sre(opcode.addressing_mode);
                }
                Instruction::RRA => {
                    self.rra(opcode.addressing_mode);
                }
                Instruction::SBCU => {
                    self.sbc(opcode.addressing_mode);
                }
                Instruction::ANC => {
                    self.anc(opcode.addressing_mode);
                }
                Instruction::ARR => {
                    self.arr(opcode.addressing_mode);
                }
                Instruction::ASR => {
                    self.asr(opcode.addressing_mode);
                }
                Instruction::ATX => {
                    self.atx(opcode.addressing_mode);
                }
            }

            self.bus.tick(opcode.cycles);
            if old_counter == self.program_counter {
                self.program_counter += opcode.bytes - 1;
            }
        } else {
            panic!("Illegal instruction: 0x{:02X}", opcode_byte);
        }
    }
}