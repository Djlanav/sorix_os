pub mod string_api;
pub mod pci;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use crate::{drawing::{fonts::{draw_string, PsfFont}, Color}, kernel::string_api::{Shell, Y_OFFSET_SHELL}, FramebufferInfo, MAIN_FONT};

pub struct Kernel<'a> {
    pub shell: Shell,
    pub fonts: BTreeMap<&'a str, PsfFont<'a>>,
    pub framebuffer: Box<FramebufferInfo<'a>>,
}

impl<'a> Kernel<'a> {
    pub fn start(framebuffer: Box<FramebufferInfo<'a>>, shell: Shell) -> Self {
        let psf_header = PsfFont::from_bytes(MAIN_FONT).unwrap();
        let mut fonts = BTreeMap::new();
        fonts.insert("main font", psf_header);

        Self {
            framebuffer,
            fonts,
            shell,
        }
    }

    pub fn println<'b>(&mut self, src: &'b str) {
        let font = match self.fonts.get("main font") {
            Some(f) => f,
            None => {
                self.fill_screen(Color::Red);
                return;
            }
        };

        let text = self.shell.write(src);
        let text_str = text.as_str();

        unsafe {
            draw_string(&mut self.framebuffer, font, text_str, 0, Y_OFFSET_SHELL, Color::White);
            Y_OFFSET_SHELL += 18;
        }

        // let src_bytes = src.as_bytes();
        // if src_bytes[src_bytes.len() - 1] == b'\n' {
        //     unsafe {
        //         Y_OFFSET_SHELL += 25;
        //     }
        // }
    }
}