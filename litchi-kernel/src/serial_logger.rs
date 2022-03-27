use lazy_static::lazy_static;
use log::SetLoggerError;
use spin::Mutex;
use uart_16550::SerialPort;
use x86_64::instructions;

lazy_static! {
    pub static ref DEBUG_SERIAL: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3f8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;

    instructions::interrupts::without_interrupts(|| {
        DEBUG_SERIAL
            .lock()
            .write_fmt(args)
            .expect("printing to debug serial failed")
    })
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::serial_logger::_print(format_args!($($arg)*))
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

struct SerialLogger;

static LOGGER: SerialLogger = SerialLogger;

impl log::Log for SerialLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!(
                "<{:>5}>: {:>12}:{:03}: {}",
                record.metadata().level(),
                record.file().unwrap_or("?"),
                record.line().unwrap_or(0),
                record.args()
            )
        }
    }

    fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)?;
    log::set_max_level(log::LevelFilter::Debug);
    Ok(())
}
