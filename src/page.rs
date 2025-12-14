//! RISCV page grained memory virtualization for shmageOS
//! None too different from unix local memory virtualization.
//! Haven't really decided on whether or not to include partitioned global address space stuff here, or keep that as an abstraction over this
use core::{mem::size_of, ptr::null_mut};
use crate::{println, print};

unsafe extern "C" {
    static HEAP_START: usize;
    static HEAP_SIZE: usize;
}

// allocated memory start address
static mut ALLOC_START: usize = 0b0;
const PAGE_ORDER: usize = 12;
// size of a page (2**12 bytes or 4096 bytes)
pub const PAGE_SIZE: usize = 0b1 << 12;

// bit repsresentation of a page
// (first bit sets whether taken or not,
// second bit sets whether its the last)
#[repr(u8)]
pub enum PageBits {
    Empty = 0b0,
    Taken = 0b1 << 0,
    Last = 0b1 << 1,
}

impl PageBits {
    pub fn val(self) -> u8 {
        self as u8
    }
}

// Idk it basically takes a value it aligns it to a power of 2
pub const fn align_value(value: usize, order: usize) -> usize {
    let order = (1usize << order)  - 1;
    (value + order) & !order
}

pub struct Page {
    flags: u8,
}

impl Page {
    pub fn is_last(&self) -> bool {
        if self.flags & PageBits::Last.val() != 0 {
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

/// Pages at virtual addresses, without zeroing the start pointer
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

/// Deallocate the page at the virt address
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

/// Allocate and zero one more or pages at virtual addresses, zeroing the start pointer
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

// Allocate zero or more pages in the partitioned global address space.
// note that like all page grained allocations, will allocate different
// physical memory on different physical machines
// allocated pages will be distributed across machines via fat pointers,
// will prob need the fat pointer PGAS abstraction in a separate library first
// the global address space should include a number of pages equal to n * pages, where n is the total num of nodes
pub fn galloc(pages: usize, n_nodes: usize, node_names: *mut u8) -> *mut u8 {
    // to be implemented... (requires virtual driver for cluster communication, so come back when you have UDP comms done)
    0 as *mut u8
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
        ALLOC_START = align_value(HEAP_START + num_pages * size_of::<Page,>(), PAGE_ORDER);
    }
}

pub struct PageTable {
    pub entries: [PageTableEntry; 512]
}
impl PageTable {
    pub fn len() -> usize {
        512
    }
}

// The bit representation of the beginning arg bits of 64 bit page table entries
#[repr(usize)]
#[derive(Copy, Clone)]
pub enum PageTableEntryBits {
    None = 0b0,
    Valid = 0b1 << 0, // first bit is valid or not
    Read = 0b1 << 1, // second bit is read permissions.
    Write = 0b1 << 2,
    Execute = 0b1 << 3,
    User = 0b1 << 4,
    Global = 0b1 << 5,
    Access =  0b1 << 6,
    Dirty = 0b1 << 7,
    ReadWrite = 0b1 << 1 | 0b1 << 2, // Combos are just bitwise ors
    ReadExecute = 0b1 << 1 | 0b1 << 3,
    ReadWriteExecute = 0b1 << 1 | 0b1 << 2 | 0b1 << 3,
    UserReadWrite = 0b1 << 1 | 0b1 << 2 | 0b1 << 4,
    UserReadExecute = 0b1 << 1 | 0b1 << 3 | 0b1 << 4,
    UserReadWriteExecute = 0b1 << 1 | 0b1 << 2 | 0b1 << 3 | 0b1 << 4,
}
impl PageTableEntryBits {
    pub fn as_usize(self) -> usize {
        self as usize
    }
    pub fn as_i64(self) -> i64 {
        self as i64
    }
    pub fn val(self) -> u8 {
        self as u8
    }
}

// does this need to be a struct? probably not but we get some
// more convenient interfaces
pub struct PageTableEntry {
    pub entry : i64,
}

impl PageTableEntry {
    pub fn is_valid(&self) -> bool {
        // checks the valid bit of the entry
        self.get_entry() & PageTableEntryBits::Valid.as_i64() != 0b0
    }
    // in riscv an entry is a leaf if any of the read write execute bits are set
    pub fn is_leaf(&self) -> bool {
        self.get_entry() & 0b111 != 0b000
    }
    // getter setter interface makes it so you can have immutable interface for
    // pte i think
    pub fn get_entry(&self) -> i64 {
        self.entry as i64
    }
    pub fn set_entry(&mut self, entry: i64) {
        self.entry = entry;
    }
    pub fn get_entry_as_usize(&self) -> usize {
        self.entry as usize
    }
}

// Map virtual memory onto physical memory in the PageTable
pub fn map(root: &mut PageTable, virtual_address: usize, physical_address: usize, bits: i64, level: usize) {
    // ensure rwx bits provided otherwise a memory leak will occur
    assert!(bits & 0b111 != 0b000);
    // get the the virtual page number fro mthe virtual address
    // page number is 9 bits so we use a 9 bit mask to just get the 9 bits of the page after rotating
    let virtual_page_numbers = [
        // remember the leading 12 bits aren't used and offset the pages so all our masks need to rotate this offest out
        (virtual_address >> 12) & 0b111111111, // bits 12:20 of the address
        (virtual_address >> 21) & 0b111111111, // bits 21:29 of the address
        (virtual_address >> 30) & 0b111111111, // bits 30:38 of the address
    ];
    // physical page number extraction is similar, but the last physical page uses all remaining 26 bits instead of 9
    let physical_page_numbers = [
        (physical_address >> 12) & 0b111111111, // bits 12:20 of the address
        (physical_address >> 21) & 0b111111111, // bits 21:29 of the address
        (physical_address >> 30) & 0b11111111111111111111111111, // bits 30:55 of the address
    ];
    let mut moving_pte_reference = &mut root.entries[virtual_page_numbers[2]];
    // traverse the pagetable and set bits accordingly
    for i in (level..2).rev() {
        if !moving_pte_reference.is_valid() {
            let page = zalloc(1);
            // we right shift by 2 places (ig cuz the rsw bits are still there?)
            moving_pte_reference.set_entry((page as i64 >> 2) | PageTableEntryBits::Valid.as_i64());
        }
        let entry = ((moving_pte_reference.get_entry() & !0b1111111111) as *mut PageTableEntry);
        // should we do better error handling than unwrapping here?
        let moving_pte_reference = unsafe { entry.add(virtual_page_numbers[i]).as_mut().unwrap() };
    }
    // After the loop should be at the 0th virtual pagen umber entry
    // set our entry to the expected entry structure
    let entry = (physical_page_numbers[2] << 28) as i64 | //the second entry is bits [53:28]
    (physical_page_numbers[1] << 19) as i64 |
    (physical_page_numbers[0] << 10) as i64 |
    bits | // reminder these are the user read write bits specified in args
    PageTableEntryBits::Valid.as_i64();
    moving_pte_reference.set_entry(entry);
}

// Map a range of addresses to the given page table
pub fn map_range(root_pointer: &mut PageTable, start_address: usize, end_address: usize, bits: i64) {
    let mut memory_address = start_address & !(PAGE_SIZE - 1);
    let num_pages = (align_value(end_address, 12) - memory_address) / PAGE_SIZE;
    for _ in 0..num_pages {
        map(root_pointer, memory_address, memory_address, bits, 0);
        memory_address += 1 << 12;
    }
}

// Unmap all memory from the root of the pagetable
pub fn unmap(root: &mut PageTable) {
    // Page table starts at level 2
    for level_2_table_i in 0..PageTable::len() {
        let ref level_2_entry = root.entries[level_2_table_i];
        if level_2_entry.is_valid() && !level_2_entry.is_leaf() {
            // If valid, free down the table
            let level_1_memory_address = (level_2_entry.get_entry() & !0b1111111111) << 2;
            let level_1_table = unsafe {
                (level_1_memory_address as *mut PageTable).as_mut().unwrap()
            };
            for level_1_table_i in 0..PageTable::len() {
                let ref level_1_entry = root.entries[level_1_table_i];
                if level_1_entry.is_valid() && !level_1_entry.is_leaf() {
                    let level_0_memory_address = (level_1_entry.get_entry() & !0b1111111111) << 2;
                    // free level 0, the outermost leaves of the tree
                    dealloc(level_0_memory_address as *mut u8);
                }
            }
            // once everything at level 0 is deallocated, deallocated level 1
            dealloc(level_1_memory_address as *mut u8);
            // note that the level 2 (highest level root) is not freed.
        }
    }
}

pub fn virtual_to_physical(root: &PageTable, virtual_address: usize) -> Option<usize> {
    let virtual_page_numbers = [
        // remember the leading 12 bits aren't used and offset the pages so all our masks need to rotate this offest out
        (virtual_address >> 12) & 0b111111111, // bits 12:20 of the address
        (virtual_address >> 21) & 0b111111111, // bits 21:29 of the address
        (virtual_address >> 30) & 0b111111111, // bits 30:38 of the address
    ];
    // Reminder the upper bits of the address define the highest lvel (root) of the pagetable
    let mut moving_pte_reference = &root.entries[virtual_page_numbers[2]];
    for i in (0..=2).rev() {
        if !moving_pte_reference.is_valid() {
            // need to page fault if the reference ends up being invalid
            break;
        }
        else if moving_pte_reference.is_leaf() {
            /*
            any level can be a leaf in risc V. in this case we have found a leaf and return it
            we apply an offest_mask to the virtual page number to get the physical page number
            since physical page numbers are offset by 12 bits + 9 for every page.
            the mask applies this offset, thereby giving us the correct physical address
            */
            let offset_mask = (0b1 << (12 + i * 9)) - 0b1;
            let virtual_address_page_offset = virtual_address & offset_mask;
            // To get a physical address we unmask after applying the 2 bit offset we had to apply above
            let physical_address = ((moving_pte_reference.get_entry() << 2) as usize) & !offset_mask;
            /*
            since we should have a valid physical address, we should return
            we need to flip the bits based on the page offset, since the unmasking process removed them
            */
            return Some(physical_address | virtual_address_page_offset);
        }
        /*
        in this case, the reference is a valid nonleaf entry. we need to set our moving refernce to the
        next entry pointed to by this entry. we did this before, but to recap we get the next entry via .get_entry(),
        mask the first 10 bits and right shift 2 places, since it was left shifted when the entry was made
        */
        let next_entry = ((moving_pte_reference.get_entry() & !0b1111111111) << 2) as *const PageTableEntry;
        // Note that if the "is_valid()" check at the top if statement works, we should hopefully not get
        // i == 0 here leading to a -1 indexing of page numbers lol
        moving_pte_reference = unsafe{ next_entry.add(virtual_page_numbers[i - 1]).as_ref().unwrap() }
    }
    // reaching here means we didn't find any leaves in the page table (a page fault!)
    None
}


// SATP regsiter located at: 0x180





pub fn print_alloc_start() {
    unsafe {
        let starting_page = HEAP_START as *const Page;
        let alloc_beginning = ALLOC_START;
        println!("pointer to starting page: {:p}", starting_page);
        println!("pointer to physical starting memory address: 0x{:x}", alloc_beginning);
    }
}

pub fn deallocate_all_pages() {
    unsafe {
		let num_pages = HEAP_SIZE / PAGE_SIZE;
		let mut beg = HEAP_START as *const Page;
		let end = beg.add(num_pages);
		let alloc_beg = ALLOC_START;
		let alloc_end = ALLOC_START + num_pages * PAGE_SIZE;
		let mut num = 0;
		while beg < end {
			if (*beg).is_taken() {
				let start = beg as usize;
				let memaddr = ALLOC_START
				              + (start - HEAP_START)
				                * PAGE_SIZE;
				loop {
					num += 1;
					if (*beg).is_last() {
						let end = beg as usize;
						let memaddr = ALLOC_START
						              + (end
						                 - HEAP_START)
						                * PAGE_SIZE
						              + PAGE_SIZE - 1;
						break;
					}
					beg = beg.add(1);
				}
                dealloc(memaddr as *mut u8);
			}
			beg = beg.add(1);
		}
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
		         "page allocation table                 |\r\n|meta: {:p} -> {:p}        |\r\n|physical mem: \
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
