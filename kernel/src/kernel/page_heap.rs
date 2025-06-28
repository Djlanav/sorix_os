use crate::alloc::string::ToString;
use core::ptr::write_bytes;

use crate::kprintln;

const PAGE_HEAP_START: usize = 0x4100000; // 65 MB
const PAGE_HEAP_END: usize = 0x4600000; // 70 MB
const PAGE_SIZE: usize = 4096; // 4 KB

static mut NEXT_FREE_PAGE: usize = PAGE_HEAP_START;

pub fn allocate_page() -> *mut u8 {
    unsafe {
        if NEXT_FREE_PAGE + PAGE_SIZE > PAGE_HEAP_END {
            kprintln!("Could not allocate page: Out of Memory!");
        }

        let ptr = NEXT_FREE_PAGE as *mut u8;
        NEXT_FREE_PAGE += PAGE_SIZE;

        kprintln!("Allocated page at: {:#x}", ptr as usize);
        ptr
    }
}

pub fn zero_page(pointer: *mut u8, count: Option<usize>) {
    unsafe {
        match count {
            Some(c) => write_bytes(pointer, 0, c),
            None => write_bytes(pointer, 0, PAGE_SIZE)
        }
    }
}