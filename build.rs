// build.rs
use std::fs;
use std::env;

// Idea is you can run 'cargo build qemu' or 'cargo build orangepi' and it will
// automatically modify the lds script to the correct values
fn main() {
    let build_target = env::var("HARDWARE_TARGET")
        .unwrap_or_else(|_| "qemu".to_string());

    println!("cargo:warning=Building with hardware target: {}", build_target);


    let linker_template = fs::read_to_string("./src/lds/virt.lds.template")
        .expect("Failed to read virt.lds.linker_template");

    let mem_template = fs::read_to_string("./src/asm/mem.s.template")
        .expect("Failed to read mem.s.template");

    if build_target == "orangepi" {
        let linker_output = linker_template.replace("{ORIGIN_ADDRESS}", "0x11000000");
        let mem_output = mem_template.replace("{UART_BASE_ADDRESS}", "0xD4017000");
        fs::write("src/lds/virt.lds", linker_output)
            .expect("Failed to write linker.lds");
        fs::write("src/asm/mem.S", mem_output)
            .expect("Failed to write mem.s");
        let uart_src = fs::read_to_string("src/uart_orangepi.rs.template")
            .expect("Failed to read uart_orangepi.rs.template");
        fs::write("src/uart.rs", uart_src)
            .expect("Failed to write uart.rs");
    } else { // Default to qemu target otherwise
        let linker_output = linker_template.replace("{ORIGIN_ADDRESS}", "0x80000000");
        let mem_output = mem_template.replace("{UART_BASE_ADDRESS}", "0x10000000");
        println!("cargo:warning=Linker file output after modification: {:?}", linker_output);
        fs::write("./src/lds/virt.lds", linker_output)
            .expect("Failed to write linker.lds");
        fs::write("src/asm/mem.S", mem_output)
            .expect("Failed to write mem.s");
        let uart_src = fs::read_to_string("./src/uart_qemu.rs.template")
            .expect("Failed to read uart_orangepi.rs.template");
        fs::write("./src/uart.rs", uart_src)
            .expect("Failed to write uart.rs");
    }
    println!("cargo:warning=Finished building for hardware target");
    
    // Modify linker and asm scripts to use the input UART and memory start magic numbers
    
    println!("cargo:rerun-if-changed=src/virt.lds.template");
    println!("cargo:rerun-if-changed=src/uart.rs.template");
}
