mod address_register;

use crate::hw::cartridge::ScreenMirroring;
use crate::hw::ppu::address_register::AddressRegister;

pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub mirroring: ScreenMirroring,
    pub address_register: AddressRegister,
}

impl PPU {
    pub fn new(chr_rom: Vec<u8>, mirroring: ScreenMirroring) -> Self {
        PPU {
            chr_rom,
            mirroring,
            palette_table: [0; 32],
            vram: [0; 2048],
            oam_data: [0; 256],
            address_register: AddressRegister::new(),
        }
    }

    fn write_to_ppu_addr_reg(&mut self, value: u8) {
        self.address_register.update(value);
    }
}