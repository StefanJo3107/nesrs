use crate::hw::cpu::{AddressingMode, CPU};
use crate::hw::cpu::opcodes::{Instruction, OPCODES};
use crate::hw::memory::Memory;

pub fn trace(cpu: &mut CPU) -> String {
    let mut trace = String::new();
    trace += &format!("{:04X}  ", cpu.program_counter);
    let opcode_byte = cpu.mem_read(cpu.program_counter);
    trace += &format!("{:02X} ", opcode_byte);
    if let Some(opcode) = OPCODES.get(&opcode_byte) {
        for i in 0..opcode.bytes - 1 {
            let operand = cpu.mem_read(cpu.program_counter + i + 1);
            trace += &format!("{:02X} ", operand);
        }
        trace = format!("{:15}", trace);
        if opcode.instruction.to_string().chars().nth(0).unwrap() == '*' {
            trace += &format!("{} ", opcode.instruction.to_string().as_str());
        } else {
            trace += &format!(" {} ", opcode.instruction.to_string().as_str());
        }
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
                let jumps_and_branches = vec![Instruction::JMP, Instruction::JSR, Instruction::RTS,
                                              Instruction::BCC, Instruction::BCS, Instruction::BEQ, Instruction::BMI,
                                              Instruction::BNE, Instruction::BPL, Instruction::BVC, Instruction::BVS];
                if jumps_and_branches.contains(&opcode.instruction) {
                    trace += &format!("${:04X}", address);
                } else {
                    trace += &format!("${:04X} = {:02X}", address, cpu.mem_read(address));
                }
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
                let lo = cpu.mem_read(indirect as u16) as u16;
                let hi = cpu.mem_read(indirect.wrapping_add(1) as u16) as u16;
                let address = (hi << 8) | lo;
                trace += &format!("(${:02X},X) @ {:02X} = {:04X} = {:02X} ", offset, indirect, address, cpu.mem_read(address));
            }
            AddressingMode::IndirectY => {
                let indirect = cpu.mem_read(cpu.program_counter + 1);
                let lo = cpu.mem_read(indirect as u16) as u16;
                let hi = cpu.mem_read(indirect.wrapping_add(1) as u16) as u16;
                let offset = (hi << 8) | lo;
                let address = offset.wrapping_add(cpu.register_y as u16);
                trace += &format!("(${:02X}),Y = {:04X} @ {:04X} = {:02X} ", indirect, offset, address, cpu.mem_read(address));
            }
            AddressingMode::Relative => {
                let offset: i8 = cpu.mem_read(cpu.program_counter + 1) as i8;
                let jump_addr = cpu.program_counter.wrapping_add(2).wrapping_add(offset as u16);
                trace += &format!("${:04X} ", jump_addr);
            }
            AddressingMode::Indirect => {
                let indirect = cpu.mem_read_u16(cpu.program_counter + 1);
                let address = cpu.mem_read_u16(indirect);
                let jumps_and_branches = vec![Instruction::JSR, Instruction::RTS,
                                              Instruction::BCC, Instruction::BCS, Instruction::BEQ, Instruction::BMI,
                                              Instruction::BNE, Instruction::BPL, Instruction::BVC, Instruction::BVS];
                if opcode.instruction == Instruction::JMP {
                    let indirect = cpu.mem_read_u16(cpu.program_counter + 1);

                    // let indirect_ref = self.mem_read_u16(mem_address);
                    // 6502 bug with page boundary (http://www.6502.org/tutorials/6502opcodes.html#JMP)
                    let indirect_ref = if indirect & 0x00FF == 0x00FF {
                        let lo = cpu.mem_read(indirect);
                        let hi = cpu.mem_read(indirect & 0xFF00);
                        (hi as u16) << 8 | (lo as u16)
                    } else {
                        cpu.mem_read_u16(indirect)
                    };

                    trace += &format!("(${:04X}) = {:04X} ", indirect, indirect_ref);
                } else if jumps_and_branches.contains(&opcode.instruction) {
                    trace += &format!("(${:04X}) = {:04X} ", indirect, address);
                } else {
                    trace += &format!("(${:04X}) @ {:04X} = {:02X} ", indirect, address, cpu.mem_read(address));
                }
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
    trace += &format!("A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}", cpu.register_a, cpu.register_x, cpu.register_y, cpu.status, cpu.stack_pointer);
    trace
}