use crate::{print, println, UART_BASE_ADDRESS};

#[unsafe(no_mangle)]
pub extern "C" fn s_trap(epc: usize,
                     tval: usize,
                     cause: usize,
                     hart: usize,
                     status: usize,
                     frame: usize) ->usize {
    // Maybe we need to dereference the pointers to actually get the value in them?
    // or are they registers that just contain the info we need?
    println!("[INFO] Encountered a trap: tval: {} | cause: {} | hart: {} | status: {} | frame: {}", tval, cause, hart, status, frame);
    return 0x0;
}
