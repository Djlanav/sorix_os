#![no_std]
#![no_main]

mod drawing;
mod kernel;
extern crate alloc;

use alloc::boxed::Box;
use drawing::*;
use linked_list_allocator::LockedHeap;

use crate::kernel::Kernel;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_START: *mut u8 = 0x3f56000 as *mut u8; // Start Address (DO. NOT. CHANGE.)
const HEAP_SIZE: usize = 1024 * 1024; // 1 MB Heap

#[repr(C)]
pub struct FramebufferInfo<'a> {
    pub buffer: &'a mut [u32],
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
    let fb_box = unsafe {
        Box::from_raw(fb_info)
    };

    let mut kernel = Kernel::start(fb_box);
    kernel.fill_screen(Color::Purple);

    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}