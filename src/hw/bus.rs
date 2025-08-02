mod tests;

use crate::hw::cartridge::Cartridge;
use crate::hw::joypad::{Joypad, JoypadButton};
use crate::hw::memory::Memory;
use crate::hw::ppu::PPU;

pub struct Bus<'call> {
    cpu_vram: [u8; 2048],
    cartridge: Option<Cartridge>,
    pub(crate) ppu: PPU,
    cycles: usize,

    gameloop_callback: Box<dyn FnMut(&mut PPU, &mut Joypad) + 'call>,
    pub joypad1: Joypad,
    keys_to_press: Vec<JoypadButton>,
    keys_to_release: Vec<JoypadButton>,
}

const RAM_START: u16 = 0x0000;
const RAM_END: u16 = 0x1FFF;
const PPU_REG_END: u16 = 0x3FFF;
const PRG_START: u16 = 0x8000;
const PRG_END: u16 = 0xFFFF;

impl<'a> Bus<'a> {
    pub fn new<'call, F>(cartridge: Option<Cartridge>, gameloop_callback: F) -> Bus<'call>
    where
        F: FnMut(&mut PPU, &mut Joypad) + 'call,
    {
        let ppu = if cartridge.is_some() {
            let c = cartridge.clone().unwrap().clone();
            PPU::new(c.chr_rom, c.screen_mirroring)
        } else { PPU::new_empty_rom() };

        Bus {
            cpu_vram: [0; 2048],
            cartridge,
            ppu,
            cycles: 0,
            gameloop_callback: Box::from(gameloop_callback),
            joypad1: Joypad::new(),
            keys_to_press: vec![],
            keys_to_release: vec![],
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

        let nmi_before = self.ppu.nmi_interrupt.is_some();
        self.ppu.tick(cycles * 3);
        let nmi_after = self.ppu.nmi_interrupt.is_some();

        if !nmi_before && nmi_after {
            (self.gameloop_callback)(&mut self.ppu, &mut self.joypad1);
            self.handle_key_events();
        }
    }

    pub fn handle_key_events(&mut self) {
        for key in &self.keys_to_release {
            self.joypad1.set_button_pressed_status(key, false);
        }
        
        for key in &self.keys_to_press {
            self.joypad1.set_button_pressed_status(key, true);
        }

        self.keys_to_press = vec![];
        self.keys_to_release = vec![];
    }

    pub fn set_key_to_press(&mut self, key_to_press: JoypadButton) {
        self.keys_to_press.push(key_to_press);
    }

    pub fn set_key_to_release(&mut self, key_to_release: JoypadButton) {
        self.keys_to_release.push(key_to_release);
    }
}

impl<'a> Memory for Bus<'a> {
    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            RAM_START..=RAM_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }
            0x2000 | 0x2001 | 0x2003 | 0x2005 | 0x2006 | 0x4014 => {
                // panic!("Attempt to read from write-only PPU address {:x}", addr);
                0
            }
            0x2002 => self.ppu.read_status(),
            0x2004 => self.ppu.read_oam_data(),
            0x2007 => self.ppu.read_data(),
            0x2008..=PPU_REG_END => {
                let mirror_down_addr = addr & 0b00100000_00000111;
                self.mem_read(mirror_down_addr)
            }
            0x4016 => {
                self.joypad1.read()
            }

            0x4017 => {
                // ignore joypad 2
                0
            }
            PRG_START..=PRG_END => {
                self.read_prg_rom(addr)
            }
            _ => {
                // println!("Ignoring mem access at {:#x}", addr);
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
            // https://wiki.nesdev.com/w/index.php/PPU_programmer_reference#OAM_DMA_.28.244014.29_.3E_write
            0x4014 => {
                let mut buffer: [u8; 256] = [0; 256];
                let hi: u16 = (data as u16) << 8;
                for i in 0..256u16 {
                    buffer[i as usize] = self.mem_read(hi + i);
                }

                self.ppu.write_oam_dma(&buffer);

                // todo: handle this eventually
                // let add_cycles: u16 = if self.cycles % 2 == 1 { 514 } else { 513 };
                // self.tick(add_cycles); //todo this will cause weird effects as PPU will have 513/514 * 3 ticks
            }
            0x4016 => {
                self.joypad1.write(data);
            }
            0x4017 => {
                // ignore joypad 2
            }
            0x8000..=0xFFFF => panic!("Attempt to write to Cartridge ROM space: {:x}", addr),
            _ => {
                // println!("Ignoring mem write-access at {:x}", addr);
            }
        }
    }
}
