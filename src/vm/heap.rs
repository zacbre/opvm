use crate::vm::field::Field;

pub struct Heap {
    pub item: Option<Box<Field>>
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            item: None
        }
    }
}