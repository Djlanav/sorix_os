use core::marker::PhantomData;

use alloc::vec::Vec;
use bumpalo::Bump;

pub struct Shell<'a> {
    bump: Bump,
    pub x_offset: usize,
    pub y_offset: usize,
    phantom: PhantomData<&'a u8>
}

impl<'a> Shell<'a> {
    pub fn new() -> Self {
        Self {
            bump: Bump::new(),
            x_offset: 0,
            y_offset: 0,
            phantom: PhantomData
        }
    }

    pub fn write(&'a mut self, src: &'a str) -> &'a mut str {
        let src_bytes = src.as_bytes();
        if src_bytes[src.len()] == b'\n' {
            self.y_offset += 5;
        }

        let string = self.bump.alloc_str(src);
        string
    }
}

fn t() {
    let mut v = Vec::new();
    v.push(1);
    v.push(4);
}