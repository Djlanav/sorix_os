use crate::drawing::Color;

pub static FONT: &[u8] = include_bytes!("../font.psf");

pub struct PSF1Header {
    pub magic: [u8; 2], // 0x36, 0x04
    pub mode: u8,
    pub glyph_height: u8,
}

pub struct PSFData<'a> {
    header: PSF1Header,
    glyph_count: usize,
    glyphs: &'a [u8]
}

impl<'a> PSFData<'a> {
    pub fn new(header: PSF1Header, count: usize, glyphs: &'a [u8]) -> PSFData<'a> {
        Self {
            header,
            glyph_count: count,
            glyphs
        }
    }
}

impl PSF1Header {
    #[allow(unused_assignments)]
    pub fn init() -> Self {
        let mut magic = [0u8; 2];
        let mut mode = 0u8;
        let mut glyph_height = 0u8;

        let first_bytes = &FONT[..4];
        magic[0] = first_bytes[0];
        magic[1] = first_bytes[1];

        mode = first_bytes[2];
        glyph_height = first_bytes[3];

        Self {
            magic,
            mode,
            glyph_height
        }
    }
}

pub fn draw_char(
    fb: *mut u8,
    header: &PSF1Header,
    glyphs: &[u8],
    fb_width: usize,
    x: usize,
    y: usize,
    ascii_code: u8,
    color: Color
) -> u8 {
    let char_size = header.glyph_height as usize; // Charsize (i.e. glyph height) is 16: VERIFIED
    let start = ascii_code as usize; // First byte
    let end = start + char_size; // Last byte
    
    // Crashes after any number above 78
    let glyph_bytes = &glyphs[16..32];

    for (row, byte) in glyph_bytes.iter().enumerate() {
        for col in 0..8 {
            if byte & (7 >> col) != 0 {
                let px = x + col;
                let py = y + row;

                let index = (py * fb_width + px) * 4;
                let ptr = unsafe { fb.add(index) as *mut u32 };
                unsafe {
                    *ptr = color as u32;
                }
            }
        }
    }

    return 0;
}