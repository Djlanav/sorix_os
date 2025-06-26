use core::{arch::asm, mem::offset_of};
use core::ptr::read_volatile;

use alloc::{boxed::Box, vec::Vec};

use crate::drawing::fonts::{draw_string_raw, PsfFont};
use crate::drawing::Color;
use crate::kernel::Kernel;
use crate::kprintln;

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
    pub ports: Vec<Box<HbaPort>>, // Max of 32 ports
}

#[derive(Default)]
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

fn pci_read_config(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address: u32 = 
        (1 << 31) |
        ((bus as u32) << 16) |
        ((device as u32) << 11) |
        ((function as u32) << 8) |
        ((offset as u32) & 0xFC);

    outl(0xCF8, address);
    inl(0xCFC)
}

pub fn scan_pci_devices() {
    kprintln!("Scanning PCI Devices");
    for device in 0..32 {
        let value = pci_read_config(0, device, 0, 0x0);

        let vendor_id = (value & 0xFFFF) as u16;
        let device_id = ((value >> 16) & 0xFFFF) as u16;

        if vendor_id == 0xFFFF {
            continue;
        }

        kprintln!(
            "PCI Device at 0:{}.0 -> Vendor ID: {:04x}, Device ID: {:04x}",
            device,
            vendor_id,
            device_id
        );
    }
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
fn read_hba_ports_volatile(mmio: u32) -> Box<HbaPort> {
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

    let port_box = Box::new(hba_port);
    port_box
}

pub fn read_hba() {
    let mmio = read_bar5(0, 13, 0);
    let mem = read_hba_mem_volatile(mmio);
    let ports = read_hba_ports_volatile(mmio);

    kprintln!("CAP: {:#x}, GHC: {:#x}, PI: {:#x}", mem.cap, mem.ghc, mem.pi);
    kprintln!("Port 0: SSTS: {:#x}, SIG: {:#x}, CMD: {:#x}", ports.ssts, ports.sig, ports.cmd);
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

pub fn scan_pci_for_ahci() {
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

                    find_ahci_device(&hba); // <--- This function causes a crash
                }
            }
        }
    }

    if !found {
        kprintln!("ERROR: Could not find an AHCI controller");
    }
}

pub fn ahci_probe_port_type(hba_mem: &HbaMem, index: usize) -> PortType {
    let port = &hba_mem.ports[index]; // Crash point here

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

#[inline]
pub fn outl(port: u16, value: u32) {
    unsafe {
        asm!(
            "out dx, eax",
            in("dx") port,
            in("eax") value,
            options(nomem, nostack, preserves_flags),
        );
    }
}
#[allow(unused_assignments)]
#[inline]
pub fn inl(port: u16) -> u32 {
    let mut ret = 0u32;
    unsafe {
        asm!(
            "in eax, dx",
            in("dx") port,
            out("eax") ret,
            options(nomem, nostack, preserves_flags),
        );
    }
    ret
}