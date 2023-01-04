pub struct Heap {
}

impl Heap {
    pub fn new() -> Self {
        Heap {
        }
    }

    pub fn allocate(&mut self, size: usize) -> *mut [usize] {
        let heap = vec![0; size].into_boxed_slice();
        let ptr = Box::into_raw(heap);
        ptr
    }

    pub unsafe fn deallocate(&self, field: *mut [usize]) {
        let _ = Box::from_raw(field);
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        // todo: deallocate all in heap
    }
}