#![no_main]
#![no_std]

mod drawing;
extern crate alloc;

use alloc::vec::Vec;
//use alloc::vec::Vec;
use drawing::*;
use linked_list_allocator::LockedHeap;

use crate::drawing::characters::{PSF1Header, FONT};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_START: *mut u8 = 0x3f56000 as *mut u8; // Start Address
const HEAP_SIZE: usize = 1024 * 1024; // 1 MB Heap

#[repr(C)]
pub struct FramebufferInfo {
    pub addr: *mut u8,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub pixels_per_scan_line: usize,
}

pub fn kernel_heap_init() {
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }
}

#[unsafe(no_mangle)]
pub extern "sysv64" fn _start(fb_info: *mut FramebufferInfo) -> ! {
    kernel_heap_init();
    let fb = unsafe { &*fb_info };
    kernel_fill_screen(fb, Color::Purple);

    let psf_header = PSF1Header::init();
    let _count = if psf_header.mode & 0x01 != 0 {
        512 as usize
    } else {
        256 as usize
    };
    // Note to self: Ask ChatGPT about setting up potential heap memory allocation for the kernel
    let _glyph_data = &FONT[4..];
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);

    loop {}
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}