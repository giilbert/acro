mod math;
mod reflect;

pub use reflect::{Reflect, ReflectExt, ReflectFunctionCallError, ReflectPath, ReflectSetError};
pub use reflect_derive::Reflect;

pub fn type_mismatch<T>(_: T) -> ReflectSetError {
    ReflectSetError::TypeMismatch
}
