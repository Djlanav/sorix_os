#[repr(C)]
pub struct FramebufferInfo {
    pub addr: *mut u8,
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub pixels_per_scan_line: usize,
}