use core::arch::asm;

use crate::kprintln;

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
    _reserved: [u8; 0xA0 - 0x2C], // Pad to port list
    pub ports: [HbaPort; 32], // Max of 32 ports
}

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
    _reserved1: [u32; 11],
    _vendor: [u32; 4],
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
                    let hba = unsafe { &*(mmio as *mut HbaMem) };
                    kprintln!("HBA CAP: {:#x}, GHC: {:#x}, PI (Ports Implemented): {:#x}", hba.cap, hba.ghc, hba.pi);
                }
            }
        }
    }

    if !found {
        kprintln!("ERROR: Could not find an AHCI controller");
    }
}

pub fn read_bar5(bus: u8, device: u8, function: u8) -> u32 {
    let bar5 = pci_read_config(bus, device, function, 0x24);
    let mmio_base = bar5 & 0xFFFFFFF0;

    kprintln!("ACHI MMIO Base Address: {:#010x}", mmio_base);
    mmio_base
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