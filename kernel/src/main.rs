#![no_std]
#![no_main]

mod drawing;
mod kernel;
extern crate alloc;

use alloc::boxed::Box;
use drawing::*;
use linked_list_allocator::LockedHeap;

use crate::kernel::pci::{ahci_probe_port_type, find_ahci_device};
use crate::kernel::{string_api::Shell, Kernel};
use crate::kernel::{pci, prelude::*};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_START: *mut u8 = 0x3f56000 as *mut u8; // Start Address (DO. NOT. CHANGE.)
const HEAP_SIZE: usize = 1024usize.pow(2); // 1 MB Heap

const MAIN_FONT: &[u8] = include_bytes!("drawing/font.psf");

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

    let shell = Shell::new();
    let mut kernel = Kernel::start(fb_box, shell);
    kernel.fill_screen(Color::Black);

    pci::scan_pci_devices();
    pci::scan_pci_for_ahci();

    KERNEL_EVENT_MANAGER.lock().run(&mut kernel);
    KERNEL_EVENT_MANAGER.lock().clean_events();
    
    loop {

    }
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}