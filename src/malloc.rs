// Provides the allocators for malloc using page module

use crate::page::{align_value, zalloc, PageTable, PAGE_SIZE};
