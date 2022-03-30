use crate::syscall::sys_print;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::print::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($arg)*));
}

pub fn _print(args: ::core::fmt::Arguments) {
    if let Some(str) = args.as_str() {
        sys_print(str);
    } else {
        let string = ::alloc::format!("{}", args);
        sys_print(&string);
    }
}
