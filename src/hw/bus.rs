mod tests;

use crate::hw::cartridge;
use crate::hw::cartridge::Cartridge;
use crate::hw::memory::Memory;
use crate::hw::ppu::PPU;

pub struct Bus {
    cpu_vram: [u8; 2048],
    cartridge: Option<Cartridge>,
    ppu: PPU,
    cycles: usize,
}

const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;
const PRG_START: u16 = 0x8000;
const PRG_END: u16 = 0xFFFF;

impl Bus {
    pub fn new(cartridge: Option<Cartridge>) -> Self {
        let ppu = if cartridge.is_some() {
            let c = cartridge.clone().unwrap().clone();
            PPU::new(c.chr_rom, c.screen_mirroring)
        } else { PPU::new_empty_rom() };

        Bus {
            cpu_vram: [0; 2048],
            cartridge,
            ppu,
            cycles: 0,
        }
    }

    pub fn insert_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(cartridge.clone());
        self.ppu = PPU::new(cartridge.chr_rom.clone(), cartridge.screen_mirroring);
    }

    fn read_prg_rom(&self, mut addr: u16) -> u8 {
        addr -= 0x8000;
        let cartridge = self.cartridge.as_ref();
        if let Some(c) = cartridge {
            if c.prg_rom.len() == 0x4000 && addr >= 0x4000 {
                //mirror if needed
                addr = addr % 0x4000;
            }
            c.prg_rom[addr as usize]
        } else {
            0
        }
    }

    pub fn poll_nmi_status(&mut self) -> Option<u8> {
        self.ppu.nmi_interrupt.take()
    }

    pub fn tick(&mut self, cycles: u8) {
        self.cycles += cycles as usize;
        self.ppu.tick(cycles * 3);
    }
}

impl Memory for Bus {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                panic!("Attempt to read from write-only PPU address {:x}", addr);
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),
            0x2008..=PPU_REG_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(mirror_down_addr)
            }
            PRG_START..=PRG_END => {
                self.read_prg_rom(addr)
            }
            _ => {
                println!("Ignoring mem access at {:#x}", addr);
                0
            }
        }
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        match addr {
            RAM_START..=RAM_END => {
                let mirror_down_addr = addr & 0b11111111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            0x2000 => {
                self.ppu.write_to_ctrl(data);
            }
            0x2001 => {
                self.ppu.write_to_mask(data);
            }
            0x2002 => panic!("attempt to write to PPU status register"),

            0x2003 => {
                self.ppu.write_to_oam_addr(data);
            }
            0x2004 => {
                self.ppu.write_to_oam_data(data);
            }
            0x2005 => {
                self.ppu.write_to_scroll(data);
            }

            0x2006 => {
                self.ppu.write_to_ppu_addr_reg(data);
            }
            0x2007 => {
                self.ppu.write_to_data(data);
            }
            0x2008..=PPU_REG_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_write(mirror_down_addr, data);
            }
            0x8000..=0xFFFF => panic!("Attempt to write to Cartridge ROM space: {:x}", addr),
            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }
}
