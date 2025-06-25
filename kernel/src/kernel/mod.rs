pub mod string_api;
pub mod prelude;
pub mod pci;

use alloc::vec::Vec;
use alloc::{boxed::Box, string::String};
use alloc::collections::BTreeMap;
use crate::{drawing::{fonts::{draw_string, PsfFont}, Color}, kernel::string_api::{Shell, Y_OFFSET_SHELL}, FramebufferInfo, MAIN_FONT};

#[allow(dead_code)]
pub enum EventType {
    PrintLine(String),
}

pub struct KernelEvent {
    event_type: EventType,
    fired: bool,
}

pub struct EventManager {
    events: Vec<KernelEvent>
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            events: Vec::new()
        }
    }

    pub fn run(&mut self, kernel: &mut Kernel) {
        for event in &self.events {
            if !event.fired {
                match &event.event_type {
                    EventType::PrintLine(s) => kernel.println(s.as_str()),
                }
            }
        }
    }

    pub fn clean_events(&mut self) {
        self.events.clear();
    }

    #[allow(dead_code)]
    pub fn new_event(&mut self, event: EventType) {
        let kernel_event = KernelEvent {
            event_type: event,
            fired: false,
        };
        self.events.push(kernel_event);
    }
}

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
    }

    #[allow(dead_code)]
    pub fn print<'b>(&mut self, src: &'b str) {
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
        }

        let src_bytes = src.as_bytes();
        if src_bytes[src_bytes.len() - 1] == b'\n' {
            unsafe {
                Y_OFFSET_SHELL += 25;
            }
        }
    }
}