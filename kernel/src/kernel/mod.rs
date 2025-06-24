pub mod string_api;
pub mod pci;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;

use crate::{drawing::{fonts::{draw_string, PsfFont}, Color}, kernel::string_api::Shell, FramebufferInfo, MAIN_FONT};

pub struct Kernel<'a> {
    pub shell: Shell<'a>,
    pub fonts: BTreeMap<&'a str, PsfFont<'a>>,
    pub framebuffer: Box<FramebufferInfo<'a>>,
}

impl<'a> Kernel<'a> {
    pub fn start(framebuffer: Box<FramebufferInfo<'a>>, shell: Shell<'a>) -> Self {
        let psf_header = PsfFont::from_bytes(MAIN_FONT).unwrap();
        let mut fonts = BTreeMap::new();
        fonts.insert("main font", psf_header);

        Self {
            framebuffer,
            fonts,
            shell,
        }
    }

    pub fn println(&'a mut self, src: &'a str) {
        let y = self.shell.y_offset;
        let text = self.shell.write(src);
        let font = self.fonts.get("main font").unwrap();

        draw_string(&mut self.framebuffer, font, text, 0, y, Color::White);
    }
}