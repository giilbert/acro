use std::{
    alloc::{handle_alloc_error, Layout},
    ptr::NonNull,
};

pub type Dropper = Option<unsafe fn(NonNull<u8>)>;

#[derive(Debug)]
pub struct AnyVec {
    layout: Layout,
    length: usize,
    capacity: usize,
    data: NonNull<u8>,
    dropper: Dropper,
}

#[derive(Debug)]
pub struct SwapRemoveResult {
    pub is_last: bool,
    pub replacement_index: Option<usize>,
}

impl AnyVec {
    pub fn new(layout: Layout, dropper: Dropper, capacity: usize) -> Self {
        if layout.size() == 0 {
            Self {
                data: NonNull::dangling(),
                layout,
                capacity: usize::MAX,
                length: 0,
                dropper,
            }
        } else {
            let initial_allocation = NonNull::new(unsafe {
                std::alloc::alloc(array_layout(&layout, capacity).expect("error in layout"))
            })
            .expect("allocation failed");

            Self {
                data: initial_allocation,
                layout,
                capacity,
                length: 0,
                dropper,
            }
        }
    }

    pub fn new_of<T: 'static>(capacity: usize) -> Self {
        Self::new(
            Layout::new::<T>(),
            Some(|ptr| unsafe {
                std::ptr::drop_in_place(ptr.as_ptr() as *mut T);
            }),
            capacity,
        )
    }

    /// Caller must ensure that the type of `item` matches the type of the vector.
    pub unsafe fn push<T>(&mut self, item: T) {
        debug_assert!(self.layout.size() == std::mem::size_of::<T>());
        debug_assert!(self.layout.align() == std::mem::align_of::<T>());
        self.push_from_ptr(std::mem::transmute(&item as *const T));
    }

    /// Caller must ensure that the type of data `ptr` is the same as the type of the vector.
    pub unsafe fn push_from_ptr(&mut self, ptr: *const u8) {
        self.reserve(1);
        std::ptr::copy_nonoverlapping(
            ptr,
            self.data.as_ptr().add(self.length * self.layout.size()),
            self.layout.size(),
        );
        self.length += 1;
    }

    pub unsafe fn set_from_ptr(&mut self, from: *const u8, to: usize) {
        assert!(to < self.len(), "index out of bounds");
        std::ptr::copy_nonoverlapping(
            from,
            self.data.as_ptr().add(to * self.layout.size()),
            self.layout.size(),
        );
    }

    /// Caller must ensure that `T` matches the type of the vector.
    pub unsafe fn get<T: 'static>(&self, index: usize) -> Option<&T> {
        if index < self.length {
            Some(&*(self.data.as_ptr().add(index * self.layout.size()) as *const T))
        } else {
            None
        }
    }

    unsafe fn copy_within(&mut self, from: usize, to: usize) {
        debug_assert!(from < self.len(), "index out of bounds");
        debug_assert!(to < self.len(), "index out of bounds");
        std::ptr::copy_nonoverlapping(
            self.data.as_ptr().add(from * self.layout.size()),
            self.data.as_ptr().add(to * self.layout.size()),
            self.layout.size(),
        );
    }

    pub unsafe fn swap_remove(&mut self, index: usize) -> SwapRemoveResult {
        assert!(index < self.len(), "index out of bounds");

        let last_index = self.len() - 1;

        if last_index == index {
            self.length -= 1;
            return SwapRemoveResult {
                is_last: true,
                replacement_index: None,
            };
        }

        // Move the last element to the removed element's position
        self.get_ptr(index).expect("index out of bounds");
        self.copy_within(last_index, index);

        self.length -= 1;

        SwapRemoveResult {
            is_last: false,
            replacement_index: Some(last_index),
        }
    }

    #[inline]
    pub fn get_ptr(&self, index: usize) -> Option<NonNull<u8>> {
        if index >= self.length {
            None
        } else {
            unsafe {
                Some(NonNull::new_unchecked(
                    self.data.as_ptr().add(index * self.layout.size()),
                ))
            }
        }
    }

    pub fn reserve(&mut self, additional: usize) {
        let required_capacity = self.length + additional;
        if required_capacity > self.capacity {
            let new_capacity = (required_capacity * 2).max(self.capacity * 2);
            self.realloc(new_capacity);
        }
    }

    fn realloc(&mut self, new_capacity: usize) {
        debug_assert!(new_capacity > self.capacity);
        debug_assert!(self.layout.size() != 0);

        let old_layout = array_layout(&self.layout, self.capacity).expect("invalid layout");
        let new_layout = array_layout(&self.layout, new_capacity).expect("invalid layout");
        let new_data =
            unsafe { std::alloc::realloc(self.data.as_ptr(), old_layout, new_layout.size()) };

        self.data = NonNull::new(new_data).unwrap_or_else(|| handle_alloc_error(new_layout));
        self.capacity = new_capacity;
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Drop for AnyVec {
    fn drop(&mut self) {
        // Run Drop::drop for each element in the vector
        if let Some(drop_fn) = self.dropper {
            for i in 0..self.length {
                unsafe { drop_fn(self.get_ptr(i).expect("index out of bounds")) }
            }
        }

        if self.layout.size() != 0 {
            unsafe {
                std::alloc::dealloc(
                    self.data.as_ptr(),
                    array_layout(&self.layout, self.capacity).expect("invalid layout"),
                );
            }
        }
    }
}

// From https://github.com/bevyengine/bevy/blob/main/crates/bevy_ecs/src/storage/blob_vec.rs

/// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
fn array_layout(layout: &Layout, n: usize) -> Option<Layout> {
    let (array_layout, offset) = repeat_layout(layout, n)?;
    debug_assert_eq!(layout.size(), offset);
    Some(array_layout)
}

// TODO: replace with `Layout::repeat` if/when it stabilizes
/// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
fn repeat_layout(layout: &Layout, n: usize) -> Option<(Layout, usize)> {
    // This cannot overflow. Quoting from the invariant of Layout:
    // > `size`, when rounded up to the nearest multiple of `align`,
    // > must not overflow (i.e., the rounded value must be less than
    // > `usize::MAX`)
    let padded_size = layout.size() + padding_needed_for(layout, layout.align());
    let alloc_size = padded_size.checked_mul(n)?;

    // SAFETY: self.align is already known to be valid and alloc_size has been
    // padded already.
    unsafe {
        Some((
            Layout::from_size_align_unchecked(alloc_size, layout.align()),
            padded_size,
        ))
    }
}

/// From <https://doc.rust-lang.org/beta/src/core/alloc/layout.rs.html>
const fn padding_needed_for(layout: &Layout, align: usize) -> usize {
    let len = layout.size();

    // Rounded up value is:
    //   len_rounded_up = (len + align - 1) & !(align - 1);
    // and then we return the padding difference: `len_rounded_up - len`.
    //
    // We use modular arithmetic throughout:
    //
    // 1. align is guaranteed to be > 0, so align - 1 is always
    //    valid.
    //
    // 2. `len + align - 1` can overflow by at most `align - 1`,
    //    so the &-mask with `!(align - 1)` will ensure that in the
    //    case of overflow, `len_rounded_up` will itself be 0.
    //    Thus the returned padding, when added to `len`, yields 0,
    //    which trivially satisfies the alignment `align`.
    //
    // (Of course, attempts to allocate blocks of memory whose
    // size and padding overflow in the above manner should cause
    // the allocator to yield an error anyway.)

    let len_rounded_up = len.wrapping_add(align).wrapping_sub(1) & !align.wrapping_sub(1);
    len_rounded_up.wrapping_sub(len)
}

#[cfg(test)]
mod tests {
    use crate::storage::anyvec::AnyVec;

    #[test]
    fn insertions_and_deletions() {
        let mut v = super::AnyVec::new_of::<u32>(10);
        unsafe {
            v.push(1u32);
            v.push(2u32);
            v.push(3u32);
            assert_eq!(v.get::<u32>(0), Some(&1u32));
            assert_eq!(v.get::<u32>(1), Some(&2u32));
            assert_eq!(v.get::<u32>(2), Some(&3u32));
            assert_eq!(v.get::<u32>(3), None);
        };
    }

    #[test]
    fn capacity() {
        let mut v = super::AnyVec::new_of::<u32>(3);
        assert_eq!(v.capacity(), 3);
        unsafe {
            v.push(1u32);
            v.push(2u32);
            v.push(3u32);
        };
        assert_eq!(v.capacity(), 3);
        unsafe {
            v.push(4u32);
        };
        assert_eq!(v.capacity(), 8);
    }

    #[test]
    fn zero_sized() {
        #[derive(Debug, PartialEq)]
        struct ZeroSized;
        let mut v = AnyVec::new_of::<ZeroSized>(1);

        unsafe {
            v.push(ZeroSized);
            assert_eq!(v.layout.size(), 0);
            assert_eq!(v.layout.align(), 1);
            assert_eq!(v.len(), 1);
            assert_eq!(v.capacity(), usize::MAX);
            assert_eq!(v.get::<ZeroSized>(0), Some(&ZeroSized));
            assert_eq!(v.get::<ZeroSized>(1), None);
            assert_eq!(v.swap_remove(0).is_last, true);
            assert_eq!(v.len(), 0);
        };
    }
}
