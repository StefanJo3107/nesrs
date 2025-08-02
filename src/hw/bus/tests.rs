#[cfg(test)]
mod tests {
    use crate::hw::bus::Bus;
    use crate::hw::memory::Memory;

    #[test]
    fn test_bus_new() {
        let bus = Bus::new(None, move |_, _| {});
        // Verify that the bus is initialized with zeros
        for i in 0..2048 {
            assert_eq!(bus.cpu_vram[i], 0);
        }
    }

    #[test]
    fn test_ram_read_write() {
        let mut bus = Bus::new(None, move |_, _| {});

        bus.mem_write(0x0000, 0x42);
        assert_eq!(bus.mem_read(0x0000), 0x42);

        bus.mem_write(0x0100, 0xAB);
        bus.mem_write(0x1FFF, 0xCD);
        assert_eq!(bus.mem_read(0x0100), 0xAB);
        assert_eq!(bus.mem_read(0x1FFF), 0xCD);
    }

    #[test]
    fn test_ram_mirroring() {
        let mut bus = Bus::new(None, move |_, _| {});

        bus.mem_write(0x0000, 0x55);

        assert_eq!(bus.mem_read(0x0800), 0x55); // 0x0000 + 0x0800
        assert_eq!(bus.mem_read(0x1000), 0x55); // 0x0000 + 0x1000
        assert_eq!(bus.mem_read(0x1800), 0x55); // 0x0000 + 0x1800

        bus.mem_write(0x0800, 0x77);
        assert_eq!(bus.mem_read(0x0000), 0x77);
        assert_eq!(bus.mem_read(0x1000), 0x77);
        assert_eq!(bus.mem_read(0x1800), 0x77);
    }

    #[test]
    fn test_ram_mirroring_boundary() {
        let mut bus = Bus::new(None, move |_, _| {});

        bus.mem_write(0x07FF, 0x99);
        assert_eq!(bus.mem_read(0x0FFF), 0x99);
        assert_eq!(bus.mem_read(0x17FF), 0x99);
        assert_eq!(bus.mem_read(0x1FFF), 0x99);
    }

    #[test]
    fn test_ram_mirroring_mask() {
        let mut bus = Bus::new(None, move |_, _| {});

        let test_cases = vec![
            (0x0000, 0x0000),
            (0x0800, 0x0000),
            (0x1000, 0x0000),
            (0x1800, 0x0000),
            (0x0123, 0x0123),
            (0x0923, 0x0123),
            (0x1123, 0x0123),
            (0x1923, 0x0123),
        ];

        for (write_addr, expected_internal_addr) in test_cases {
            bus.mem_write(write_addr, 0x42);
            assert_eq!(bus.cpu_vram[expected_internal_addr], 0x42);
        }
    }

    #[test]
    fn test_ppu_read_panics() {
        let mut bus = Bus::new(None, move |_, _| {});
        bus.mem_read(0x2000);
    }

    #[test]
    fn test_ppu_write_panics() {
        let mut bus = Bus::new(None, move |_, _| {});
        bus.mem_write(0x2000, 0x42);
    }

    #[test]
    fn test_ppu_read_end_panics() {
        let mut bus = Bus::new(None, move |_, _| {});
        bus.mem_read(0x3FFF); // PPU register end
    }

    #[test]
    fn test_ppu_write_end_panics() {
        let mut bus = Bus::new(None, move |_, _| {});
        bus.mem_write(0x3FFF, 0x42);
    }

    #[test]
    fn test_ppu_middle_range_panics() {
        let mut bus = Bus::new(None, move |_, _| {});
        bus.mem_write(0x2500, 0x42);
    }

    #[test]
    fn test_unmapped_memory_read() {
        let mut bus = Bus::new(None, move |_, _| {});

        assert_eq!(bus.mem_read(0x4000), 0);
        assert_eq!(bus.mem_read(0x8000), 0);
        assert_eq!(bus.mem_read(0xFFFF), 0);
    }

    #[test]
    #[should_panic]
    fn test_unmapped_memory_write() {
        let mut bus = Bus::new(None, move |_, _| {});

        bus.mem_write(0x4000, 0x42);
        bus.mem_write(0x8000, 0x55);
        bus.mem_write(0xFFFF, 0x99);

        assert_eq!(bus.mem_read(0x4000), 0);
        assert_eq!(bus.mem_read(0x8000), 0);
        assert_eq!(bus.mem_read(0xFFFF), 0);
    }

    #[test]
    fn test_ram_full_range() {
        let mut bus = Bus::new(None, move |_, _| {});

        for addr in 0x0000..=0x1FFF {
            let value = (addr & 0xFF) as u8;
            bus.mem_write(addr, value);
            assert_eq!(bus.mem_read(addr), value);
        }
    }

    #[test]
    fn test_ram_independence() {
        let mut bus = Bus::new(None, move |_, _| {});

        bus.mem_write(0x0000, 0x11);
        bus.mem_write(0x0001, 0x22);
        bus.mem_write(0x0002, 0x33);

        assert_eq!(bus.mem_read(0x0000), 0x11);
        assert_eq!(bus.mem_read(0x0001), 0x22);
        assert_eq!(bus.mem_read(0x0002), 0x33);
    }

    #[test]
    fn test_write_mask_discrepancy() {
        let mut bus = Bus::new(None, move |_, _| {});

        bus.mem_write(0x0800, 0x42);
        assert_eq!(bus.mem_read(0x0000), 0x42);
        assert_eq!(bus.mem_read(0x0800), 0x42);
    }
}