mod tests;

use crate::hw::cartridge::Cartridge;
use crate::hw::memory::Memory;

pub struct Bus {
    cpu_vram: [u8; 2048],
    cartridge: Option<Cartridge>,
}

const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const PPU_REG_START: u16 = 0x2000;
const PPU_REG_END: u16 = 0x3FFF;
const PRG_START: u16 = 0x8000;
const PRG_END: u16 = 0xFFFF;

impl Bus {
    pub fn new(cartridge: Option<Cartridge>) -> Self {
        Bus {
            cpu_vram: [0; 2048],
            cartridge,
        }
    }

    pub fn insert_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(cartridge);
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
}

impl Memory for Bus {
    fn mem_read(&self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            PPU_REG_START..=PPU_REG_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU is not supported yet")
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
            PPU_REG_START..=PPU_REG_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU is not supported yet");
            }
            PRG_START..=PRG_END => {
                panic!("Attempt to write to Cartridge ROM space")
            }
            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }
}
