use crate::kprintln;

const PAGE_HEAP_START: usize = 0x4000000; // 64 MB
const PAGE_HEAP_END: usize = 0x4500000; // 69 MB
const PAGE_SIZE: usize = 4096;

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