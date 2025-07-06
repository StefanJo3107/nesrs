use crate::hw::cpu::{CpuFlags, CPU};

#[cfg(test)]
mod test {
    use std::ops::BitAnd;
    use crate::hw::cpu::STACK_START;
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

        assert_eq!(cpu.register_x, STACK_START)
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

    #[test]
    fn test_0x48_pha_pushes_accumulator_to_stack() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x48, 0x00]);

        assert_eq!(cpu.mem_read(0x01FF), 0);
        assert_eq!(cpu.stack_pointer, 0xFE);
    }

    #[test]
    fn test_0x48_pha_with_zero_accumulator() {
        let mut cpu = CPU::new();
        cpu.register_a = 0x00;
        cpu.load_and_run(vec![0x48, 0x00]);

        assert_eq!(cpu.mem_read(0x01FF), 0x00);
        assert_eq!(cpu.stack_pointer, 0xFE);
    }

    #[test]
    fn test_0x48_pha_multiple_pushes() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x48, 0xa9, 0x22, 0x48, 0x00]);

        assert_eq!(cpu.mem_read(0x01FF), 0);
        assert_eq!(cpu.mem_read(0x01FE), 0x22);
        assert_eq!(cpu.stack_pointer, 0xFD);
    }

    #[test]
    fn test_0x08_php_pushes_status_to_stack() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x08, 0x00]);

        let pushed_status = cpu.mem_read(0x01FF);
        assert_eq!(pushed_status & CpuFlags::ZERO.bits(), 0);
        assert_eq!(pushed_status & CpuFlags::CARRY.bits(), 0);
        assert_eq!(pushed_status & CpuFlags::BREAK.bits(), CpuFlags::BREAK.bits());
        assert_eq!(cpu.stack_pointer, 0xFE);
    }

    #[test]
    fn test_0x08_php_sets_break_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x08, 0x00]);

        let pushed_status = cpu.mem_read(0x01FF);
        assert_eq!(pushed_status & CpuFlags::BREAK.bits(), CpuFlags::BREAK.bits());
        assert_eq!(pushed_status & CpuFlags::NEGATIVE.bits(), 0);
    }

    #[test]
    fn test_0x08_php_with_empty_status() {
        let mut cpu = CPU::new();
        cpu.status = CpuFlags::empty();
        cpu.load_and_run(vec![0x08, 0x00]);

        let pushed_status = cpu.mem_read(0x01FF);
        assert_eq!(pushed_status & CpuFlags::BREAK.bits(), CpuFlags::BREAK.bits());
        assert_eq!(cpu.stack_pointer, 0xFE);
    }

    #[test]
    fn test_0x68_pla_pulls_from_stack_to_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x48, 0xa9, 0x00, 0x68, 0x00]);

        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }

    #[test]
    fn test_0x68_pla_sets_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x48, 0xa9, 0x42, 0x68, 0x00]);

        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.status.bitand(CpuFlags::ZERO).bits(), CpuFlags::ZERO.bits());
    }

    #[test]
    fn test_0x28_plp_pulls_status_from_stack() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x08, 0x28, 0x00]);

        assert_eq!(cpu.status.clone().bitand(CpuFlags::ZERO).bits(), 0);
        assert_eq!(cpu.status.bitand(CpuFlags::CARRY).bits(), 0);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }

    #[test]
    fn test_pha_pla_round_trip() {
        let mut cpu = CPU::new();
        cpu.register_a = 0;
        cpu.load_and_run(vec![0x48, 0x68, 0x00]);

        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }

    #[test]
    fn test_mixed_stack_operations() {
        let mut cpu = CPU::new();

        cpu.load_and_run(vec![
            0x48,       // PHA - push accumulator (0x11)
            0x08,       // PHP - push status (CARRY)
            0xa9, 0x22, // LDA #$22 - change accumulator
            0x28,       // PLP - restore status
            0x68,       // PLA - restore accumulator
            0x00        // BRK
        ]);

        assert_eq!(cpu.register_a, 0);
        assert_eq!(cpu.status.bitand(CpuFlags::CARRY).bits(), 0);
        assert_eq!(cpu.stack_pointer, 0xFF);
    }

    #[test]
    fn test_stack_underflow_behavior() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0x68, 0x00]); // PLA, BRK

        assert_eq!(cpu.register_a, 0x00);
        assert_eq!(cpu.stack_pointer, 0x00);
    }
}