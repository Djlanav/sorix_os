use core::ptr::NonNull;

use log::*;
use uefi::{boot::{self, MemoryType}, proto::media::{file::{Directory, File, FileAttribute, FileInfo, FileMode, FileSystemVolumeLabel}, fs::SimpleFileSystem}, CStr16, Identify, Status};

pub struct KernelElfData {
    pub _len: usize,
    pub binary_size: u64,
    pub buffer: NonNull<u8>
}

impl KernelElfData {
    pub fn get_buffer_slice(&self) -> &[u8] {
        unsafe {
            return core::slice::from_raw_parts(self.buffer.as_ptr(), self.binary_size as usize);
        }
    }
}

pub fn find_kernel_volume() -> Option<Directory> {
    let handle_buffer = match boot::locate_handle_buffer(boot::SearchType::ByProtocol(&SimpleFileSystem::GUID)) {
        Ok(buffer) => buffer,
        Err(err) => {
            error!("Could not locate an SFS handle! Error: {}", err);
            return None;
        },
    };
    let handle_iter = handle_buffer.iter();
    let os_label = widestring::u16cstr!("OS").as_slice_with_nul();
    for handle in handle_iter {
        let mut sfs = match boot::open_protocol_exclusive::<SimpleFileSystem>(*handle) {
            Ok(sfs) => {
                info!("Successfully opened an SFS");
                sfs
            },
            Err(e) => {
                error!("Failed to open SFS protocol! Error: {}", e);
                return None;
            }
        };
        let mut sfs_volume = match sfs.open_volume() {
            Ok(v) => {
                info!("Successfully opened the SFS volume");
                v
            },
            Err(e) => {
                error!("Failed to open SFS volume! Error: {}", e);
                return None;
            }
        };

        let mut volume_buffer = [0u8; 512];
        let volume_info = match sfs_volume.get_info::<FileSystemVolumeLabel>(&mut volume_buffer) {
            Ok(vi) => {
                info!("Successfully got volume info");
                vi
            },
            Err(e) => {
                error!("Failed to get SFS Volume Info! Error: {}", e);
                return None;
            }
        };

        let volume_label = volume_info.volume_label();
        let os_label = match CStr16::from_u16_with_nul(&os_label) {
            Ok(label) => label,
            Err(e) => {
                error!("Failed to convert OS volume label to CStr16. Error: {}", e);
                return None;
            }
        };
        info!("VOLUME LABEL: {}", volume_label);
        if volume_label.eq(os_label) {
            return Some(sfs_volume);
        } else {
            warn!("Volume is not an OS volume or does not contain a kernel binary. Trying again.");
            continue;
        }
    }

    None
}

pub fn open_kernel_elf(dir: &mut Directory) -> (Option<KernelElfData>, Status) {
    info!("Retrieving kernel binary filename");
    let mut filename_buf = [0u16; 7];

    let filename = match CStr16::from_str_with_buf("kernel", &mut filename_buf) {
        Ok(filename) => {
            info!("Got kernel filename");
            filename
        },
        Err(err) => {
            error!("FAILED TO MAKE UTF-16 STRING FOR KERNEL BINARY FILENAME. ERROR: {}", err);
            return (None, Status::ABORTED);
        },
    };
    let mut kernel_file = match dir.open(filename, FileMode::Read, FileAttribute::empty()) {
        Ok(kf) => match kf.into_regular_file() {
            Some(rf) => {
                info!("Opened kernel binary for proper reading");
                rf
            },
            None => {
                error!("File is not a regular file! Cannot load kernel. No kernel binary.");
                return (None, Status::LOAD_ERROR);
            }
        },
        Err(err) => {
            error!("FAILED TO OPEN KERNEL BINARY. ERROR: {}", err);
            return (None, Status::LOAD_ERROR);
        },
    };

    let mut kfi_buffer = [0u8; 512];
    let kernel_info = match kernel_file.get_info::<FileInfo>(&mut kfi_buffer) {
        Ok(ifo) => ifo,
        Err(e) => {
            error!("Failed to get kernel binary info! Error: {}", e);
            return (None, Status::LOAD_ERROR);
        },
    };
    info!("Kernel binary size: {}", kernel_info.file_size());

    let memory_pool = match boot::allocate_pool(MemoryType::LOADER_DATA, kernel_info.file_size() as usize) {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to allocate memory for kernel buffer! Error: {}", e);
            return (None, Status::LOAD_ERROR);
        }
    };
    let mut kernel_buffer = unsafe {
        core::slice::from_raw_parts_mut(memory_pool.as_ptr(), kernel_info.file_size() as usize)
    };
    info!("Made kernel buffer");

    let buffer_length = match kernel_file.read(&mut kernel_buffer) {
        Ok(len) => {
            info!("Successfully read kernel binary. Length: {}", len);
            len
        },
        Err(err) => {
            error!("FAILED TO READ KERNEL BINARY DATA INTO BUFFER. FAILED TO LOAD KERNEL!!! ERROR: {}", err);
            return (None, Status::LOAD_ERROR);
        }
    };

    info!("Making KernelElfData structure");
    let buffer_nonnull = NonNull::new(kernel_buffer.as_mut_ptr()).expect("FAILED TO MAKE NONNULL");

    let kernel_data = KernelElfData {
        _len: buffer_length,
        binary_size: kernel_info.file_size(),
        buffer: buffer_nonnull
    };

    info!("Continuing boot process");
    (Some(kernel_data), Status::SUCCESS)
}