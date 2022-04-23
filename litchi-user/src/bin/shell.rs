#![no_std]
#![no_main]
#![feature(let_else)]

extern crate alloc;

use alloc::{string::String, vec, vec::Vec};
use anyhow::{anyhow, Error, Result};
use litchi_user::{
    print, println,
    syscall::{sys_open, sys_read, sys_sleep},
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

fn handle<'a>(command: String, mut args: impl Iterator<Item = &'a str>) -> Result<()> {
    let mut next_arg = || args.next().ok_or_else(|| anyhow!("expect argument"));

    match command.as_str() {
        "echo" => {
            let content = args.collect::<Vec<_>>().join(" ");
            println!("{}", content);
        }
        "sleep" => {
            let slice: usize = next_arg()?.parse().map_err(Error::msg)?;
            sys_sleep(slice);
        }
        _ => return Err(anyhow!("unknown command: `{}`", command)),
    }

    Ok(())
}

#[no_mangle]
extern "C" fn main() {
    let mut term = Term::open().unwrap();
    println!("\n\n\nWelcome to the Litchi Shell.");

    loop {
        print!("> ");
        let line = term.read_line().unwrap();
        let line = line.trim();

        let mut tokens = line.split_ascii_whitespace();
        let Some(command) = tokens.next() else {
            continue;
        };

        match handle(command.to_lowercase(), tokens) {
            Ok(_) => {}
            Err(e) => println!("Error: {}", e),
        }
    }
}
