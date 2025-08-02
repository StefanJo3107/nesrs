use nesrs::api::emulator::Emulator;

fn main() {
    let mut emu = Emulator::new("/home/stefan/Dev/nesrs/assets/pacman.nes", true).unwrap();
    emu.reset_cpu();
    loop {
        emu.step_emulation();
    }
}
