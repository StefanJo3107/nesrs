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
    fn test_dex() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x02,  // LDX #$02
                              0xca,       // DEX
                              0xca,       // DEX
                              0x00]);
        assert_eq!(cpu.register_x, 0);
    }

    #[test]
    fn test_dex_underflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa2, 0x00,  // LDX #$00
                              0xca,       // DEX (should wrap to 0xff)
                              0x00]);
        assert_eq!(cpu.register_x, 0xff);
    }

    #[test]
    fn test_dey() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x02,  // LDY #$02
                              0x88,       // DEY
                              0x88,       // DEY
                              0x00]);
        assert_eq!(cpu.register_y, 0);
    }

    #[test]
    fn test_dey_underflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0x00,  // LDY #$00
                              0x88,       // DEY (should wrap to 0xff)
                              0x00]);
        assert_eq!(cpu.register_y, 0xff);
    }

    #[test]
    fn test_inc() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xfe,  // LDA #$fe
                              0x85, 0x10,  // STA $10
                              0xe6, 0x10,  // INC $10
                              0xe6, 0x10,  // INC $10
                              0x00]);
        assert_eq!(cpu.mem_read(0x10), 0x00);
    }

    #[test]
    fn test_iny() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa0, 0xfe,  // LDY #$fe
                              0xc8,       // INY
                              0xc8,       // INY
                              0x00]);
        assert_eq!(cpu.register_y, 0x00);
    }

    #[test]
    fn test_dec() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x02,  // LDA #$02
                              0x85, 0x10,  // STA $10
                              0xc6, 0x10,  // DEC $10
                              0xc6, 0x10,  // DEC $10
                              0x00]);
        assert_eq!(cpu.mem_read(0x10), 0x00);
    }

    #[test]
    fn test_dec_underflow() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00,  // LDA #$00
                              0x85, 0x10,  // STA $10
                              0xc6, 0x10,  // DEC $10 (should wrap to 0xff)
                              0x00]);
        assert_eq!(cpu.mem_read(0x10), 0xff);
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

    #[test]
    fn test_and() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x0f,     // LDA #$0f
            0x29, 0x55,     // AND #$55
            0x00
        ]);
        assert_eq!(cpu.register_a, 0x05);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(!cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_and_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0xff,     // LDA #$ff
            0x29, 0x00,     // AND #$00
            0x00
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_and_negative() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x80,     // LDA #$80
            0x29, 0xff,     // AND #$ff
            0x00
        ]);
        assert_eq!(cpu.register_a, 0x80);
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_eor() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0xaa,     // LDA #$aa
            0x49, 0x55,     // EOR #$55
            0x00
        ]);
        assert_eq!(cpu.register_a, 0xff);
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_eor_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x55,     // LDA #$55
            0x49, 0x55,     // EOR #$55
            0x00
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_ora() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x0f,     // LDA #$0f
            0x09, 0xf0,     // ORA #$f0
            0x00
        ]);
        assert_eq!(cpu.register_a, 0xff);
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_ora_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x00,     // LDA #$00
            0x09, 0x00,     // ORA #$00
            0x00
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_ora_memory() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x33,     // LDA #$33
            0x85, 0x10,     // STA $10
            0xa9, 0x0c,     // LDA #$0c
            0x05, 0x10,     // ORA $10
            0x00
        ]);
        assert_eq!(cpu.register_a, 0x3f);
        assert!(!cpu.status.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_and_memory() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0xff,     // LDA #$ff
            0x85, 0x20,     // STA $20
            0xa9, 0xaa,     // LDA #$aa
            0x25, 0x20,     // AND $20
            0x00
        ]);
        assert_eq!(cpu.register_a, 0xaa);
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_eor_memory() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x55,     // LDA #$55
            0x85, 0x30,     // STA $30
            0xa9, 0xaa,     // LDA #$aa
            0x45, 0x30,     // EOR $30
            0x00
        ]);
        assert_eq!(cpu.register_a, 0xff);
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_asl_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x42,     // LDA #$42
            0x0a,           // ASL A
            0x00
        ]);
        assert_eq!(cpu.register_a, 0x84);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
        assert!(cpu.status.contains(CpuFlags::NEGATIVE));
    }

    #[test]
    fn test_asl_carry() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x81,     // LDA #$81
            0x0a,           // ASL A
            0x00
        ]);
        assert_eq!(cpu.register_a, 0x02);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_asl_memory() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x33,     // LDA #$33
            0x85, 0x20,     // STA $20
            0x06, 0x20,     // ASL $20
            0x00
        ]);
        assert_eq!(cpu.mem_read(0x20), 0x66);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_lsr_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x84,     // LDA #$84
            0x4a,           // LSR A
            0x00
        ]);
        assert_eq!(cpu.register_a, 0x42);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(!cpu.status.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_lsr_carry_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x01,     // LDA #$01
            0x4a,           // LSR A
            0x00
        ]);
        assert_eq!(cpu.register_a, 0x00);
        assert!(cpu.status.contains(CpuFlags::CARRY));
        assert!(cpu.status.contains(CpuFlags::ZERO));
    }

    #[test]
    fn test_rol() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x41,     // LDA #$41
            0x0a,           // ASL A (sets carry)
            0xa9, 0x80,     // LDA #$80
            0x2a,           // ROL A (rotate with carry)
            0x00
        ]);
        assert_eq!(cpu.register_a, 0);
        assert!(cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_ror() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x02,     // LDA #$02
            0x4a,           // LSR A (sets carry)
            0xa9, 0x80,     // LDA #$80
            0x6a,           // ROR A (rotate with carry)
            0x00
        ]);
        assert_eq!(cpu.register_a, 0x40);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
    }

    #[test]
    fn test_ror_memory_zero() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![
            0xa9, 0x00,     // LDA #$00
            0x85, 0x40,     // STA $40
            0x66, 0x40,     // ROR $40
            0x00
        ]);
        assert_eq!(cpu.mem_read(0x40), 0x00);
        assert!(!cpu.status.contains(CpuFlags::CARRY));
        assert!(cpu.status.contains(CpuFlags::ZERO));
    }
}