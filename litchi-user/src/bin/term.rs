#![no_std]
#![no_main]

extern crate alloc;

use alloc::{string::String, vec, vec::Vec};
use anyhow::{anyhow, Error, Result};
use litchi_user::{
    println,
    syscall::{sys_open, sys_read},
};
use litchi_user_common::resource::ResourceHandle;

struct Term {
    handle: ResourceHandle,

    buf: Vec<u8>,

    cursor: usize,
}

impl Term {
    fn open() -> Result<Self> {
        Ok(Self {
            handle: sys_open("/device/term").map_err(Error::msg)?,
            buf: Vec::new(),
            cursor: 0,
        })
    }

    fn read_line(&mut self) -> Result<String> {
        let mut read = Vec::new();

        loop {
            if self.cursor == self.buf.len() {
                let mut buf = vec![0u8; 256];
                let len = sys_read(self.handle, &mut buf).map_err(Error::msg)?;
                if len == 0 {
                    return Err(anyhow!("term eof"));
                }
                buf.resize(len, 0);
                self.buf = buf;
                self.cursor = 0;
            }

            let byte = self.buf[self.cursor];
            read.push(byte);
            self.cursor += 1;

            if byte == b'\n' {
                let s = String::from_utf8_lossy(&read).into_owned();
                return Ok(s);
            }
        }
    }
}

#[no_mangle]
extern "C" fn main() {
    let mut term = Term::open().unwrap();

    loop {
        let line = term.read_line().unwrap();
        println!("term received: \"{}\"", line);
    }
}
