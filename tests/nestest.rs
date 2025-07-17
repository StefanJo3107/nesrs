#[cfg(test)]
mod test {
    use std::fs;
    use nesrs::hw::bus::Bus;
    use nesrs::hw::cartridge::Cartridge;
    use nesrs::hw::cpu::CPU;
    use nesrs::hw::cpu::tracer::trace;
    use nesrs::hw::memory::Memory;

    #[test]
    fn run_nestest() {
        let data: Vec<u8> = fs::read("tests/nestest.nes").unwrap_or_else(|e| panic!("{}", e));
        let bus = Bus::new(Some(Cartridge::new(data).unwrap_or_else(|e| panic!("{}", e))));
        let mut cpu = CPU::new(bus);
        cpu.program_counter = 0xC000;
        let mut result: Vec<String> = vec![];
        cpu.run_with_callback(|cpu| {
            result.push(trace(cpu));
        });

        let nestest_string = fs::read_to_string("tests/nestest_no_cycle.log").unwrap_or_else(|e| panic!("{}", e));
        let nestest_lines: Vec<String> = nestest_string.lines().map(|s| s.to_string()).collect();
        for i in 0..nestest_lines.len() {
            assert_eq!(result[i], nestest_lines[i]);
        }
    }
}