use std::fmt::format;
use log::trace;
use crate::hw::cpu::{AddressingMode, CPU};
use crate::hw::cpu::opcodes::OPCODES;
use crate::hw::memory::Memory;

pub fn trace(cpu: &CPU) {
    let mut trace = String::new();
    trace += &format!("{:04X}  ", cpu.program_counter);
    let opcode_byte = cpu.mem_read(cpu.program_counter);
    trace += &format!("{:02X} ", opcode_byte);
    if let Some(opcode) = OPCODES.get(&opcode_byte) {
        for _ in 0..opcode.bytes - 1 {
            let operand = cpu.mem_read(cpu.program_counter + 1);
            trace += &format!("{:02X} ", operand);
        }
        trace = format!("{:16}", trace);
        trace += &format!("{} ", opcode.instruction.to_string().as_str());
        match opcode.addressing_mode {
            AddressingMode::Immediate => {
                trace += &format!("#${:02X} ", cpu.mem_read(cpu.program_counter + 1));
            }
            AddressingMode::ZeroPage => {
                let address = cpu.mem_read(cpu.program_counter + 1);
                trace += &format!("${:02X} = {:02X} ", address, cpu.mem_read(address as u16));
            }
            AddressingMode::ZeroPageX => {
                let offset = cpu.mem_read(cpu.program_counter + 1);
                let address = cpu.register_x.wrapping_add(offset);
                trace += &format!("${:02X},X @ {:02X} = {:02X} ", offset, address, cpu.mem_read(address as u16));
            }
            AddressingMode::ZeroPageY => {
                let offset = cpu.mem_read(cpu.program_counter + 1);
                let address = cpu.register_y.wrapping_add(offset);
                trace += &format!("${:02X},Y @ {:02X} = {:02X} ", offset, address, cpu.mem_read(address as u16));
            }
            AddressingMode::Absolute => {
                let address = cpu.mem_read_u16(cpu.program_counter + 1);
                trace += &format!("${:04X} = ${:02X} ", address, cpu.mem_read(address));
            }
            AddressingMode::AbsoluteX => {
                let offset = cpu.mem_read_u16(cpu.program_counter + 1);
                let address = offset.wrapping_add(cpu.register_x as u16);
                trace += &format!("${:04X},X @ {:04X} = {:02X} ", offset, address, cpu.mem_read(address));
            }
            AddressingMode::AbsoluteY => {
                let offset = cpu.mem_read_u16(cpu.program_counter + 1);
                let address = offset.wrapping_add(cpu.register_y as u16);
                trace += &format!("${:04X},Y @ {:04X} = {:02X} ", offset, address, cpu.mem_read(address));
            }
            AddressingMode::IndirectX => {
                let offset = cpu.mem_read(cpu.program_counter + 1);
                let indirect = cpu.register_x.wrapping_add(offset);
                let address = cpu.mem_read_u16(indirect as u16);
                trace += &format!("(${:02X},X) @ {:02X} = {:04X} = {:02X} ", offset, indirect, address, cpu.mem_read(address));
            }
            AddressingMode::IndirectY => {
                let indirect = cpu.mem_read(cpu.program_counter + 1);
                let offset = cpu.mem_read_u16(indirect as u16);
                let address = offset.wrapping_add(cpu.register_y as u16);
                trace += &format!("(${:02X}),Y @ {:02X} = {:04X} = {:02X} ", indirect, offset, address, cpu.mem_read(address));
            }
            AddressingMode::Relative => {
                trace += &format!("${:04X} ", cpu.program_counter + cpu.mem_read(cpu.program_counter + 1) as u16);
            }
            AddressingMode::Indirect => {
                let indirect = cpu.mem_read_u16(cpu.program_counter + 1);
                let address = cpu.mem_read_u16(indirect);
                trace += &format!("(${:04X}) @ {:04X} = {:02X} ", indirect, address, cpu.mem_read(address));
            }
            AddressingMode::Implicit => {
                trace += " ";
            }
        }
    } else {
        trace += &format!("{:16}", "");
        trace += "ILLEGAL";
    }

    trace = format!("{:48}", trace);
    trace += &format!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", cpu.register_a, cpu.register_x, cpu.register_y, cpu.status, cpu.program_counter);
    println!("{}", trace);
}