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


use crate::println;
use crate::uart::Uart;
use crate::print;

// Initializes the process loop and uses arena allocaiton to allocate
// a heap
pub fn shmage_init() -> ! {
    let mut uart_instance = Uart::new(0x1000_0000);
    uart_instance.init();
    shfetch();
    // single character input process loop
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
                },
                }
            }
        }
    }
