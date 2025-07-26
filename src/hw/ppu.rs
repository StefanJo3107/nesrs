mod address_register;
mod controller_register;
mod mask_register;
mod scroll_register;
mod status_register;
mod tests;

use crate::hw::cartridge::ScreenMirroring;
use crate::hw::ppu::address_register::AddressRegister;
use crate::hw::ppu::controller_register::ControllerRegister;
use crate::hw::ppu::mask_register::MaskRegister;
use crate::hw::ppu::scroll_register::ScrollRegister;
use crate::hw::ppu::status_register::StatusRegister;

pub struct PPU {
    pub chr_rom: Vec<u8>,
    pub mirroring: ScreenMirroring,
    pub address_register: AddressRegister,
    pub controller_register: ControllerRegister,
    pub mask_register: MaskRegister,
    pub scroll_register: ScrollRegister,
    pub status_register: StatusRegister,

    pub vram: [u8; 2048],

    pub oam_address: u8,
    pub oam_data: [u8; 256],
    pub palette_table: [u8; 32],

    internal_data_buf: u8,

    scanline: u16,
    cycles: usize,
    pub nmi_interrupt: Option<u8>,
}

impl PPU {
    pub fn new_empty_rom() -> Self {
        PPU::new(vec![0; 2048], ScreenMirroring::Horizontal)
    }

    pub fn new(chr_rom: Vec<u8>, mirroring: ScreenMirroring) -> Self {
        PPU {
            chr_rom,
            mirroring,
            palette_table: [0; 32],
            vram: [0; 2048],
            address_register: AddressRegister::new(),
            controller_register: ControllerRegister::new(),
            mask_register: MaskRegister::new(),
            scroll_register: ScrollRegister::new(),
            status_register: StatusRegister::new(),
            internal_data_buf: 0,
            oam_data: [0; 256],
            oam_address: 0,
            scanline: 0,
            cycles: 0,
            nmi_interrupt: None,
        }
    }
    pub fn tick(&mut self, cycles: u8) -> bool {
        if self.cycles >= 341 {
            self.cycles = self.cycles - 341;
            self.scanline += 1;

            if self.scanline == 241 {
                if self.controller_register.generate_vblank_nmi() {
                    self.status_register.set_vblank_status(true);
                    todo!("Should trigger NMI interrupt")
                }
            }

            if self.scanline >= 262 {
                self.scanline = 0;
                self.status_register.reset_vblank_status();
                return true;
            }
        }
        return false;
    }
    pub(crate) fn write_to_ppu_addr_reg(&mut self, value: u8) {
        self.address_register.update(value);
    }

    pub(crate) fn write_to_ctrl(&mut self, value: u8) {
        let before_nmi_status = self.controller_register.generate_vblank_nmi();
        self.controller_register.update(value);
        if !before_nmi_status && self.controller_register.generate_vblank_nmi() && self.status_register.is_in_vblank() {
            self.nmi_interrupt = Some(1);
        }
    }

    pub(crate) fn write_to_oam_addr(&mut self, value: u8) {
        self.oam_address = value;
    }

    pub(crate) fn write_to_oam_data(&mut self, value: u8) {
        self.oam_data[self.oam_address as usize] = value;
        self.oam_address = self.oam_address.wrapping_add(1);
    }

    pub(crate) fn read_oam_data(&self) -> u8 {
        self.oam_data[self.oam_address as usize]
    }

    pub(crate) fn write_to_scroll(&mut self, value: u8) {
        self.scroll_register.write(value);
    }

    pub(crate) fn write_to_mask(&mut self, value: u8) {
        self.mask_register.update(value);
    }

    pub(crate) fn read_status(&mut self) -> u8 {
        let data = self.status_register.snapshot();
        self.status_register.reset_vblank_status();
        self.address_register.reset_latch();
        self.scroll_register.reset_latch();
        data
    }

    fn increment_vram_addr(&mut self) {
        self.address_register.increment(self.controller_register.vram_addr_increment());
    }

    pub(crate) fn write_to_data(&mut self, value: u8) {
        let addr = self.address_register.get();
        match addr {
            0..=0x1fff => println!("attempt to write to chr rom space {}", addr),
            0x2000..=0x2fff => {
                self.vram[self.mirror_vram_addr(addr) as usize] = value;
            }
            0x3000..=0x3eff => unimplemented!("addr {} shouldn't be used", addr),

            //Addresses $3F10/$3F14/$3F18/$3F1C are mirrors of $3F00/$3F04/$3F08/$3F0C
            0x3f10 | 0x3f14 | 0x3f18 | 0x3f1c => {
                let add_mirror = addr - 0x10;
                self.palette_table[(add_mirror - 0x3f00) as usize] = value;
            }
            0x3f00..=0x3fff =>
                {
                    self.palette_table[(addr - 0x3f00) as usize] = value;
                }
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
        self.increment_vram_addr();
    }

    pub(crate) fn read_data(&mut self) -> u8 {
        let addr = self.address_register.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.chr_rom[addr as usize];
                result
            }
            0x2000..=0x2FFF => {
                let result = self.internal_data_buf;
                self.internal_data_buf = self.vram[self.mirror_vram_addr(addr) as usize];
                result
            }
            0x3000..=0x3EFF => panic!("addr space 0x3000..0x3eff is not expected to be used, requested = {} ", addr),
            0x3f00..=0x3fff =>
                {
                    self.palette_table[(addr - 0x3f00) as usize]
                }
            _ => panic!("unexpected access to mirrored space {}", addr),
        }
    }

    // Horizontal:
    //   [ A ] [ a ]
    //   [ B ] [ b ]

    // Vertical:
    //   [ A ] [ B ]
    //   [ a ] [ b ]
    pub fn mirror_vram_addr(&self, addr: u16) -> u16 {
        let mirrored_vram = addr & 0b10111111111111; // mirror down 0x3000-0x3eff to 0x2000 - 0x2eff
        let vram_index = mirrored_vram - 0x2000; // to vram vector
        let name_table = vram_index / 0x400; // to the name table index
        match (&self.mirroring, name_table) {
            (ScreenMirroring::Vertical, 2) | (ScreenMirroring::Vertical, 3) => vram_index - 0x800,
            (ScreenMirroring::Horizontal, 2) => vram_index - 0x400,
            (ScreenMirroring::Horizontal, 1) => vram_index - 0x400,
            (ScreenMirroring::Horizontal, 3) => vram_index - 0x800,
            _ => vram_index,
        }
    }

    fn write_oam_dma(&mut self, data: &[u8; 256]) {
        for x in data.iter() {
            self.oam_data[self.oam_address as usize] = *x;
            self.oam_address = self.oam_address.wrapping_add(1);
        }
    }
}