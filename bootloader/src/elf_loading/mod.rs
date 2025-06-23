use alloc::vec::Vec;
use log::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use uefi::boot::PAGE_SIZE;

#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum OSABI {
    SystemV = 0x0,
    HPUX = 0x01,
    NetBSD = 0x02,
    Linux = 0x03,
    GNUHurd = 0x04,
    Solaris = 0x06,
    AIX = 0x07,
    IRIX = 0x08,
    FreeBSD = 0x09,
    Tru64 = 0x0A,
    NovellModesto = 0x0B,
    OpenBSD = 0x0C,
    OpenVMS = 0x0D,
    NonStopKernel = 0x0E,
    AROS = 0x0F,
    FenixOS = 0x10,
    NuxiCloudABI = 0x11,
    OpenVOS = 0x12,
}

#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum Endianness {
    Little = 0x1,
    Big = 0x2,
}

#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ELFClass {
    X32 = 0x1,
    X64 = 0x2,
}

#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum ObjectFileType {
    Unknown = 0x00,
    Relocatable = 0x01,
    Executable = 0x02,
    SharedObject = 0x03,
    CoreFile = 0x04,
}

#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u16)]
pub enum ISA {
    Unknown = 0x00,
    X86 = 0x03,
    ARM = 0x28,
    AMD64 = 0x3E,
}

#[derive(Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum ProgramHeaderType {
    Null = 0x00000000,
    Load = 0x00000001,
    Dynamic = 0x00000002,
    Interp = 0x00000003,
    Note = 0x00000004,
    Reserved = 0x00000005,
    Header = 0x00000006,
    TLS = 0x00000007,
}

pub struct ELFHeader {
    pub elf_identity: ELFIdentity,
    pub program_headers: Vec<ProgramHeader>,
    pub e_type: ObjectFileType,
    pub isa: ISA,
    pub version: u32,
    pub entry_function: usize,
    pub program_header_offset: usize,
    pub flags: u32,
    pub header_size: u16, // Contains the size in bytes of the ELF header (64 bytes for 64-bit and 52 for 32-bit)
    pub phentry_size: u16,
    pub phentry_amount: u16,
}

impl ELFHeader {
    pub fn make(elf_data: &[u8]) -> Option<Self> {
        let identity = match ELFIdentity::make(elf_data) {
            Some(ident) => ident,
            None => {
                error!("Failed to make ELF identity struct");
                return None;
            }
        };

        let (e_type, isa, version) = match Self::make_first_three(elf_data) {
            Some(three) => three,
            None => return None,
        };

        let entry_function = u64::from_le_bytes(elf_data[24..32].try_into().unwrap()) as usize;
        let program_header_offset = u64::from_le_bytes(elf_data[32..40].try_into().unwrap()) as usize;
        let header_size = u16::from_le_bytes(elf_data[52..54].try_into().unwrap());
        let phentry_size = u16::from_le_bytes(elf_data[54..56].try_into().unwrap());
        let phentry_amount = u16::from_le_bytes(elf_data[56..58].try_into().unwrap());

        Some(Self {
            elf_identity: identity,
            program_headers: Vec::new(),
            e_type,
            isa,
            version,
            entry_function,
            program_header_offset,
            flags: 0x0,
            header_size,
            phentry_size,
            phentry_amount
        })
    }

    fn make_first_three(elf_data: &[u8]) -> Option<(ObjectFileType, ISA, u32)> {
        let etype_integer = u16::from_le_bytes(elf_data[16..18].try_into().unwrap());
        let e_type = match ObjectFileType::try_from_primitive(etype_integer) {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to convert elf_data[16..18] to ObjectFileType. Error: {}", e);
                return None;
            }
        };

        let emachine_integer = u16::from_le_bytes(elf_data[18..20].try_into().unwrap());
        let isa = match ISA::try_from_primitive(emachine_integer) {
            Ok(isa) => isa,
            Err(e) => {
                error!("Failed to convert elf_data[18..20] to ISA. Error: {}", e);
                return None;
            }
        };

        let version = u32::from_le_bytes(elf_data[20..24].try_into().unwrap());

        Some((e_type, isa, version))
    }
}

pub struct ELFIdentity {
    pub magic: [u8; 4],
    pub class: ELFClass,
    pub endianness: Endianness,
    pub ei_version: u8,
    pub os_abi: OSABI,
    pub abi_version: u8, // Mostly unused. Can ignore but keep as a field.
}

impl ELFIdentity {
    pub fn make(elf_data: &[u8]) -> Option<Self> {
        let magic_slice = &elf_data[0..4];
        if magic_slice != b"\x7FELF" {
            error!("Not a valid ELF. Provided data does not contain the ELF magic number.");
            return None;
        }

        let mut magic = [0u8; 4];
        let mut index = 0;
        for b in magic_slice {
            magic[index] = *b;
            index += 1;
        }

        let class = match ELFClass::try_from_primitive(elf_data[4]) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to convert elf_data[5] into ELFClass enum. Error: {}", e);
                return None;
            }
        };
        let endianness = match Endianness::try_from_primitive(elf_data[5]) {
            Ok(end) => end,
            Err(e) => {
                error!("Failed to convert elf_data[6] into Endianness enum. Error: {}", e);
                return None;
            }
        };

        let os_abi = match OSABI::try_from_primitive(elf_data[7]) {
            Ok(osabi) => osabi,
            Err(e) => {
                error!("Failed to convert elf_data[8] into OSABI enum. Error: {}", e);
                return None;
            }
        };

        let ei_version = elf_data[7];
        let abi_version = elf_data[9];

        Some(Self {
            magic,
            class,
            endianness,
            os_abi,
            ei_version,
            abi_version
        })
    }
}

pub struct ProgramHeader {
    pub p_type: ProgramHeaderType,
    pub offset: usize,
    pub virtual_addr: usize,
    pub physical_addr: usize,
    pub file_size: usize,
    pub memory_size: usize,
    pub vaddr_end: usize,
    pub page_count: usize,
}

impl ProgramHeader {
    pub fn new(elf_data: &[u8], ph_offset: usize, phentry_size: u16, index: usize) -> Option<Self> {
        let p_type = Self::get_type(elf_data, ph_offset, phentry_size, index);
        if let ProgramHeaderType::Load = p_type {
            let offset = u64::from_le_bytes(elf_data[ph_offset + 8..ph_offset + 16].try_into().unwrap()) as usize;
            let virtual_addr = u64::from_le_bytes(elf_data[ph_offset + 16..ph_offset + 24].try_into().unwrap()) as usize;
            let physical_addr = u64::from_le_bytes(elf_data[ph_offset + 24..ph_offset + 32].try_into().unwrap()) as usize;
            let file_size = u64::from_le_bytes(elf_data[ph_offset + 32..ph_offset + 40].try_into().unwrap()) as usize;
            let memory_size = u64::from_le_bytes(elf_data[ph_offset + 40..ph_offset + 48].try_into().unwrap()) as usize;

            // Alignment
            let virtual_addr = virtual_addr & !(PAGE_SIZE - 1);
            let vaddr_end = (virtual_addr + memory_size + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
            let page_count = (vaddr_end - virtual_addr) / PAGE_SIZE;
            
            return Some(Self {
                p_type,
                offset,
                virtual_addr,
                physical_addr,
                file_size,
                memory_size,
                vaddr_end,
                page_count
            });
        }
;
        None
    }

    pub fn get_type(elf_data: &[u8], ph_offset: usize, phentry_size: u16, index: usize) -> ProgramHeaderType {
        let ph_offset = ph_offset + index * phentry_size as usize;
        let p_type = u32::from_le_bytes(elf_data[ph_offset..ph_offset + 4].try_into().unwrap());
        let p_type = match ProgramHeaderType::try_from_primitive(p_type) {
            Ok(t) => t,
            Err(e) => {
                error!("Failed to convert p_type to ProgramHeaderType enum. Error: {}", e);
                return ProgramHeaderType::Null;
            }
        };

        p_type
    }
}