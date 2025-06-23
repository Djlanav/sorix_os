#![no_main]
#![no_std]

mod dir_management;
mod utils;
mod elf_loading;

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use dir_management::*;

use log::*;
use uefi::{boot::{open_protocol_exclusive, MemoryDescriptor, MemoryType, PAGE_SIZE}, mem::memory_map::MemoryMap, prelude::*, proto::{console::gop::GraphicsOutput, media::file::File}};
use linked_list_allocator::LockedHeap;

use crate::{elf_loading::{ELFHeader, ELFIdentity}, utils::framebuffer::FramebufferInfo};

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

    let fb_slice = unsafe {
        core::slice::from_raw_parts_mut(fb.as_mut_ptr() as *mut u32, fb_size / 4)
    };

    let fb_info = FramebufferInfo {
        buffer: fb_slice,
        size: fb_size,
        width: mode_info.resolution().0,
        height: mode_info.resolution().1,
        pixels_per_scan_line: mode_info.stride()
    };

    let fb_info_box = Box::new(fb_info);
    let fb_info_raw = Box::into_raw(fb_info_box);

    info!("Booting");
    info!("Exiting UEFI Boot Services");
    unsafe {
        _ = boot::exit_boot_services(None);
    }

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

    let elf_header = match ELFHeader::make(data) {
        Some(eh) => eh,
        None => {
            error!("Failed to make ELFHeader. Invalid header possbily provided. Cannot continue. Boot failed.");
            return Err(());
        }
    };

    let e_entry = u64::from_le_bytes(data[24..32].try_into().unwrap()) as usize;
    let e_phoff = u64::from_le_bytes(data[32..40].try_into().unwrap()) as usize;
    let e_phentsize = u16::from_le_bytes(data[54..56].try_into().unwrap()) as usize;
    let e_phnum = u16::from_le_bytes(data[56..58].try_into().unwrap()) as usize;

    let mmap = boot::memory_map(MemoryType::LOADER_DATA).expect("Failed to get memory map");
    let mmap_iter = mmap.entries();
    let mut mmap_vec = Vec::new();
    for m in mmap_iter {
        mmap_vec.push(*m);
    }

    match largest_conventional_region(mmap_vec) {
        Some(d) => {
            info!("Largest conventional memory region: ");
            info!("Start: {:#x},  Pages: {},  Type: {:?}", d.phys_start, d.page_count, d.ty);
        },
        None => todo!(),
    };

    let mut allocated_regions = alloc::collections::BTreeSet::new();

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

        let mem_start = p_vaddr & !(PAGE_SIZE - 1);
        let mem_end = (p_vaddr + p_memsz + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let page_count = (mem_end - mem_start) / PAGE_SIZE;

        let dst = if allocated_regions.contains(&mem_start) {
            info!("Already allocated at {:#x}. Skipping.", mem_start);
            mem_start as *mut u8
        } else {
            let pages = uefi::boot::allocate_pages(
            boot::AllocateType::Address(mem_start.try_into().unwrap()), 
            MemoryType::LOADER_DATA, 
            page_count).expect("UEFI failed to allocate pages"); // TODO: Fix cant find pages
            allocated_regions.insert(mem_start);
            debug!(
                "Segment @ p_vaddr={:#x}, mem_start={:#x}, pages={}, filesz={:#x}, memsz={:#x}",
                p_vaddr, mem_start, page_count, p_filesz, p_memsz
            );

            pages.as_ptr()
        };

        let dst_ptr = unsafe { dst.add(p_vaddr - mem_start) };
        unsafe {
            core::ptr::copy_nonoverlapping(data.as_ptr().add(p_offset), dst_ptr, p_filesz);
        }

        if p_memsz > p_filesz {
            unsafe {
                core::ptr::write_bytes(
                dst_ptr.add(p_filesz),
                0,
                p_memsz - p_filesz,
                );
            }
        }
    }
    Ok(e_entry)
}

fn largest_conventional_region(mmap: Vec<MemoryDescriptor>) -> Option<MemoryDescriptor> {
    mmap.iter()
        .filter(|desc| desc.ty == MemoryType::CONVENTIONAL)
        .max_by_key(|desc| desc.page_count)
        .copied()
}