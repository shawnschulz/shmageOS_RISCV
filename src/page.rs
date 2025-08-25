use core::{mem::size_of, ptr::null_mut};
use crate::{println, print};

// i think we have to define this ourselves???
unsafe extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

static mut ALLOC_START: usize = 0;
const PAGE_ORDER: usize = 12;
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

// Idk it basically takes a value it aligns it to a power of 2
pub const fn align_val(val: usize, order: usize) -> usize {
    let order = (1usize << order)  - 1;
    (val + order) & !order
}

pub struct Page {
    flags: u8,
}

impl Page {
    pub fn is_last(&self) -> bool {
        if self.flags & PageBits::Last.val() != 0{
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
        self.flags |= flag.val()
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
                    if (*pointer.add(j)).is_taken() {
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
                return (ALLOC_START + PAGE_SIZE * i) as *mut u8;
            }
        }
    }
    // return a null mutable pointer if no contiguous allocation found
    null_mut()
}

/// note deallocating doesn't actually clear the memory, just the descriptor
pub fn dealloc(pointer: *mut u8) {
    assert!(!pointer.is_null());
    unsafe {
        // grab the address of the first page
        let address = HEAP_START + (pointer as usize - ALLOC_START) / PAGE_SIZE;
        // check that page structure makes sense
        assert!(address >= HEAP_START && address < HEAP_START + HEAP_SIZE);
        let mut page_instance = address as *mut Page;
        // Loop through the pages and clear them until we hit the last page
        while (*page_instance).is_taken() && !(*page_instance).is_last() {
            (*page_instance).clear();
            page_instance = page_instance.add(1);
        }
        // Try to prevent double frees
        assert!((*page_instance).is_last() == true,
            "Possible double free detected");
        // clear the last page
        (*page_instance).clear();
    }
}

/// Allocate and zero one more or pages
pub fn zalloc(pages: usize) -> *mut u8 {
    let ret = alloc(pages);
    // If the ALLOC_START pointer is not null, need to zero it
    if !ret.is_null() {
        let size = (PAGE_SIZE * pages) / 8;
        // use a u64 instead of a u8 to force store doubleword sd instruction instead
        // of store byte (sb) to preform 8x less stores. normally this would not work
        // as we need to handle remaining bytes, but here 4096 %  8 = 0, so we don't
        // have to worry about it
        // basically overwriting pages with 0s in 8 byte (u64) sized pointers
        let big_pointer = ret as *mut u64;
        for i in 0..size {
            unsafe {
                (*big_pointer.add(i)) = 0;
            }
        }
    }
    ret
}

pub fn init() {
    unsafe {
        let num_pages = HEAP_SIZE / PAGE_SIZE;
        let pointer = HEAP_START as *mut Page;
        // clear all pages
        for i in 0..num_pages {
            (*pointer.add(i)).clear();
        }
        // align  alloc start to the page boundry
        ALLOC_START = align_val(HEAP_START + num_pages * size_of::<Page,>(), PAGE_ORDER);
    }
}

pub fn print_alloc_start() {
    unsafe {
        let starting_page = HEAP_START as *const Page;
        let alloc_beginning = ALLOC_START;
        println!("pointer to starting page: {:p}", starting_page);
        println!("pointer to physical starting memory address: 0x{:x}", alloc_beginning);
    }
}


pub fn print_page_allocations() {
	unsafe {
		let num_pages = HEAP_SIZE / PAGE_SIZE;
		let mut beg = HEAP_START as *const Page;
		let end = beg.add(num_pages);
		let alloc_beg = ALLOC_START;
		let alloc_end = ALLOC_START + num_pages * PAGE_SIZE;
		println!();
		println!(" ______________________________________");
        print!("|");
		println!(
		         "page allocation table                 |\n|meta: {:p} -> {:p}        |\n|physical mem: \
		          0x{:x} -> 0x{:x}|",
		         beg, end, alloc_beg, alloc_end
		);
		println!(" --------------------------------------");
		let mut num = 0;
		while beg < end {
			if (*beg).is_taken() {
				let start = beg as usize;
				let memaddr = ALLOC_START
				              + (start - HEAP_START)
				                * PAGE_SIZE;
				print!("0x{:x} => ", memaddr);
				loop {
					num += 1;
					if (*beg).is_last() {
						let end = beg as usize;
						let memaddr = ALLOC_START
						              + (end
						                 - HEAP_START)
						                * PAGE_SIZE
						              + PAGE_SIZE - 1;
						print!(
						       "0x{:x}: {:>3} page(s)",
						       memaddr,
						       (end - start + 1)
						);
						println!(".");
						break;
					}
					beg = beg.add(1);
				}
			}
			beg = beg.add(1);
		}
		println!(" ________________________________________");
		println!(
		         "|allocated: {:>5} pages ({:>9} bytes)|",
		         num,
		         num * PAGE_SIZE
		);
		println!(
		         "|free     : {:>5} pages ({:>9} bytes)|",
		         num_pages - num,
		         (num_pages - num) * PAGE_SIZE
		);
		println!(" ----------------------------------------");
	}
}
