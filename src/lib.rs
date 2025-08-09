#![no_std]
#![feature(panic_info_message)]

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({});
}
#[macro_export]
macro_rules! println {
    ($($args:tt)+) => ({});
}

// replacing the eh_personality C function name
#[unsafe(no_mangle)]
pub extern "C" fn eh_personality() {}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("[ERROR] program paniced | stack trace:");
    if let Some(p) = info.location() {
        println!("line {}, file {}: {}", p.line(), p.file(), info.message().unwrap());
    }
    else {
        println!("Failed to find information about panic!")
    }
    abort();
}

#[unsafe(no_mangle)]
pub extern "C" fn abort() -> !{
    loop {
        // process waits for some interrupt indefinitely on abort
        use core::arch::asm;
        unsafe {asm!("wfi")};
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {}