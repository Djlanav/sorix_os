pub mod fonts;

use crate::kernel::Kernel;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Color {
    White = 0xffffff,
    Black = 0x000000,
    Purple = 0x8b2ef5,
    LightBlue = 0x4cace3,
    Red = 0xff0000,
}

impl Kernel<'_> {
    pub fn draw_area(&mut self, width: usize, height: usize, color: Color) {
        for x in 0..width {
            for y in 0..height {
                let index = y * self.framebuffer.pixels_per_scan_line + x;
                self.framebuffer.buffer[index] = color as u32;
            }
        }
    }

    pub fn fill_screen(&mut self, color: Color) {
        self.draw_area(self.framebuffer.width, self.framebuffer.height, color);
    }
}