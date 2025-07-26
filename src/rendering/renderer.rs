use crate::hw::ppu::PPU;
use crate::rendering::frame::Frame;
use crate::rendering::palette;

fn bg_pallette(ppu: &PPU, tile_column: usize, tile_row: usize) -> [u8; 4] {
    let attr_table_idx = tile_row / 4 * 8 + tile_column / 4;
    let attr_byte = ppu.vram[0x3c0 + attr_table_idx];  // note: still using hardcoded first nametable

    let pallet_idx = match (tile_column % 4 / 2, tile_row % 4 / 2) {
        (0, 0) => attr_byte & 0b11,
        (1, 0) => (attr_byte >> 2) & 0b11,
        (0, 1) => (attr_byte >> 4) & 0b11,
        (1, 1) => (attr_byte >> 6) & 0b11,
        (_, _) => panic!("should not happen"),
    };

    let pallete_start: usize = 1 + (pallet_idx as usize) * 4;
    [ppu.palette_table[0], ppu.palette_table[pallete_start], ppu.palette_table[pallete_start + 1], ppu.palette_table[pallete_start + 2]]
}

pub fn render_bg(ppu: &PPU, frame: &mut Frame) {
    let bank = ppu.controller_register.bknd_pattern_addr();

    for i in 0..0x03c0 { // just for now, lets use the first nametable
        let tile = ppu.vram[i] as u16;
        let tile_column = i % 32;
        let tile_row = i / 32;
        let tile = &ppu.chr_rom[(bank + tile * 16) as usize..=(bank + tile * 16 + 15) as usize];
        let palette = bg_pallette(ppu, tile_column, tile_row);

        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];

            for x in (0..=7).rev() {
                let value = (1 & lower) << 1 | (1 & upper);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => palette::SYSTEM_PALLETE[ppu.palette_table[0] as usize],
                    1 => palette::SYSTEM_PALLETE[palette[1] as usize],
                    2 => palette::SYSTEM_PALLETE[palette[2] as usize],
                    3 => palette::SYSTEM_PALLETE[palette[3] as usize],
                    _ => panic!("invalid palette index"),
                };
                frame.set_pixel(tile_column * 8 + x, tile_row * 8 + y, rgb)
            }
        }
    }
}

fn sprite_palette(ppu: &PPU, pallete_idx: u8) -> [u8; 4] {
    let start = 0x11 + (pallete_idx * 4) as usize;
    [
        0,
        ppu.palette_table[start],
        ppu.palette_table[start + 1],
        ppu.palette_table[start + 2],
    ]
}

pub fn render_sprites(ppu: &PPU, frame: &mut Frame) {
    for i in (0..ppu.oam_data.len()).step_by(4).rev() {
        let tile_idx = ppu.oam_data[i + 1] as u16;
        let tile_x = ppu.oam_data[i + 3] as usize;
        let tile_y = ppu.oam_data[i] as usize;

        let flip_vertical = if ppu.oam_data[i + 2] >> 7 & 1 == 1 {
            true
        } else {
            false
        };
        let flip_horizontal = if ppu.oam_data[i + 2] >> 6 & 1 == 1 {
            true
        } else {
            false
        };
        let pallette_idx = ppu.oam_data[i + 2] & 0b11;
        let sprite_palette = sprite_palette(ppu, pallette_idx);

        let bank: u16 = ppu.controller_register.sprt_pattern_addr();

        let tile = &ppu.chr_rom[(bank + tile_idx * 16) as usize..=(bank + tile_idx * 16 + 15) as usize];


        for y in 0..=7 {
            let mut upper = tile[y];
            let mut lower = tile[y + 8];
            for x in (0..=7).rev() {
                let value = (1 & lower) << 1 | (1 & upper);
                upper = upper >> 1;
                lower = lower >> 1;
                let rgb = match value {
                    0 => None, // skip coloring the pixel
                    1 => Some(palette::SYSTEM_PALLETE[sprite_palette[1] as usize]),
                    2 => Some(palette::SYSTEM_PALLETE[sprite_palette[2] as usize]),
                    3 => Some(palette::SYSTEM_PALLETE[sprite_palette[3] as usize]),
                    _ => panic!("invalid palette index"),
                };

                if rgb.is_none() {
                    continue;
                }

                match (flip_horizontal, flip_vertical) {
                    (false, false) => frame.set_pixel(tile_x + x, tile_y + y, rgb.unwrap()),
                    (true, false) => frame.set_pixel(tile_x + 7 - x, tile_y + y, rgb.unwrap()),
                    (false, true) => frame.set_pixel(tile_x + x, tile_y + 7 - y, rgb.unwrap()),
                    (true, true) => frame.set_pixel(tile_x + 7 - x, tile_y + 7 - y, rgb.unwrap()),
                }
            }
        }
    }
}

pub fn render(ppu: &PPU, frame: &mut Frame) {
    render_bg(ppu, frame);
    render_sprites(ppu, frame);
}