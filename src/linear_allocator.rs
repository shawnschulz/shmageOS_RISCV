// As an alternative to the linked list memory paging system, this simple linear allocator
// let's us play aorund with a shell to start

use core::sync::atomic::{AtomicUsize};

// Contains information about a linearly allocated heap
pub struct LinearAllocator {
    head: AtomicUsize, // index of the buffer
    // The index of the buffer is atomic, so threads can only access one at a time
    // (helping with safety of accesss)
    start: *mut u8,
    end: *mut u8,
}

unsafe impl Sync for LinearAllocator {}

impl LinearAllocator {
    pub const fn empty() -> Self {
        Self {
            head: AtomicUsize::new(0),
            start: core::ptr::null_mut(),
            end: core::ptr::null_mut(),
        }
    }
    pub fn init(&mut self, start: usize, size: usize) {
        self.start = start as *mut u8;
        // dereferencing a raw pointer here
        self.end = unsafe { self.start.add(size) };
    }
}

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use core::sync::atomic::{Ordering};
// Rust needs to know that this is a global memory allocator, so it can use a shared
// alloc and dealloc interface
unsafe impl GlobalAlloc for LinearAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // using the core Layout crate to keep bytes aligned for performance
        let align = layout.align();
        // number of bytes to allocate
        let size = layout.size();
        let mut head = self.head.load(Ordering::Relaxed);
        // align the head
        if head % align != 0 {
            // e.g. if head is 1 and alignment is 4 bytes, 3 will get added to the head
            head += align - (head % align);
        }
        // Move the head forward by the allocation size
        let new_head = head + size;
        // Check for going over end of heap memory
        if self.start.add(new_head) > self.end {
            return core::ptr::null_mut();
        }
        self.head.store(new_head, Ordering::Relaxed);
        // This core struct let's us handle errors rather than straight up returning a null pointer
        NonNull::new_unchecked(self.start.add(head) as *mut u8).as_ptr()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // This doesn't do anything! We can't really free memory, only reset the entire arena for something else
    }
}
