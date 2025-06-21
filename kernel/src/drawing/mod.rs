pub mod characters;
use crate::FramebufferInfo;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Color {
    Purple = 0x8b2ef5,
    LightBlue = 0x4cace3,
    Red = 0xFF0000,
}

#[unsafe(no_mangle)]
pub fn kernel_draw_screen_area(fb: &FramebufferInfo, width: usize, height: usize, color: Color) {
    for x in 0..width {
        for y in 0..height {
            unsafe {
                let index = y * fb.pixels_per_scan_line + x;
                let ptr = fb.addr.add(index * 4) as *mut u32;
                *ptr = color as u32;
            }
        }
    }
}

pub fn kernel_fill_screen(fb: &FramebufferInfo, color: Color) {
    kernel_draw_screen_area(fb, 800, 600, color);
}