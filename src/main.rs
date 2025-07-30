use std::collections::HashMap;
use std::time::{Duration, SystemTime, SystemTimeError};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use nesrs::hw::bus::Bus;
use nesrs::hw::cartridge::Cartridge;
use nesrs::hw::cpu::CPU;
use nesrs::hw::emulator::Emulator;
use nesrs::hw::joypad;
use nesrs::hw::joypad::JoypadButton;
use nesrs::hw::ppu::PPU;
use nesrs::rendering::frame::Frame;
use nesrs::rendering::{renderer, tile_viewer};

fn main() {
    let mut emulator = Emulator::new("/home/stefan/Dev/nesrs/assets/supermario.nes", true);
    emulator.reset_cpu();
    loop {
        emulator.step_emulation();
    }
}
