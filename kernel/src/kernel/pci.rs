use core::arch::asm;
use crate::alloc::string::ToString;

use crate::kprintln;

pub fn pci_read_config(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
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