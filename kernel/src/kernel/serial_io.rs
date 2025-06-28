use core::arch::asm;

pub fn serial_init() {
    unsafe {
        outb(0x3F8 + 1, 0x00); // Disable all interrupts
        outb(0x3F8 + 3, 0x80); // Enable DLAB (set baud rate divisor)
        outb(0x3F8 + 0, 0x03); // Set divisor to 3 (lo byte) 38400 baud
        outb(0x3F8 + 1, 0x00); //                  (hi byte)
        outb(0x3F8 + 3, 0x03); // 8 bits, no parity, one stop bit
        outb(0x3F8 + 2, 0xC7); // Enable FIFO, clear them, with 14-byte threshold
        outb(0x3F8 + 4, 0x0B); // IRQs enabled, RTS/DSR set
    }
}

pub fn serial_write_byte(byte: u8) {
    while (inb(0x3F8 + 5) & 0x20) == 0 {}
    unsafe {
        outb(0x3F8, byte);
    }
}

pub fn serial_write_str(s: &str) {
    for b in s.bytes() {
        serial_write_byte(b);
    }
}


pub unsafe fn outb(port: u16, val: u8) {
    unsafe {
        asm!("out dx, al", in("dx") port, in("al") val);
    }
}

pub fn inb(port: u16) -> u8 {
    let ret: u8;
    unsafe {
        asm!("in al, dx", out("al") ret, in("dx") port);
    }
    ret
}