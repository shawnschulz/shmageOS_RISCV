// build.rs
use std::fs;
use std::env;

// Idea is you can run 'cargo build qemu' or 'cargo build orangepi' and it will
// automatically modify the lds script to the correct values
fn main() {
    let args: Vec<String> = env::args().collect();
    let build_target = &args[1];

    let linker_template = fs::read_to_string("src/lds/virt.lds.template")
        .expect("Failed to read virt.lds.linker_template");

    let uart_driver_template = fs::read_to_string("src/uart.rs.template")
        .expect("Failed to read virt.lds.uart_driver_template");

    if (build_target == "orangepi") {
        let linker_output = linker_template.replace("{ORIGIN_ADDRESS}", "0x11000000");
        fs::write("src/lds/virt.lds", linker_output)
            .expect("Failed to write virt.lds.html");

        let mut uart_output = uart_driver_template.replace("{RHR_ADDRESSS}", "0x11000000");
        uart_output = uart_output.replace("{LHR_ADDRESS}", "0x11000000");
        fs::write("src/uart.rs", linker_output)
            .expect("Failed to write virt.lds.html");
    } else {
        let linker_output = linker_template.replace("{ORIGIN_ADDRESS}", "0x80000000");
        fs::write("src/lds/virt.lds", linker_output)
            .expect("Failed to write virt.lds.html");
    }
    
    // Modify linker and asm scripts to use the input UART and memory start magic numbers
    
    println!("cargo:rerun-if-changed=index.html.template");
    println!("cargo:rerun-if-env-changed=IP_ADDRESS");
}
