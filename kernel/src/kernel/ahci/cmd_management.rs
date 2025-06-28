use core::cell::RefCell;

use alloc::{rc::Rc, slice, string::{String, ToString}};

use crate::{kernel::{ahci::HbaPort, page_heap::{self, allocate_page, zero_page}}, kserialprint};

#[repr(C, packed)]
pub struct CommandHeader {
    pub flags: u16,
    pub prdt_length: u16,
    pub prdbc: u32,
    pub ctba: u32,
    pub ctbau: u32,
    pub reserved: [u32; 4]
}

#[repr(C, packed)]
pub struct FisRegH2D {
    pub fis_type: u8,
    pub pm_port: u8,
    pub command: u8,
    pub featurel: u8,

    pub lba0: u8,
    pub lba1: u8,
    pub lba2: u8,
    pub device: u8,

    pub lba3: u8,
    pub lba4: u8,
    pub lba5: u8,
    pub featureh: u8,

    pub countl: u8,
    pub counth: u8,
    pub icc: u8,
    pub control: u8,

    pub reserved: [u8; 4]
}

#[repr(C, packed)]
pub struct PhysicalRegionDescriptor {
    pub data_base: u32,
    pub data_base_upper: u32,
    pub reserved: u32,
    pub byte_count: u32,
    pub flags: u32
}

#[repr(C, packed)]
pub struct CommandTable {
    pub command_fis: FisRegH2D,
    pub atapi: [u8; 16],
    pub reserved: [u8; 48],
    prdt_entry: [PhysicalRegionDescriptor; 1]
}

pub fn create_command_header(port_rc: Rc<RefCell<HbaPort>>) {
    let hba_port = port_rc.borrow_mut();
    let slot = 0;

    let ctba = page_heap::allocate_page();
    zero_page(ctba, None);

    let clb = hba_port.clb as *mut CommandHeader;

    let cmdheader = unsafe { &mut *clb.add(slot) };

    cmdheader.flags = ((core::mem::size_of::<FisRegH2D>() / 4) as u16) & 0x1F;
    cmdheader.prdt_length = 1;
    cmdheader.prdbc = 0;
    cmdheader.ctba = ctba as u32; // Command Table
    cmdheader.ctbau = (ctba as u64 >> 32) as u32; // CT Upper
}

pub fn setup_command_table(port_rc: Rc<RefCell<HbaPort>>) {
    let hba_port = port_rc.borrow_mut();
    let cmdheader = unsafe { &mut *(hba_port.clb as *mut CommandHeader) };

    let ctba = unsafe { &mut *(cmdheader.ctba as *mut CommandTable) };
    ctba.command_fis = FisRegH2D {
        fis_type: 0x27,
        pm_port: 1 << 7,
        command: 0x25,
        featurel: 0,
        lba0: 0,
        lba1: 0,
        lba2: 0,
        device: 1 << 6,

        lba3: 0,
        lba4: 0,
        lba5: 0,
        featureh: 0,

        countl: 1,
        counth: 0,
        icc: 0,
        control: 0,
        reserved: [0; 4],
    };
    //kserialprint!("Setup command FIS with command: {:#x}", ctba.command_fis.command);
}

pub fn create_prdt_entry(port_rc: Rc<RefCell<HbaPort>>) {
    let hbaport = port_rc.borrow_mut();
    let cmdheader = unsafe { &mut *(hbaport.clb as *mut CommandHeader) };
    let ctba = unsafe { &mut *(cmdheader.ctba as *mut CommandTable) };

    let data_buffer = allocate_page();
    page_heap::zero_page(data_buffer, None);

    ctba.prdt_entry[0] = PhysicalRegionDescriptor {
        data_base: data_buffer as u32,
        data_base_upper: (data_buffer as u64 >> 32) as u32,
        reserved: 0,
        byte_count: (512 - 1), // One Sector
        flags: ((512 - 1) & 0x3FFFFF) | (1 << 31)
    };

    cmdheader.prdt_length = 1;
    cmdheader.prdbc = 0;
}

pub fn issue_command(port_rc: Rc<RefCell<HbaPort>>) {
    let mut hbaport = port_rc.borrow_mut();
    let cmdheader_ptr = hbaport.clb as *const CommandHeader;
    let cmdheader = unsafe { &*(hbaport.clb as *const CommandHeader) };

    let cmdtable_ptr = cmdheader.ctba as *const CommandTable;
    let cmdtable = unsafe { &*(cmdtable_ptr) };

    let prdt_entry = &cmdtable.prdt_entry[0];
    let dbc = prdt_entry.byte_count;
    let i = prdt_entry.flags;

    kserialprint!("dbc: {}", dbc); // should be 511
    kserialprint!("i: {:#x}", i);     // should be 1

    // Start Command Engine
    hbaport.cmd &= !(1 << 0); // Start
    while hbaport.cmd & (1 << 15) != 0 {}
    kserialprint!(String::from("CR Cleared"));

    // Now enable FIS Receive (FRE)
    hbaport.cmd |= 1 << 4;
    hbaport.cmd |= 1 << 0;
    hbaport.ci = 1 << 0;

    kserialprint!("Waiting for Command To Finish");

    let mut success = false;
    let mut timeout = 900_000_000;
    // Wait for finish
    while hbaport.ci & (1 << 0) != 0 && timeout != 0 {
        if timeout == 0 {
            break;
        }

        timeout -= 1;
    }
    kserialprint!("TDF Raw = {:#010b}", hbaport.tfd);

    if timeout > 0 {
        success = true;
    }

    // Also check for error
    if !success {
        kserialprint!("Command Timeout!");
    } else if hbaport.tfd & 0x88 != 0 {
        kserialprint!("AHCI Error: Task File Data = {:#x}", hbaport.tfd);
    } else {
        kserialprint!("Read completed successfully!");
    }
}

pub fn read_data_buffer(port_rc: Rc<RefCell<HbaPort>>) {
    let hbaport = port_rc.borrow_mut();
    let cmdheader = unsafe { &mut *(hbaport.clb as *mut CommandHeader) };
    let ctba = unsafe { &mut *(cmdheader.ctba as *mut CommandTable) };

    let data_buffer = ctba.prdt_entry[0].data_base as *mut u8;
    let buf = unsafe {
        slice::from_raw_parts(data_buffer as *const u8, 512)
    };


    for i in 0..16 {
        kserialprint!("Data at index {}: {:#x}", i, buf[i]);
    }
}

pub fn check_integrity(port_rc: Rc<RefCell<HbaPort>>) {
    let hba_port = port_rc.borrow();
    let cmd_header = unsafe { &mut *(hba_port.clb as *mut CommandHeader) };
    let cmd_table = unsafe { &mut *(cmd_header.ctba as *mut CommandTable) };

    let flags = cmd_header.flags;
    let prdt_length = cmd_header.prdt_length;
    let ctbau = cmd_header.ctbau;
    let ctba = cmd_header.ctba;

    // Check Command Header
    kserialprint!("Command Header Status:");
    kserialprint!("  FIS length: {:#x}", flags); // Should be 5
    kserialprint!("  PRDT Length: {}", prdt_length); // Should be 1
    kserialprint!("  CTBA: {:#x}{:08x}", ctbau, ctba); // 64-bit address

    let fis_ptr = &cmd_table.command_fis as *const FisRegH2D as *const u8;
    for i in 0..20 {
        let byte = unsafe { *fis_ptr.add(i) };
        kserialprint!("FIS[{:02}] = {:#04x}  ", i, byte);
    }

    let prdt = &cmd_table.prdt_entry[0];
    let dbau = prdt.data_base_upper;
    let dba = prdt.data_base;
    let byte_count = prdt.byte_count;
    let flags = prdt.flags;

    let dbau64 = ((dbau as u64) << 32) | dba as u64;

    kserialprint!("PRDT Entry:");
    kserialprint!("  DBA: {:#018x}", dbau64);
    kserialprint!("  Byte Count: {} ", byte_count + 1);
    kserialprint!("  Flags (I/O bit): {:#x} ", flags);
    kserialprint!(" PRDT IOC Flag Set: {}\n", (flags & (1 << 31)) != 0);

    let ssts = hba_port.ssts;
    let det = ssts & 0xF;
    kserialprint!("Port State:");
    kserialprint!("  CMD: {:#x}", hba_port.cmd);
    kserialprint!("  CI: {:#x}", hba_port.ci);
    kserialprint!("  IS: {:#x}", hba_port.is);
    kserialprint!("  TFD: {:#010b}", hba_port.tfd);
    kserialprint!("  DET: {:#x}", det);

}