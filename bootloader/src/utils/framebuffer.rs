#[repr(C)]
pub struct FramebufferInfo<'a> {
    pub buffer: &'a mut [u32],
    pub size: usize,
    pub width: usize,
    pub height: usize,
    pub pixels_per_scan_line: usize,
}