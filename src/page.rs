use core::{mem::size_of, ptr::null_mut};

unsafe extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

pub const PAGE_SIZE: usize = 1 << 12;

#[repr(u8)]
pub enum PageBits {
    Empty = 0,
    Taken = 1 << 0,
    Last = 1 << 1,
}

impl PageBits {
    pub fn val(self) -> u8 {
        self as u8
    }
}


pub struct Page {
    flags: u8,
}

impl Page {
    pub fn is_last(&self) -> bool {
        if self.flags & PageBits::last.val() != 0{
            true
        } else {
            false
        }
    }
    pub fn is_taken (&self) -> bool {
        if self.flags & PageBits::Taken.val() != 0 {
            true
        } else {
            false
        }
    }
    pub fn is_free (&self) -> bool {
        !self.is_taken()
    }
    pub fn clear(&mut self) {
        self.flags = PageBits::Empty.val();
    }
    pub fn set_flag(&mut self, flag: PageBits) {
        // This is how we actually set the bit value of the flags for the pages
        self.flag |= flag.val()
    }
}

pub fn alloc(pages: usize) -> *mut u8 {
    // Pages must be contiguous
    assert!(pages > 0);
    unsafe {
        // create the page structure
        let num_pages = HEAP_SIZE / PAGE_SIZE;
        let pointer = HEAP_START as *mut Page;
        // dfs through the pages
        for i in 0..num_pages-pages {
            // look for a free page
            let mut found = false;
            if (*pointer.add(i)).is_free() {
                found = true;
                for j in i..i+pages {
                    // check if contiguous allocation for requested pages
                    if (*pointer.add(j).is_taken()) {
                        found = false;
                        break
                    }
                }
            }
            // If we found an available set of contiguous pages,
            // we can set the flags to say it's taken and return
            // the pointer. Otherwise, a page fault occurs
            if found {
                for k in 1..i+pages-1 {
                    // Go through and set the contigous pages to taken
                    (*pointer.add(k)).set_flag(PageBits::Taken);
                }
                // This lets us know what the last page is
                (*pointer.add(i+pages-1)).set_flag(PageBits::Taken);
                (*pointer.add(i+pages-1)).set_flag(PageBits::Last);
                // Remember the page structure is just an abstraction
                // the kernel uses to keep track of memory allocation,
                // we return an address at the number of pages after the
                // start where we can start using memory
                return (ALLOC_START + PAGESIZE * i) as *mut u8;
            }
        }
    }
}

// TODO: Implement the dealloc function
