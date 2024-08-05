use std::any::Any;

use crate::type_mismatch;

// a_property.b_property.c_property
// => Property("a..", Property("b..", Property("c..", End)))
// Nested properties can be passsed down to further objects that are Reflect
#[derive(Debug)]
pub enum ReflectPath<'a> {
    /// Refers to the item at the current path
    End,
    /// Refers to a property of the item at the current path
    Property(&'a str, Box<ReflectPath<'a>>),
}

impl ReflectPath<'_> {
    pub fn parse<'a>(path: &'a str) -> Box<ReflectPath<'a>> {
        let mut parts = path.split('.').rev();
        let mut path = Box::new(ReflectPath::End);
        while let Some(part) = parts.next() {
            if part == "" {
                continue;
            }
            let new_path = Box::new(ReflectPath::Property(part, path));
            path = new_path;
        }

        path
    }
}

#[derive(Debug)]
pub enum ReflectSetError {
    TypeMismatch,
    PathNotFound,
}

pub trait Reflect {
    fn get_field_names(&self) -> &'static [&'static str];

    fn set_any(&mut self, path: &ReflectPath, data: Box<dyn Any>) -> Result<(), ReflectSetError>;
    fn get_opt(&self, path: &ReflectPath) -> Option<&dyn Any>;

    fn get_full_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn get_name(&self) -> &'static str {
        self.get_full_name()
            .split("::")
            .last()
            .expect("self.get_name() returned nothing")
    }
}

/// Trait to extend Reflect with some convenience methods
pub trait ReflectExt: Reflect {
    fn get<T: 'static>(&self, path: &ReflectPath) -> &T {
        self.get_opt(path)
            .unwrap_or_else(|| panic!("field {path:?} not found on {}", self.get_name()))
            .downcast_ref()
            .expect("type mismatch")
    }

    fn set<T: 'static>(&mut self, path: &ReflectPath, value: T) {
        self.set_any(path, Box::new(value))
            .expect("error setting value");
    }

    fn downcast<T: 'static>(&self) -> &T {
        self.get(&ReflectPath::End)
    }
}

impl<T: Reflect + ?Sized> ReflectExt for T {}

macro_rules! impl_reflect_prim {
    ($integer_type:ident) => {
        impl Reflect for $integer_type {
            fn get_field_names(&self) -> &'static [&'static str] {
                &[]
            }

            fn set_any(
                &mut self,
                path: &ReflectPath,
                data: Box<dyn Any>,
            ) -> Result<(), ReflectSetError> {
                match path {
                    ReflectPath::End => *self = *data.downcast().map_err(type_mismatch)?,
                    _ => return Err(ReflectSetError::PathNotFound),
                }
                Ok(())
            }

            fn get_opt(&self, path: &ReflectPath) -> Option<&dyn Any> {
                match path {
                    ReflectPath::End => Some(self),
                    _ => None,
                }
            }
        }
    };
}

impl_reflect_prim!(u8);
impl_reflect_prim!(u16);
impl_reflect_prim!(u32);
impl_reflect_prim!(u64);
impl_reflect_prim!(u128);
impl_reflect_prim!(usize);
impl_reflect_prim!(i8);
impl_reflect_prim!(i16);
impl_reflect_prim!(i32);
impl_reflect_prim!(i64);
impl_reflect_prim!(i128);
impl_reflect_prim!(isize);
impl_reflect_prim!(f32);
impl_reflect_prim!(f64);
impl_reflect_prim!(String);

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::{Reflect, ReflectExt, ReflectPath as R, ReflectSetError};

    #[derive(Debug, PartialEq, Eq)]
    struct Inner {
        b: u32,
    }

    impl Reflect for Inner {
        fn set_any(&mut self, path: &R, data: Box<dyn Any>) -> Result<(), ReflectSetError> {
            match path {
                R::End => *self = *data.downcast().map_err(|_| ReflectSetError::TypeMismatch)?,
                R::Property("b", path) => self.b.set_any(&path, data)?,
                _ => return Err(ReflectSetError::PathNotFound),
            }
            Ok(())
        }

        fn get_opt(&self, path: &R) -> Option<&dyn Any> {
            match path {
                R::End => Some(self),
                R::Property("b", path) => self.b.get_opt(path),
                _ => None,
            }
        }

        fn get_field_names(&self) -> &'static [&'static str] {
            &["b"]
        }
    }

    #[derive(Debug, PartialEq, Eq)]
    struct ReflectedStruct {
        a: u32,
        inner: Inner,
    }

    impl Reflect for ReflectedStruct {
        fn set_any(&mut self, path: &R, data: Box<dyn Any>) -> Result<(), ReflectSetError> {
            match path {
                R::End => *self = *data.downcast().map_err(|_| ReflectSetError::TypeMismatch)?,
                R::Property("a", path) => self.a.set_any(path, data)?,
                R::Property("inner", path) => self.inner.set_any(path, data)?,
                _ => return Err(ReflectSetError::PathNotFound),
            }
            Ok(())
        }

        fn get_opt(&self, path: &R) -> Option<&dyn Any> {
            match path {
                R::End => Some(self),
                R::Property("a", path) => self.a.get_opt(path),
                R::Property("inner", rest) => self.inner.get_opt(rest),
                _ => None,
            }
        }

        fn get_field_names(&self) -> &'static [&'static str] {
            &["a", "inner"]
        }
    }

    #[test]
    fn reflect_test() {
        let mut test = ReflectedStruct {
            a: 1,
            inner: Inner { b: 1 },
        };

        test.set_any(&R::Property("a", Box::new(R::End)), Box::new(2u32))
            .expect("error setting a");
        assert_eq!(*test.get::<u32>(&R::Property("a", Box::new(R::End))), 2);

        test.set_any(
            &R::Property("inner", Box::new(R::End)),
            Box::new(Inner { b: 3 }),
        )
        .expect("error setting inner");
        assert_eq!(
            *test.get::<Inner>(&R::Property("inner", Box::new(R::End))),
            Inner { b: 3 }
        );

        assert_eq!(test.get_name(), "ReflectedStruct");
    }
}
