use nesrs::api::emulator::{Emulator, EmulatorTrigger};

fn main() {
    let mut emu = Emulator::new("/home/stefan/Dev/nesrs/assets/pacman-level1.cpu",
                                true, vec![EmulatorTrigger::MemEquals { addr: 0x67, value: 0 }]).unwrap();
    emu.reset_cpu();
    loop {
        let trigger = emu.step_emulation();
        if trigger {
            emu.reset_cpu();
        }
    }
}
