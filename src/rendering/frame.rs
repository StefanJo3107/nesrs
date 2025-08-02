use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Frame {
    pub data: Vec<u8>,
}

impl Frame {
    const WIDTH: usize = 256;
    const HEIGHT: usize = 240;

    pub fn new() -> Self {
        Frame {
            // multiplied by 3 for each color channel (RGB)
            data: vec![0; (Frame::WIDTH) * (Frame::HEIGHT) * 3],
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, rgb: (u8, u8, u8)) {
        // translating 2D coordinates to 1D array index
        let base = y * 3 * Frame::WIDTH + x * 3;
        if base + 2 < self.data.len() {
            self.data[base] = rgb.0;
            self.data[base + 1] = rgb.1;
            self.data[base + 2] = rgb.2;
        }
    }
}