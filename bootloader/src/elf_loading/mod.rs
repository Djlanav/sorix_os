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

#[repr(u8)]
pub enum Endiannes {
    Little,
    Big,
}

#[repr(u16)]
pub enum ObjectFileType {
    Unknown = 0x00,
    Relocatable = 0x01,
    Executable = 0x02,
    SharedObject = 0x03,
    CoreFile = 0x04,
}

#[repr(u16)]
pub enum ISA {
    Unknown = 0x00,
    X86 = 0x03,
    ARM = 0x28,
    AMD64 = 0x3E,
}

pub enum ProgramHeaderType {
    
}

pub struct ELFHeader {
    pub elf_identity: ELFIdentity,
    pub isa: ISA,
    pub version: u32,
    pub entry_function: u64,
    pub program_header_offset: u64,
    pub section_header_offset: u64,
    pub flags: u32,
    pub header_size: u16, // Contains the size in bytes of the ELF header (64 bytes for 64-bit and 52 for 32-bit)
    pub phentry_size: u16,
    pub phentry_amount: u16,
    pub shentry_size: u16,
    pub shentry_amount: u16,
}

pub struct ELFIdentity {
    pub magic: [u8; 4],
    pub class: u8,
    pub endiannes: Endiannes,
    pub ei_version: u8,
    pub  os_abi: OSABI,
    pub abi_version: u8, // Mostly unused. Can ignore but keep as a field.
}

pub struct ProgramHeaderTable {
    p_type: u32,

}