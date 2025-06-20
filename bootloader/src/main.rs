#![no_main]
#![no_std]

mod dir_management;
mod utils;

extern crate alloc;

use alloc::boxed::Box;
use dir_management::*;

use log::*;
use uefi::{boot::{open_protocol_exclusive, MemoryType}, mem::memory_map::MemoryMap, prelude::*, proto::{console::gop::GraphicsOutput, media::file::File}};
use linked_list_allocator::LockedHeap;

use crate::utils::framebuffer::FramebufferInfo;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();
    info!("Begin boot process");
    info!("Initializing heap");

    let mut heap_space = [0u8; 1024 * 100];
    unsafe {
        ALLOCATOR.lock().init(heap_space.as_mut_ptr(), heap_space.len());
    }
    info!("Initialized heap. Dynamic memory allocation via alloc is now available");

    info!("Finding an SFS to find the kernel binary");
    let mut sfs_dir = match find_kernel_volume() {
        Some(sfs) => sfs,
        None => {
            error!("An unhandled error occurred when trying to open the kernel volume!");
            return Status::LOAD_ERROR;
        }
    };

    info!("Reading kernel binary");
    info!("TEST INFO");
    if sfs_dir.is_directory().unwrap() {
        info!("SFS Dir is a directory");
    }

    let (kd, status) = open_kernel_elf(&mut sfs_dir);
    let kernel_data = match kd {
        Some(data) => {
            info!("Got kernel binary data from open_kernel_elf function");
            data
        },
        None => {
            error!("An unhandled error occurred when reading the kernel binary. Cannot continue.");
            return status;
        },
    };

    let kernel_buffer = kernel_data.get_buffer_slice();
    //let kernel_buffer = &kernel_buffer_slice[..kernel_data.len];
    let entry = match parse_elf_and_load(kernel_buffer) {
        Ok(e) => {
            info!("Read kernel binary. Loading...");
            e
        },
        Err(_) => {
            error!("FATAL FAILED TO PARSE AND LOAD KERNEL BINARY! FAILED TO LOAD KERNEL!");
            return Status::LOAD_ERROR;
        },
    };

    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>().unwrap();
    let mut gop = open_protocol_exclusive::<GraphicsOutput>(gop_handle).unwrap();
    let mode_info = gop.current_mode_info();
    let mut fb = gop.frame_buffer();
    let fb_size = fb.size();

    info!("Framebuffer width: {}", mode_info.resolution().0);

    let fb_info = FramebufferInfo {
        addr: fb.as_mut_ptr(),
        size: fb_size,
        width: mode_info.resolution().0,
        height: mode_info.resolution().1,
        pixels_per_scan_line: mode_info.stride()
    };

    let fb_info_box = Box::new(fb_info);
    let fb_info_raw = Box::into_raw(fb_info_box);

    info!("Booting");
    let entry_fn: extern "sysv64" fn(*mut FramebufferInfo) -> ! = unsafe {
        core::mem::transmute(entry)
    };
    entry_fn(fb_info_raw);
}

fn parse_elf_and_load(data: &[u8]) -> Result<usize, ()> {
    const ELF_MAGIC: &[u8; 4] = b"\x7FELF";
    if &data[0..4] != ELF_MAGIC {
        error!("ELF FILE DOES NOT CONTAIN MAGIC HEADER. FAILED TO LOAD KERNEL");
        return Err(());
    }

    let e_entry = u64::from_le_bytes(data[24..32].try_into().unwrap()) as usize;
    let e_phoff = u64::from_le_bytes(data[32..40].try_into().unwrap()) as usize;
    let e_phentsize = u16::from_le_bytes(data[54..56].try_into().unwrap()) as usize;
    let e_phnum = u16::from_le_bytes(data[56..58].try_into().unwrap()) as usize;

    let mmap = boot::memory_map(MemoryType::LOADER_DATA).expect("Failed to get memory map");
    let mmap_iter = mmap.entries();
    for desc in mmap_iter {
        debug!("  Start: {:#010x}, Pages: {:>5}, Type: {:?}",
        desc.phys_start,
        desc.page_count,
        desc.ty);
    }


    for i in 0..e_phnum {
        let ph_offset = e_phoff + i * e_phentsize;
        let p_type = u32::from_le_bytes(data[ph_offset..ph_offset + 4].try_into().unwrap());
        if p_type != 1 {
            continue;
        }

        let p_offset = u64::from_le_bytes(data[ph_offset + 8..ph_offset + 16].try_into().unwrap()) as usize;
        let p_vaddr = u64::from_le_bytes(data[ph_offset + 16..ph_offset + 24].try_into().unwrap()) as usize;
        let p_filesz = u64::from_le_bytes(data[ph_offset + 32..ph_offset + 40].try_into().unwrap()) as usize;
        let p_memsz = u64::from_le_bytes(data[ph_offset + 40..ph_offset + 48].try_into().unwrap()) as usize;

        let dst = uefi::boot::allocate_pages(
            boot::AllocateType::Address(p_vaddr.try_into().unwrap()), 
            MemoryType::LOADER_DATA, 
            (p_memsz + 0xFFF) / 0x1000).unwrap(); // TODO: Fix cant find pages

        debug!("ELF entry point: {:#x}", e_entry);
        debug!("p_vaddr: {:#x}", p_vaddr);

        unsafe {
            core::ptr::copy_nonoverlapping(data[p_offset..].as_ptr(), dst.as_ptr(), p_filesz);
        }
    }
    Ok(e_entry)
}
