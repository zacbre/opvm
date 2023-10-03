use linked_list_allocator::Heap as heap;
use once_cell::sync::Lazy;
use std::{
    alloc::Layout,
    ptr::NonNull,
    sync::{Arc, Mutex, MutexGuard},
};

const MAX_HEAP_SIZE: usize = 100;
pub static mut HEAP_MEM: [u8; MAX_HEAP_SIZE] = [0; MAX_HEAP_SIZE];
static mut HEAP_ALLOCATED: bool = false;
static mut HEAP_INSTANCE: Lazy<Arc<Mutex<Heap>>> = Lazy::new(|| {
    let heap = Arc::new(Mutex::new(Heap {
        allocator: heap::empty(),
    }));
    unsafe {
        heap.lock()
            .unwrap()
            .allocator
            .init(HEAP_MEM.as_mut_ptr(), MAX_HEAP_SIZE);
    }
    heap
});

#[derive(Debug)]
pub struct Heap {
    allocator: heap,
}

impl Heap {
    pub fn get() -> Arc<Mutex<Self>> {
        unsafe {
            return HEAP_INSTANCE.clone();
        }
    }

    pub fn reset(&self) {
        unsafe {
            if HEAP_ALLOCATED {
                HEAP_MEM = [0; MAX_HEAP_SIZE];
            }
        }
    }

    pub fn allocate(&mut self, size: usize) -> Result<NonNull<u8>, ()> {
        self.allocator
            .allocate_first_fit(Layout::from_size_align(size, 2).map_err(|_| ())?)
    }

    pub fn deallocate(&mut self, ptr: NonNull<u8>, size: usize) -> Result<(), ()> {
        unsafe {
            self.allocator
                .deallocate(ptr, Layout::from_size_align(size, 2).map_err(|_| ())?);
        }

        Ok(())
    }

    pub fn recover_poison<'a>(heap: &'a Arc<Mutex<Heap>>) -> MutexGuard<'a, Heap> {
        let mut_heap = heap.lock();
        let data: MutexGuard<'a, Heap> = match mut_heap {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        data
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_heap() {
        let heap = Heap::get();
        let mut mut_heap = Heap::recover_poison(&heap);
        let ptr = mut_heap.allocate(10).unwrap();
        unsafe {
            ptr.as_ptr().write(10);
            assert_eq!(ptr.as_ptr().read(), 10);
        }
        mut_heap.deallocate(ptr, 10).unwrap();
    }

    #[test]
    #[should_panic]
    fn should_panic_when_deallocate_heap_out_of_bounds() {
        let heap = Heap::get();
        let mut mut_heap = Heap::recover_poison(&heap);
        let ptr = mut_heap.allocate(10).unwrap();
        mut_heap.deallocate(ptr, 100).unwrap();
    }
}
