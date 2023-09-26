use std::{alloc::Layout, ptr::NonNull};

use linked_list_allocator::Heap as heap;

use super::vm::Vm;

const MAX_HEAP_SIZE: usize = 5000;

#[derive(Debug)]
pub struct Heap {
    storage: [u8; MAX_HEAP_SIZE],
    allocator: heap,
}

impl Heap {
    pub fn new() -> Self {
        let mut heap = Self {
            storage: [0; MAX_HEAP_SIZE],
            allocator: heap::empty(),
        };

        unsafe {
            heap.allocator
                .init(heap.storage.as_mut_ptr(), MAX_HEAP_SIZE);
        }

        heap
    }

    pub fn allocate(&mut self, size: usize) -> Result<NonNull<u8>, ()> {
        self.allocator
            .allocate_first_fit(Layout::from_size_align(size, 2).map_err(|_| ())?)
    }

    pub fn deallocate(&mut self, ptr: NonNull<u8>, size: usize) -> Result<(), ()> {
        println!("{:?}", self.storage);
        unsafe {
            self.allocator
                .deallocate(ptr, Layout::from_size_align(size, 2).map_err(|_| ())?);
        }

        Ok(())
    }
}
