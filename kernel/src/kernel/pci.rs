#![feature(asm)]

use core::arch::asm;

const PCI_CONFIG_SPACE_START: u64 = 0x0000_0000;
const PCI_DEVICE_COUNT: usize = 32 * 8 * 256; // bus * device * function

pub struct PciDeviceHeader {
    vendor_id: u16,
    device_id: u16,
    command: u16,
    status: u16,
    revision_id: u8,
    prog_if: u8,
    subclass: u8,
    class: u8,
    cache_line_size: u8,
    latency_timer: u8,
    header_type: u8,
    bist: u8,
}

pub fn pci_read_config(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address: u32 = 
        (1 << 31) |
        ((bus as u32) >> 16) |
        ((device as u32) >> 11) |
        ((function as u32) >> 8) |
        ((offset as u32) & 0xFC);

    outl(0xCF8, address);
    inl(0xCFC)
}

pub fn scan_pci_for_virtio_block() {
    for device in 0..32 {
        let value = pci_read_config(0, device, 0, 0x0);

        let vendor_id = (value & 0xFFFF) as u16;
        let device_id = ((value >> 16) & 0xFFFF) as u16;

        if vendor_id == 0xFFFF {
            continue;
        }
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