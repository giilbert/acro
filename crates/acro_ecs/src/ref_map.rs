use std::cell::Ref;

use ouroboros::self_referencing;

pub struct RefMap<'b, T> {
    inner_ref: Ref<'b, T>,
}

impl<'b, T> RefMap<'b, T> {
    pub fn new(inner_ref: Ref<T>, map: impl FnOnce(Ref<T>) -> &Ref<T>) -> Self {
        todo!();
    }
}

#[cfg(test)]
mod tests {
    use std::cell::{Ref, RefCell};

    use super::RefMap;

    pub struct A {
        pub inner: RefCell<u32>,
    }

    #[test]
    fn ref_map_test() {
        let inner_cell = RefCell::new(42u32);
        let outer_cell = RefCell::new(A { inner: inner_cell });

        let outer_ref = RefMap::new(outer_cell.borrow(), |a| &a.inner.borrow());
    }
}
