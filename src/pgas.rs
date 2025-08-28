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

pub fn gdealloc(pages: usize) {
}
