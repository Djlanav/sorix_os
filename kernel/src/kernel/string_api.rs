use bumpalo::collections;
use bumpalo::Bump;

//use crate::{kprintln, KERNEL_EVENT_MANAGER, kernel::KernelEvent, alloc::string::ToString};

// Terminal offsets
pub static mut X_OFFSET_SHELL: usize = 0;
pub static mut Y_OFFSET_SHELL: usize = 0;
pub static mut ONE_LINE_LENGTH: usize = 0;

type BumpString<'a> = collections::String<'a>;

pub struct Shell {
    bump: Bump,
}

impl Shell {
    pub fn new() -> Self {
        Self {
            bump: Bump::with_capacity(5024),
        }
    }

    pub fn write(&self, src: &str) -> BumpString {
        let string = BumpString::from_str_in(src, &self.bump);
        string
    }
}