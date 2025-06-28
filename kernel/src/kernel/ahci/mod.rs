pub mod cmd_management;

use core::cell::RefCell;
use core::ptr::read_volatile;
use crate::alloc::string::ToString;

use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::kernel::page_heap::{self, allocate_page};
use crate::kprintln;
use crate::kernel::pci::*;

#[derive(Default)]
#[repr(C)]
pub struct HbaMem {
    pub cap: u32,
    pub ghc: u32,
    pub is: u32,
    pub pi: u32, // Ports Implemented
    pub vs: u32,
    pub ccc_ctl: u32,
    pub ccc_pts: u32,
    pub em_loc: u32,
    pub em_ctl: u32,
    pub cap2: u32,
    pub bohc: u32,
    //_reserved: [u8; 0xA0 - 0x2C], // Pad to port list
    pub ports: Vec<Rc<RefCell<HbaPort>>>, // Max of 32 ports
}

#[derive(Default, Copy, Clone)]
#[repr(C)]
pub struct HbaPort {
    pub clb: u32,
    pub clbu: u32,
    pub fb: u32,
    pub fbu: u32,
    pub is: u32,
    pub ie: u32,
    pub cmd: u32,
    pub reserved0: u32,
    pub tfd: u32,
    pub sig: u32,
    pub ssts: u32,
    pub sctl: u32,
    pub serr: u32,
    pub sact: u32,
    pub ci: u32,
    pub sntf: u32,
    pub fbs: u32,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum PortType {
    None,
    Sata,
    Satapi,
    EnclosureMgmtBridge,
    PortMultiplier
}

pub fn read_bar5(bus: u8, device: u8, function: u8) -> u32 {
    let bar5 = pci_read_config(bus, device, function, 0x24);
    let mmio_base = bar5 & 0xFFFFFFF0;

    kprintln!("ACHI MMIO Base Address: {:#010x}", mmio_base);
    mmio_base
}

fn read_hba_mem_volatile(mmio: u32) -> Box<HbaMem> {
    let regs_ptr = mmio as *const u32;
    let mut fields = [0u32; 11];
    for i in 0..10 {
        unsafe {
            fields[i] = read_volatile(regs_ptr.add(i));
        }
    }

    let hba = HbaMem {
        cap: fields[0],
        ghc: fields[1],
        is: fields[2],
        pi: fields[3],
        vs: fields[4],
        ccc_ctl: fields[5],
        ccc_pts: fields[6],
        em_loc: fields[7],
        em_ctl: fields[8],
        cap2: fields[9],
        bohc: fields[10],
        ports: Vec::with_capacity(32)
     };

     let mut hba_box = Box::new(hba);
     let mut n = hba_box.pi;
     let mut count = 0;
     while n > 0 {
        n &= n - 1;
        count += 1;
     }

     for _i in 0..count {
        let port = read_hba_ports_volatile(mmio);
        hba_box.ports.push(port);
     }

     hba_box
}

#[allow(unused_assignments)]
fn read_hba_ports_volatile(mmio: u32) -> Rc<RefCell<HbaPort>>{
    let ports_base = (mmio + 0x100) as *const u32;
    let mut fields = [0u32; 17];

    let mut hex_index = 0x00;
    for i in 0..16 {
        unsafe {
            fields[i] = read_volatile(ports_base.add(hex_index / 4));
        }
        hex_index += 0x04;
    }
    
    let hba_port = HbaPort {
        clb: fields[0],
        clbu: fields[1],
        fb: fields[2],
        fbu: fields[3],
        is: fields[4],
        ie: fields[5],
        cmd: fields[6],
        reserved0: fields[7],
        tfd: fields[8],
        sig: fields[9],
        ssts: fields[10],
        sctl: fields[11],
        serr: fields[12],
        sact: fields[13],
        ci: fields[14],
        sntf: fields[15],
        fbs: fields[16]
    };

    let port_rc = Rc::new(RefCell::new(hba_port));
    port_rc
}

#[allow(unused_assignments)]
pub fn scan_pci_for_ahci() -> Option<Box<HbaMem>> {
    let mut found = false;

    for bus in 0..=255 {
        for device in 0..32 {
            for function in 0..8 {
                let class_info = pci_read_config(bus, device, function, 0x08);
                let class = (class_info >> 24) & 0xFF;
                let subclass = (class_info >> 16) & 0xFF;
                let prog_if = (class_info >> 8) & 0xFF;

                if class == 0x01 && subclass == 0x06 && prog_if == 0x01 {
                    found = true;
                    kprintln!("Found AHCI controller at {}:{}:{}", 0, device, 0);

                    let mmio = read_bar5(bus, device, function);

                    let hba = read_hba_mem_volatile(mmio);
                    kprintln!("HBA CAP: {:#x}, GHC: {:#x}, PI (Ports Implemented): {:#x}", hba.cap, hba.ghc, hba.pi);
                    return Some(hba);
                }
            }
        }
    }

    if !found {
        kprintln!("ERROR: Could not find an AHCI controller");
    }

    None
}

pub fn find_ahci_device(hba: &HbaMem) -> Option<usize> {
    let implemented = hba.pi;

    for i in 0..32 {
        if (implemented >> i) & 1 == 0 {
            continue;
        }

        let port_type = ahci_probe_port_type(&hba, i); // <--- Crash here
        if port_type == PortType::Sata {
            kprintln!("Found SATA drive on port: {}", i);
            return Some(i)
        } else {
            kprintln!("Port {} is not SATA: {:?}", i, port_type);
        }
    }

    kprintln!("Failed to find SATA device!");
    None
}

pub fn ahci_probe_port_type(hba_mem: &HbaMem, index: usize) -> PortType {
    let port_rc = hba_mem.ports[index].clone(); // Crash point here
    let port = port_rc.borrow();

    let ssts = port.ssts;

    let ipm = (ssts >> 8) & 0x0F;
    let det = ssts & 0x0F;

    kprintln!("Index: {}, SSTS: {:#x}, IPM: {:#x}, DET: {:#x}, SIG: {:#x}, CMD: {:#x}",
                      index, ssts, ipm, det, port.sig, port.cmd);

    if det == 0 || ipm == 0 {
        return PortType::None;
    }

    match port.sig {
        0x00000101 => PortType::Sata,
        0xEB140101 => PortType::Satapi,
        0xC33C0101 => PortType::EnclosureMgmtBridge,
        0x96690101 => PortType::PortMultiplier,
        _ => PortType::None,
    }
}

pub fn stop_command_engine(port_rc: Rc<RefCell<HbaPort>>) {
    let mut port = port_rc.borrow_mut();

    port.cmd &= !(1 << 0); // ST
    port.cmd &= !(1 << 4); // FRE

    while (port.cmd & (1 << 15)) != 0 || (port.cmd & (1 << 14)) != 0 {}
    kprintln!("AHCI command engine off");
}

pub fn initialize_port(port_rc: Rc<RefCell<HbaPort>>) {
    let mut port = port_rc.borrow_mut();

    let clb = allocate_page();
    let fb = allocate_page();

    port.clb = clb as u32;
    port.clbu = (clb as u64 >> 32) as u32;

    port.fb = fb as u32;
    port.fbu = (fb as u64 >> 32) as u32;

    page_heap::zero_page(clb, 1024.into());
    page_heap::zero_page(fb, 256.into());
}