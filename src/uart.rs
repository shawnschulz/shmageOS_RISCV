// set two bits in the line control register (LCR)
pub fn set_word_length() {}

pub fn enable_fifo() {}
pub fn enable_receiver_interrupts() {}
use core::fmt::{Write, Error};

pub struct Uart {
    base_address: usize,
}

impl Uart {
    pub fn new(base_address:usize) -> Self {
        Uart {
            base_address
        }
    }
    pub fn init(&mut self) {
        let pointer = self.base_address as *mut u8;
        unsafe {
            // set bits 0 and 1 to 1 in line control register (LCR) at 3 offset
            // (note this is the same as saying let lcr = 3:
            // 1 << 0 = 1 right shifting by 0 does nothing
            // 1 << 1 = 2, 10 << 1 = 01 = 2
            // 1 | 2 = 10 | 01 = 11 = 3!
            let lcr = (1 << 0) | (1 << 1);
            pointer.add(3).write_volatile(lcr);
            // Set the fifo control register's (FCR) at offset 2 bit to enable
            // to 1, which is at the first bit
            pointer.add(2).write_volatile(1);
            // flip on intterrupt enable register (IER) at offset 1
            pointer.add(1).write_volatile(1);
            // Need to actually try and set the baud here, since we want to
            // use a linux serial console from a real RISC-V processor
            // Would like baud rate of 115200 from 1.6 ghz
            // 869?
            let divisor: u16 = 869;
            let divisor_least: u8 = (divisor & 0xff).try_into().unwrap();
            let divisor_most: u8 = (divisor >> 8).try_into().unwrap();
            // write 1 to the 7th bit of LCR (divisor latch access bit)
            // so the hardware knows to change what the base address is
            pointer.add(3).write_volatile(lcr | 1 << 7);
            pointer.add(0).write_volatile(divisor_least);
            pointer.add(1).write_volatile(divisor_most);
            // Clear the DLAB bit 
            pointer.add(3).write_volatile(lcr);
        }
    }
    pub fn get(&mut self) -> Option<u8> {
        let pointer = self.base_address as *mut u8;
        unsafe {
            if pointer.add(5).read_volatile() & 1 == 0 {
                None
            }
            else {
                Some(pointer.add(0).read_volatile())
            }
        }
    }
    pub fn put(&mut self, value: u8) {
        let pointer = self.base_address as *mut u8;
        unsafe { 
            pointer.add(0).write_volatile(value);
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> Result<(), Error> {
        for c in s.bytes() {
            self.put(c);
        }
        Ok(())
    }
}
