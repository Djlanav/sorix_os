use crate::{drawing::Color, FramebufferInfo};

#[derive(Debug)]
pub struct PsfFont<'a> {
    glyphs: &'a [u8],
    glyph_height: usize,
    glyph_num: usize,
}

impl<'a> PsfFont<'a> {
    pub fn from_bytes(data: &'a [u8]) -> Option<Self> {
        if data.len() < 4 || data[0] != 0x36 || data[1] != 0x04 {
            return None;
        }

        let mode = data[2];
        let glyph_height = data[3] as usize;
        let glyph_num = if mode & 0x01 != 0 { 512 } else { 256 };
        let glyphs = &data[4..];

        if glyphs.len() < glyph_height * glyph_num {
            return None;
        }

        Some(Self {
            glyphs,
            glyph_height,
            glyph_num
        })
    }

    pub fn glyph_for(&self, ascii: u8) -> &[u8] {
        let index = ascii as usize;
        let start = index * self.glyph_height;
        let end = start + self.glyph_height;

        &self.glyphs[start..end]
    }
}

pub fn draw_char(
    fb: &mut FramebufferInfo,
    font: &PsfFont,
    ascii: u8,
    x: usize,
    y: usize,
    color: Color
) {
    let glyph = font.glyph_for(ascii);

    for (row, byte) in glyph.iter().enumerate() {
        for col in 0..8 {
            let mask = 0x80 >> col;
            if byte & mask != 0 {
                let px = x + col;
                let py = y + row;

                if px < fb.width && py < fb.height {
                    let index = py * fb.pixels_per_scan_line + px;
                    fb.buffer[index] = color as u32;
                }
            }
        }
    }
}

pub fn draw_string(fb: &mut FramebufferInfo, font: &PsfFont, text: &str, x: usize, y: usize, color: Color) {
    let mut x_offset = x;
    
    for byte in text.bytes() {
        if byte == b'\n' {
            x_offset = x;
            continue;
        }

        draw_char(fb, font, byte, x_offset, y, color);
        x_offset += 8;
    }
}

pub fn draw_string_raw(fb: &mut FramebufferInfo, font: &PsfFont, text: &str, x: usize, y: usize, color: Color) {
    let mut x_offset = x;
    
    for byte in text.bytes() {
        if byte == b'\n' {
            x_offset = x;
            continue;
        }

        draw_char(fb, font, byte, x_offset, y, color);
        x_offset += 8;
    }
}