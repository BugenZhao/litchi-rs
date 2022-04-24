use core::arch::asm;

pub fn read_tsc() -> u64 {
    unsafe {
        let lo: u32;
        let hi: u32;
        asm!("rdtsc",
            out("eax") lo,
            out("edx") hi,
        );
        ((hi as u64) << 32) | (lo as u64)
    }
}
