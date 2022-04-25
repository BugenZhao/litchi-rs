use x86_64::instructions::hlt;
use x86_64::instructions::port::Port;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit(code: ExitCode) -> ! {
    let mut port = Port::new(0xf4);
    unsafe {
        port.write(code as u32);
    }

    // In case the `isa-debug-exit` device is not enabled.
    loop {
        hlt();
    }
}
