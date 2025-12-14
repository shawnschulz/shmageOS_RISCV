// Provides the memory for the kernel, for now also exposes a global allocator for heap memory

use crate::page::{align_value, zalloc, PageTable, PAGE_SIZE};
use core::{mem::size_of, ptr::null_mut};
use crate::println;
use crate::print;

// Mark an allocated address as taken by setting the 64th bit to 1
#[repr(usize)]
enum AllocationFlags {
    Taken = 0b1 << 63
}
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

// Kernel memory needs an allcoator interface we can use. since our memory paging is setup,
// normally we would do this by creating a special syscall function which initializes a page table
// in the kernel of the requested size, maps the page table to physical memory
// and returns the pointer to its head. however there are 2 problems:
// 1. the page table head is just a virtual abstraction. the pointer we give to rust  will not
// actually be mapped onto physical memory. we have to somehow get rust to use the translation
// layer for interacting with this memory
// 2. we have to somehow tell rust to actually use the virtual memory system we made, i.e. actually
// do the page translations with offsets and also allocate, deallocate and dereference using
// the correct addresses.
// The solution: rust provides a template macro to allow you to define your own allocators.
// We will need to use these templates along with our paging system to define our allocator

// Note that for now we are only allocating some heap memory for the kernel. but eventually
// we may need to edit this or use some other method to provide a syscall interface for
// processes to ask for heap allocated memory (i.e. to malloc)
use core::alloc::{GlobalAlloc, Layout};
struct KernelAllocator;

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        kernel_zmalloc(layout.size())
    }
    unsafe fn dealloc(&self, pointer: *mut u8, _layout: Layout) {
        kernel_free(pointer);
    }
}

#[global_allocator]
static GLOBAL: KernelAllocator = KernelAllocator;

#[alloc_error_handler]
// undefined to call alloc on a null_ptr, since we are doing this in the kernel the entire kernel
// should panic (would crash anyways) so we can more easily find where this happens
pub fn kernel_alloc_error(layout: Layout) -> ! {
    panic!(
        "[ERROR] Kernel failed to allocate {} bytes with {} byte-alignment.",
        layout.size(),
        layout.align()
    )
}
