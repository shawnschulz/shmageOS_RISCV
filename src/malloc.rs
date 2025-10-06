// Provides the memory for the kernel, for now also exposes a global allocator for heap memory

use crate::page::{align_value, zalloc, PageTable, PAGE_SIZE};
use core::{mem::size_of, ptr::null_mut};

// Mark an allocated address as taken by setting the 64th bit to 1
#[repr(usize)]
enum AllocationFlags {
    Taken = 0b1 << 63
}

impl AllocationFlags {
    pub fn value(self) -> usize {
       self as usize
    }
}

struct AllocationList {
    pub flags_size: usize
}

impl AllocationList {
    pub fn is_taken(&self) -> bool {
        self.flags_size & AllocationFlags::Taken.value() != 0b0
    }
    pub fn set_taken(&mut self) {
        self.flags_size |= AllocationFlags::Taken.value();
    }
    pub fn set_free(&mut self) {
        self.flags_size = !AllocationFlags::Taken.value() & self.flags_size;
    }
    // ngl this makes no sense to me
    pub fn set_size(&mut self, size: usize) {
        let taken_check = self.is_taken();
        self.flags_size = size & !AllocationFlags::Taken.value();
        if taken_check {
            self.flags_size = self.flags_size | AllocationFlags::Taken.value();
        }
    }
    pub fn get_size(&self) -> usize {
        self.flags_size & !AllocationFlags::Taken.value()
    }
}

// The head of kernel memory allocation
static mut KERNEL_MEMORY_HEAD: *mut AllocationList = null_mut();
//Track memory footprint to see if more pages need to be allocated to the kernel
static mut KERNEL_MEMORY_ALLOCATION_SIZE: usize = 0;
static mut KERNEL_MEMORY_PAGE_TABLE: *mut PageTable = null_mut();
pub fn get_head() -> *mut u8 {
    unsafe {KERNEL_MEMORY_HEAD as *mut u8}
}
pub fn get_page_table() -> *mut PageTable {
    unsafe {KERNEL_MEMORY_PAGE_TABLE as *mut PageTable}
}
pub fn get_number_allocations() -> usize {
    unsafe {KERNEL_MEMORY_ALLOCATION_SIZE}
}

// intialize kernel memory. user processes should not be allowed to do this
pub fn init() {
    unsafe {
        // allocate 64 kernel pages
        let kernel_allocation = zalloc(64);
        assert!(!kernel_allocation.is_null());
        KERNEL_MEMORY_ALLOCATION_SIZE = 64;
        KERNEL_MEMORY_HEAD = kernel_allocation as *mut AllocationList;
        (*KERNEL_MEMORY_HEAD).set_free();
        (*KERNEL_MEMORY_HEAD).set_size(KERNEL_MEMORY_ALLOCATION_SIZE * PAGE_SIZE);
        // since the page table is tracking our memory footprint dynamically it also needs memory allocated for it
        KERNEL_MEMORY_PAGE_TABLE = zalloc(1) as *mut PageTable;

    }
}

// allocate memory based on bytes
pub fn kernel_malloc(size: usize) -> *mut u8 {
    unsafe {
       let size = align_value(size, 3) + size_of::<AllocationList>();
       let mut head = KERNEL_MEMORY_HEAD;
       let tail = (head as *mut u8).add(KERNEL_MEMORY_ALLOCATION_SIZE * PAGE_SIZE) as *mut AllocationList;
        while head < tail {
            // while space in kernel memory left and more space to allocate, allocate chunks chunks
            // by iterating through linked list
            if !(*head).is_taken() && size <= (*head).get_size() {
                let chunk_size = (*head).get_size();
                let remainder = chunk_size - size;
                (*head).set_taken();
                if remainder > size_of::<AllocationList>() {
                    let next = (head as *mut u8).add(size)
                        as *mut AllocationList;
                    (*next).set_free();
                    (*next).set_size(remainder);
                    (*head).set_size(size);
                }
                else {
                    // give the head the whole chunk if the remaining free space is bigger than how much your allocating
                    (*head).set_size(chunk_size);
                }
                return head.add(1) as *mut u8;
            }
            else {
               // since chunk wasn't free, move on to next chunk
               head = (head as *mut u8).add((*head).get_size()) as *mut AllocationList;
            }
        }
        // If we go through all the addresses and don't find any chunks we can allocate, return a null ptr
        null_mut()
    }
}

 // allocate zeroed memory based on number of bytes
 pub fn kernel_zmalloc(size: usize) -> *mut u8 {
     let aligned_size = align_value(size, 3);
     let ret = kernel_malloc(size);
     if !ret.is_null() {
         for i in 0..size {
             unsafe {
                 (*ret.add(i)) = 0;
             }
         }
     }
     return ret;
 }

// free kernel allocated memory
pub fn kernel_free(address_pointer: *mut u8) {
    unsafe {
        if !address_pointer.is_null() {
            let memory_pointer = (address_pointer as *mut AllocationList).offset(-1);
            if (*memory_pointer).is_taken() {
                (*memory_pointer).set_free();
            // free space pattern: want to coalesce smaller chunks into a bigger chunk of free memory
            }
            coalesce();
        }
    }
}

// Take the kernel memory head and traverse it looking for contiguous addresses that are free
// if 2 contiguous addresses are free, coalesce them into one address
pub fn coalesce() {
    unsafe {
        let mut head = KERNEL_MEMORY_HEAD;
        let tail = (head as *mut u8).add(KERNEL_MEMORY_ALLOCATION_SIZE * PAGE_SIZE) as *mut AllocationList;
        while head < tail {
            // Get the next address
            let next = (head as *mut u8).add((*head).get_size()) as *mut AllocationList;
            // in this case, could have a double free or other problem. continue to the next one
            if (*head).get_size() == 0 {
                break
            }
            // in this case, the next pointer has gone past the tail, should not do anything in this case
            else if next >= tail {
                break
            }
            // if they are both free, coalesce them into one address by setting the size of the
            // head to go over the next addresses's size
            if !(*head).is_taken() && !(*next).is_taken() {
                (*head).set_size((*head).get_size() + (*next).get_size())
            }
            // go to the next address
            head = (head as *mut u8).add((*head).get_size()) as *mut AllocationList;
        }
    }
}

// print for debugging ( this is pulled directly from the tutorial )
pub fn print_kernel_memory_table() {
    unsafe {
        let mut head = KERNEL_MEMORY_HEAD;
        let tail = (head as *mut u8).add(KERNEL_MEMORY_ALLOCATION_SIZE * PAGE_SIZE) as *mut AllocationList;
        println!("address, size, free_status");
        while head < tail {
            println!("{:p}, {:<10}, {}", head, (*head).get_size(), (*head).is_taken());
            head = (head as *mut u8).add((*head).get_size()) as *mut AllocationList;
        }
    }
}
