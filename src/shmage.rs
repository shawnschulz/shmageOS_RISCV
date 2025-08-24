
// This is our basic shell
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

#[global_allocator]
static mut KERNEL_HEAP_ALLOCATOR: LinearAllocator = LinearAllocator::empty();
static mut KERNEL_HEAP: [u8; 0x20000] = [0; 0x20000];
pub unsafe fn init_kernel_heap() {
    let heap_start = KERNEL_HEAP.as_ptr() as usize;
    let heap_size = KERNEL_HEAP.len();
    KERNEL_HEAP_ALLOCATOR.init(heap_start, heap_size);
}

use crate::println;
use crate::uart::Uart;
// Initializes the process loop and uses arena allocaiton to allocate
// a heap
pub fn shmage_init() -> ! {
    let mut uart_instance = uart::Uart::new(0x1000_0000);
    uart_instance.init();
    unsafe {
        init_kernel_heap();
    }
    shfetch();
    // single character input process loop
    let mut v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    println!("{:?}", v);
    loop {
        // Get the character
        if let Some(c) = uart_instance.get() {
            match c {
                8 => {
                    // 8 is the backspace character, need to replace the
                    // previous character with a ' '
                    print!("{}{}{}", 8 as char, ' ', 8 as char);
                },
                10 | 13 => {
                    // carriage returns
                    println!();
                },
                0x1b => {
                    //ANSI escape sequences
                    if let Some(next_byte) = uart_instance.get() {
                        if next_byte == 91 {
                            if let Some(b) = uart_instance.get() {
                                match b as char {
                                    'A' => {
                                        println!("up arrow press");
                                    },
                                    'B' => {
                                        println!("down arrow press");
                                    },
                                    'C' => {
                                        println!("right arrow press");
                                    },
                                    'D' => {
                                        println!("left arrow press");
                                    },

                                        println!("idk what happened");
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        print!("{}", c as char);
                },
                }
            }
        }
    }
