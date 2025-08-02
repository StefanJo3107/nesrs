use nesrs::api::emulator::Emulator;

fn main() {
    let mut emu = Emulator::new("/home/stefan/Dev/nesrs/assets/pacman-level1.cpu",
                                true, vec![]).unwrap();
    emu.reset_cpu();
    loop {
        emu.step_emulation();
    }
}
