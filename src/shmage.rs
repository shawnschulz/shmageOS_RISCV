use core::arch::asm;

// This is our basic shell
pub fn shfetch() {
    println!("Welcome to shmageOS!");
    println!("         _                user@wip");
    println!("        ___               ------------------------------");
    println!("      l..l.l              OS: shmageOS 0.0.1 RISCV");
    println!("    __________            Host: Orangepi RV2");
    println!("  ______________          Kernel: 0.0.1");
    println!("_____________________     Cluster Connections:");
    println!("ooooooooooooooooooooooo   Network:");
    println!("   |  =    =  |           CPU: KY_X1 8 cores @ 1.6 GHZ");
    println!("   j  O    O  j           GPU:");
    println!(r"   \          /           Mem:");
    println!("                          ------------------------------");
    println!("_______________________");
    println!("\"Writing a computer program is simple,");
    println!("but writing a simple computer program");
    println!("is the hardest thing there is!\" - Shawn");
    println!("_______________________");
}

use crate::page;
use crate::malloc;

// Remember the page tables are just an abstraction, pages need to be
// mapped properly onto real physical memory locations. This function but
// initializes a page table for the kernel to use as heap virtual memory by
// peforming that mapping and also makes pages for the stack, heap BSS
// DATA etc. Some of these will only be read from but its important all of them
// are known to the kernel's virtual memory allocation system so that we don't do something
// dumb like allocate parts of the BSS or kernel memory to a process
pub fn initialize_kernel_memory() {
    page::init();
    malloc::init();
    println!("[INFO] Printing page allocations before initial allocations:");
    page::print_page_allocations();
    malloc::print_kernel_memory_table();
    println!("[INFO] Printing page allocations after initial allocations:");
    // use the page::map_range function to map all necessary tables for kernel
    let kernel_root = malloc::get_page_table();
    let root_u = kernel_root as usize;
    let mut root = unsafe { kenrel_root.as_mut().unwrap() };
    let kernel_heap_head = malloc::get_head() as usize;
    let total_pages = malloc::get_number_allocations();
    unsafe {
        println!("TEXT:   0x{:x} -> 0x{:x}", TEXT_START, TEXT_END);
        println!("RODATA: 0x{:x} -> 0x{:x}", RODATA_START, RODATA_END);
        println!("DATA:   0x{:x} -> 0x{:x}", DATA_START, DATA_END);
        println!("BSS:    0x{:x} -> 0x{:x}", BSS_START, BSS_END);
        println!("KERNEL STACK:  0x{:x} -> 0x{:x}", KERNEL_STACK_START, KERNEL_STACK_END);
        println!("KERNEL HEAP:   0x{:x} -> 0x{:x}", kernel_heap_head, kernel_heap_head + total_pages * 4096);
    }
    // this should map the kernel's heap
    page::map_range(&mut root, kernel_heap_head, kernel_heap_head + total_pages * 4096, page::PageTableEntryBits::ReadWrite.val());
    // before mapping all the other stuff let's check this worked. we could map the mmio allocated memory now, but let's wait until
    // we set up a filesystem so we can use a .dtb file to this and have a better interface for block drivers too
}

pub fn ptable() {
    page::init();
    page::print_page_allocations();
}
pub fn pkmemtable() {
    initialize_kernel_memory();
    malloc::print_kernel_memory_table();
}
pub fn clear() {
    for i in 0..200 {
        println!();
    }
}

use crate::test;
/// These use stack allocated memory to check for commands, but this is very limited!
/// shows why we have a need for heap allocated memory
/// obviously, we could just allocate from one global heap (arena allocation),
/// although this would still require letting rust know about the heap
/// we could also make a macro for this, since the length of the command name is actually known at compile time, but
/// we will want heap allocation anyways
pub fn basic_command_process (input_array: &[char; 8]) {
    let shfetch_arr: [char; 7] = ['s', 'h', 'f', 'e', 't', 'c', 'h'];
    let mut shfetch_command: bool = true;
    for i in 0..6 {
        if input_array[i] != shfetch_arr[i] {
            shfetch_command = false;
        }
    }
    if shfetch_command {
        shfetch();
    }

    let ptable_arr: [char; 6] = ['p', 't', 'a', 'b', 'l', 'e'];
    let mut ptable_command: bool = true;
    for i in 0..5 {
        if input_array[i] != ptable_arr[i] {
            ptable_command = false;
        }
    }
    if ptable_command {
        ptable();
    }

    let clear_arr: [char; 5] = ['c', 'l', 'e', 'a', 'r'];
    let mut clear_command: bool = true;
    for i in 0..4 {
        if input_array[i] != clear_arr[i] {
            clear_command = false;
        }
    }
    if clear_command {
        clear();
    }

    let test_arr: [char; 4] = ['t', 'e', 's', 't'];
    let mut test_command: bool = true;
    for i in 0..3 {
        if input_array[i] != test_arr[i] {
            test_command = false;
        }
    }
    if test_command {
        test();
    }

    let kmem_arr: [char; 5] = ['p', 'k', 'm', 'e', 'm'];
    let mut kmem_command: bool = true;
    for i in 0..4 {
        if input_array[i] != kmem_arr[i] {
            kmem_command = false;
        }
    }
    if kmem_command {
        pkmemtable();
    }
}


unsafe extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

use crate::println;
use crate::uart::Uart;
use crate::print;

// Initializes the process loop and uses arena allocaiton to allocate
// a heap
pub fn shmage_init() -> ! {
    let mut uart_instance = Uart::new(0xD4017000);
    // uart_instance.init();
    shfetch();
    page::init();
    unsafe {
    println!("heap start = {:#x}", HEAP_START);
    println!("heap size = {:#x}", HEAP_SIZE);
    }
    //malloc::init();
    let mut input_array: [char; 8] = [' ',' ',' ',' ',' ',' ',' ',' '];
    // single character input process loop
    let mut input_i: usize = 0;
    // prob eventually want to represent shell state in an enum
    let mut prompt_active: bool = true;
    loop {
        if prompt_active {
            print!("t(-_-) — ˎˊ˗");
            prompt_active = false;
        }
        // Get the character
        if let Some(c) = uart_instance.get() {
            match c {
                0x08b => {
                    // 8 is the backspace character, need to replace the
                    // previous character with a ' '
                    print!("{}{}{}", 0x08b as char, ' ', 0x08b as char);
                    if input_i > 0 {
                        input_i -= 1;
                        input_array[input_i] = ' ';
                    }
                },
                10 | 13 => {
                    // carriage returns
                    println!();
                    basic_command_process(&input_array);
                    input_array = [' ',' ',' ',' ',' ',' ',' ',' '];
                    input_i = 0;
                    prompt_active = true;
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
                                        _ => {

                                            println!("idk what happened");
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        print!("{}", c as char);
                        if input_i < 7 {
                            input_array[input_i] = c as char;
                            input_i += 1;
                        }
                },
                }
            }
            // Try to sleep the processor, i think this would work but
            // qemu uses a whole core
       //     for i in 0..1000{
       //         unsafe {
       //             asm!("ADDI x0, x0, 0")
       //         }
       //     }
        }
    }
