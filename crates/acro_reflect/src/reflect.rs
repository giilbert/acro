use std::any::Any;

use crate::type_mismatch;

// a_property.b_property.c_property
// => Property("a..", Property("b..", Property("c..", End)))
// Nested properties can be passsed down to further objects that are Reflect
#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug)]
pub enum ReflectFunctionCallError {
    ArgumentTypeMismatch(u8),
    MissingArgument(u8),
    PathNotFound,
}

pub trait Reflect {
    fn get_field_names(&self) -> &'static [&'static str];

    fn set_any(&mut self, path: &ReflectPath, data: Box<dyn Any>) -> Result<(), ReflectSetError>;
    fn get_opt(&self, path: &ReflectPath) -> Option<&dyn Any>;

    fn call_method(
        &self,
        _path: &ReflectPath,
        _arguments: Vec<Box<dyn Any>>,
    ) -> Result<Option<Box<dyn Any>>, ReflectFunctionCallError> {
        Ok(None)
    }

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
impl_reflect_prim!(bool);

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::{Reflect, ReflectExt, ReflectFunctionCallError, ReflectPath as R, ReflectSetError};

    #[derive(Debug, PartialEq, Eq)]
    struct Inner {
        b: u32,
    }

    impl Inner {
        pub fn get_b_times_two(&self) -> u32 {
            self.b * 2
        }

        pub fn multiply_by_b(&self, rhs: u32) -> u32 {
            self.b * rhs
        }
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

        fn call_method(
            &self,
            path: &R,
            arguments: Vec<Box<dyn Any>>,
        ) -> Result<Option<Box<dyn Any>>, ReflectFunctionCallError> {
            match path {
                R::Property("get_b_times_two", path) if **path == R::End => {
                    Ok(Some(Box::new(self.get_b_times_two())))
                }
                R::Property("multiply_by_b", path) if **path == R::End => {
                    let mut arguments = arguments.into_iter();
                    let arg_0 = arguments
                        .next()
                        .ok_or_else(|| ReflectFunctionCallError::MissingArgument(0))?
                        .downcast()
                        .map_err(|_| ReflectFunctionCallError::ArgumentTypeMismatch(0))?;

                    Ok(Some(Box::new(self.multiply_by_b(*arg_0))))
                }
                _ => Err(ReflectFunctionCallError::PathNotFound),
            }
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

        fn call_method(
            &self,
            path: &R,
            arguments: Vec<Box<dyn Any>>,
        ) -> Result<Option<Box<dyn Any>>, ReflectFunctionCallError> {
            match path {
                R::Property("inner", path) => self.inner.call_method(path, arguments),
                _ => Err(ReflectFunctionCallError::PathNotFound),
            }
        }
    }

    #[test]
    fn reflect_test() {
        let mut test = ReflectedStruct {
            a: 1,
            inner: Inner { b: 1 },
        };

        test.set_any(&R::parse("a"), Box::new(2u32))
            .expect("error setting a");
        assert_eq!(*test.get::<u32>(&R::parse("a")), 2);

        test.set_any(&R::parse("inner"), Box::new(Inner { b: 3 }))
            .expect("error setting inner");
        assert_eq!(*test.get::<Inner>(&R::parse("inner")), Inner { b: 3 });

        assert_eq!(test.get_name(), "ReflectedStruct");

        assert_eq!(
            *test
                .call_method(&R::parse("inner.get_b_times_two"), vec![])
                .expect("error calling function")
                .expect("function should return something")
                .downcast::<u32>()
                .expect("type mismatch"),
            6u32
        );

        assert_eq!(
            *test
                .call_method(&R::parse("inner.multiply_by_b"), vec![Box::new(4u32)])
                .expect("error calling function")
                .expect("function should return something")
                .downcast::<u32>()
                .expect("type mismatch"),
            12u32
        );
    }
}
