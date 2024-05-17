use std::any::Any;

#[allow(unused)]
pub(crate) trait QueryInfoUtils: Any {
    fn is_borrowed() -> bool {
        false
    }

    fn is_borrowed_mut() -> bool {
        false
    }
}

impl<T: 'static> QueryInfoUtils for &'static T {
    fn is_borrowed() -> bool {
        true
    }
}

impl<T: 'static> QueryInfoUtils for &'static mut T {
    fn is_borrowed_mut() -> bool {
        true
    }
}
