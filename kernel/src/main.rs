#![no_std]
#![no_main]

mod drawing;
mod kernel;
extern crate alloc;

use alloc::boxed::Box;
use drawing::*;
use linked_list_allocator::LockedHeap;

use crate::drawing::fonts::draw_string_raw;
use crate::kernel::serial_io::serial_init;
use crate::kernel::{string_api::Shell, Kernel};
use crate::kernel::{ahci, pci, prelude::*};
//use crate::alloc::string::ToString;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const HEAP_START: *mut u8 = 0x3000000 as *mut u8; // 48 MB into memory
const HEAP_SIZE: usize = 0x100000; // 1 MB Heap

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

    serial_init();

    pci::scan_pci_devices();
    if let Some(hba) = ahci::scan_pci_for_ahci() {
        if let Some(port_index) = ahci::find_ahci_device(&hba) {
            let port = hba.ports[port_index].clone();
            ahci::stop_command_engine(port.clone());
            ahci::initialize_port(port.clone());

            ahci::cmd_management::create_command_header(port.clone());

            ahci::cmd_management::check_integrity(port.clone());

            ahci::cmd_management::setup_command_table(port.clone());

            ahci::cmd_management::check_integrity(port.clone());

            ahci::cmd_management::create_prdt_entry(port.clone());

            ahci::cmd_management::check_integrity(port.clone());

            ahci::cmd_management::issue_command(port.clone());

            ahci::cmd_management::check_integrity(port.clone());

            //ahci::cmd_management::read_data_buffer(port.clone());
        }
    }

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