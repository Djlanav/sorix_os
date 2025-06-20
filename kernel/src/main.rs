#![no_main]
#![no_std]

mod drawing;

use drawing::*;

use crate::drawing::characters::{PSF1Header, FONT};

#[repr(C)]
pub struct FramebufferInfo {
    pub addr: *mut u8,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub pixels_per_scan_line: usize,
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn _start(fb_info: *mut FramebufferInfo) -> ! {
    let fb = unsafe { &*fb_info };
    kernel_fill_screen(fb, Color::Purple);

    let psf_header = PSF1Header::init();
    let count = if psf_header.mode & 0x01 != 0 {
        512 as usize
    } else {
        256 as usize
    };
    // Note to self: Ask ChatGPT about setting up potential heap memory allocation for the kernel
    let glyph_data = &FONT[4..];

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}