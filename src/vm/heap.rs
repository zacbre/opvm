use std::collections::HashMap;

#[derive(Debug)]
pub struct Heap {
    allocations: HashMap<*mut [usize], usize>
}

impl Heap {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
        }
    }

    pub fn contains(&self, allocation: *mut [usize]) -> bool {
        self.allocations.contains_key(&allocation)
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.allocations.len()
    }

    #[allow(dead_code)]
    pub fn get_allocations(&self) -> HashMap<*mut [usize], usize> {
        self.allocations.clone()
    }

    pub fn deref(field: *mut [usize]) -> Box<[usize]> {
        unsafe { Box::from_raw(field) }
    }

    pub fn reref(field: Box<[usize]>) -> *mut [usize] {
        Box::into_raw(field)
    }

    pub fn allocate(&mut self, size: usize) -> *mut [usize] {
        let heap = vec![0; size].into_boxed_slice();
        let ptr = Box::into_raw(heap);
        // add this to hashmap.
        self.allocations.insert(ptr.clone(), size);
        ptr
    }

    pub unsafe fn free(&mut self, field: *mut [usize]) {
        let mut _dbox = Box::from_raw(field);
        for item in _dbox.iter_mut() {
            *item = 0;
        }
        self.allocations.remove(&field);
    }

    pub fn clear(&mut self) {
        let mut freed_allocations: Vec<*mut [usize]> = Vec::new();
        for (allocation, _) in &self.allocations {
            let mut _dbox = unsafe {Box::from_raw(allocation.clone())};
            freed_allocations.push(allocation.clone());
        }

        for item in freed_allocations {
            self.allocations.remove(&item);
        }
    }
}