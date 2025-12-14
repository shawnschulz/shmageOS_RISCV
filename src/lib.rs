#![no_std]

pub mod uart;
pub mod page;
pub mod linear_allocator;
pub mod shmage;
pub mod malloc;
// pub mod test;

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        // it's macro magic, but basically the stuff in a print will
        // get put into a write! call in the Uart's write method
        let _ = write!(crate::uart::Uart::new(0xD4017000), $($args)+);
    });
}
#[macro_export]
macro_rules! println {
    () => ({
        // Need to change this to the correct 16550 endline byte char
        // i think this is 0x0A
        // print!("\n")
		print!("\r\n")
	});
	($fmt:expr) => ({
		print!(concat!($fmt, "\r\n"))
	});
	($fmt:expr, $($args:tt)+) => ({
		print!(concat!($fmt, "\r\n"), $($args)+)
	});
}

// replacing the eh_personality C function name
#[unsafe(no_mangle)]
pub extern "C" fn eh_personality() {}

#[panic_handler]
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("[ERROR] program paniced | stack trace:");
    if let Some(p) = info.location() {
        println!("line {}, file {}: {}", p.line(), p.file(), info.message());
    }
    else {
        println!("Failed to find information about panic!")
    }
    abort();
}

// Given an mmio address and offest, write the 8 bit value at that address(input)
pub fn mmio_write(address: usize, offset: usize, value: u8) {
    let reg = address as *mut u8;
    unsafe {
    reg.add(offset).write_volatile(value);
    }
}

// Given an mmio address and offest, get the 8 bit value at that address (output)
pub fn mmio_read(address: usize, offset: usize) -> u8 {
    let reg = address as *mut u8;
    unsafe {
    return reg.add(offset).read_volatile()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn abort() -> !{
    loop {
        // process waits for some interrupt indefinitely on abort
        use core::arch::asm;
        unsafe {asm!("wfi")};
    }
}

// In order to enter supervisor mode, we must map virtual memory addresses
// to physical memory addresses rather than use the physical addresses as we've been doing
// in machine mode. This function maps a range of virtual page table addresses to physical addresses
pub fn map_range_of_addresses(root: &mut page::PageTable, start: usize, end: usize, bits: i64) {
    let mut memory_address =  start & !(page::PAGE_SIZE - 1);
    // This is named "kilobytes" because its the number of 4012 KB pages
    // this was intended to allow for the addition of huge page sizes (GiB and MiB)
    // later in the tutorial
    let num_pages_kilobits = (page::align_value(end, 12) - memory_address) / page::PAGE_SIZE;
    for _ in 0..num_pages_kilobits {
        page::map(root, memory_address, memory_address, bits, 0);
        memory_address += 1 << 12;
    }
}

pub fn shfetch() {
    println!("Welcome to shmageOS!");
    println!("           _              user@wip");
    println!("        ___               ------------------------------");
    println!("      l..l.l              OS: shmageOS 0.0.1 RISCV");
    println!("    __________            Host: QEMU");
    println!("  ______________          Kernel: 0.0.1");
    println!("_____________________     Cluster Connections:");
    println!("ooooooooooooooooooooooo   Network:");
    println!("   |  =    =  |           CPU:");
    println!("   j  O    O  j           GPU:");
    println!(r"   \          /           Mem:");
    println!("                          ------------------------------");
    println!("");
    println!("_______________________");
    println!("\"Writing a computer program is simple,");
    println!("but writing a simple computer program");
    println!("is the hardest thing there is!\" - Shawn");
    println!("_______________________");
    println!("");
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    shmage::shmage_init();
}

// Really would like this to be run in a test harness, but its been a nightmare to get the custom unit testing framework
// to work and i want to at least have some test coverage now
pub fn test_pages() {
    println!("running test test_pages:");
    page::print_page_allocations();
    assert!(1 == 1);
    println!("[ok]")
;
}

pub fn test_alloc() {
    println!("initializing...");
    page::init();
    println!("[ok]");
    println!("allocating some pages");
    let page = page::zalloc(1);
    let mut alloc_start: *mut u8;
    alloc_start = page::alloc(1);
    for i in (1..32704).step_by(1000) {
        alloc_start = page::alloc(1000);
    }
    page::print_page_allocations();
    println!("[ok]");
    println!("deallocating all the pages (i think)");
    page::deallocate_all_pages();
    page::print_page_allocations();
    println!("[ok]");
;
}

/// Eventually want to randomly generate some keyboard inputs and
/// see if the uart console can handle the inputs properly
pub fn test_fuzzed_uart_inputs() {}

pub fn test() {
    test_pages();
    test_alloc();
    println!("tests succeeded!")
}
