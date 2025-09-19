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
    pub fn get_size(&self) {
        self.flags_size & !AllocationFlags::Taken.value();
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
       let aligned_size = align_value(size, 3) + size_of::<AllocationList>();
       let mut head = KERNEL_MEMORY_HEAD;
    }
}

// allocate zeroed memory based on number of bytes
pub fn kernel_zmalloc(size: usize) -> *mut u8 {
    let aligned_size = align_value(size, 3);
    let ret = kernel
}
