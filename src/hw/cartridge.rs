mod tests;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum ScreenMirroring {
    Horizontal,
    Vertical,
    FourScreen,
}

#[derive(Error, Debug)]
pub enum CartridgeError {
    #[error("Invalid INES header (expected {expected:?}, got {found:?})")]
    InvalidHeader {
        expected: Vec<u8>,
        found: Vec<u8>,
    },
    #[error("NES2.0 format is not supported")]
    UnsupportedINESVersion,
    #[error("Illegal screen mirroring found")]
    IllegalScreenMirroring,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: ScreenMirroring,
}

impl Cartridge {
    const INES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
    pub(crate) const PRG_ROM_PAGE_SIZE: usize = 16384;
    pub(crate) const CHR_ROM_PAGE_SIZE: usize = 8192;
    const HEADER_SIZE: usize = 16;
    const TRAINER_SIZE: usize = 512;

    pub fn new(raw: Vec<u8>) -> anyhow::Result<Self> {
        if raw[0..4] != Self::INES_TAG {
            return Err(CartridgeError::InvalidHeader
            { expected: Self::INES_TAG.to_vec(), found: raw[0..4].to_vec() }.into());
        }

        let cb1 = raw[6];
        let cb2 = raw[7];

        let mapper = (cb2 & 0b1111_0000) | (cb1 >> 4);
        let ines_ver = cb2 & 0b0000_1100;
        if ines_ver != 0 {
            return Err(CartridgeError::UnsupportedINESVersion.into());
        }

        let vertical = cb1 & 0b0000_0001 != 0;
        let horizontal = cb1 & 0b0000_0010 == 0;
        let four_screen = cb1 & 0b0000_1000 != 0;

        let mirroring = match (vertical, horizontal, four_screen) {
            (true, _, false) => ScreenMirroring::Vertical,
            (false, true, false) => ScreenMirroring::Horizontal,
            (_, _, true) => ScreenMirroring::FourScreen,
            _ => return Err(CartridgeError::IllegalScreenMirroring.into())
        };

        let prg_rom_size = raw[4] as usize * Self::PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * Self::CHR_ROM_PAGE_SIZE;

        let skip_trainer = cb1 & 0b0000_0100 == 0;
        let prg_rom_start = Self::HEADER_SIZE + if skip_trainer { 0 } else { Self::TRAINER_SIZE };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Ok(Cartridge {
            prg_rom: raw[prg_rom_start..prg_rom_start + prg_rom_size].to_vec(),
            chr_rom: raw[chr_rom_start..chr_rom_start + chr_rom_size].to_vec(),
            mapper,
            screen_mirroring: mirroring,
        })
    }
}