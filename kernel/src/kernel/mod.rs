use alloc::boxed::Box;

use crate::FramebufferInfo;

pub struct Kernel<'a> {
    pub framebuffer: Box<FramebufferInfo<'a>>
}

impl<'a> Kernel<'a> {
    pub fn start(framebuffer: Box<FramebufferInfo<'a>>) -> Self {
        Self {
            framebuffer,
        }
    }
}