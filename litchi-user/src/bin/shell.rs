#![no_std]
#![no_main]
#![feature(let_else)]
#![feature(bench_black_box)]

extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use anyhow::{anyhow, Error, Result};
use litchi_user::syscall::{sys_halt, sys_open, sys_read, sys_sleep};
use litchi_user::tsc::read_tsc;
use litchi_user::{print, println};
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

fn bench<const SYSCALL: bool>() {
    let start = read_tsc();
    for _ in 0..1000 {
        let mut v = vec![0i64; 65536];
        v[1] = 1;
        for i in 2..(v.len() - 1) {
            v[i] = core::hint::black_box(v[i - 1].wrapping_add(v[i - 2]));
            if SYSCALL && i % 8192 == 0 {
                sys_sleep(0);
            }
        }
        core::hint::black_box(v.last().unwrap());
    }
    let end = read_tsc();
    println!(
        "bench with syscall {:5}: end {} - start {} = {}",
        SYSCALL,
        end,
        start,
        end - start
    );
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
        "halt" => {
            sys_halt();
        }
        "tsc" => {
            println!("tsc: {}", read_tsc());
        }
        "bench" => {
            for _ in 0..10 {
                bench::<true>();
            }
            for _ in 0..10 {
                bench::<false>();
            }
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
