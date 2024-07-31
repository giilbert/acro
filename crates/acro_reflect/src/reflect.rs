use std::any::Any;

// a_property.b_property.c_property
// => Property("a..", Property("b..", Property("c..", End)))
// Nested properties can be passsed down to further objects that are Reflect
#[derive(Debug)]
pub enum ReflectPath<'a> {
    /// Refers to the item at the current path
    End,
    /// Refers to a property of the item at the current path
    Property(&'a str, &'a ReflectPath<'a>),
}

#[derive(Debug)]
pub enum ReflectSetError {
    TypeMismatch,
    PathNotFound,
}

pub trait Reflect
where
    Self: Sized,
{
    fn set(&mut self, path: &ReflectPath, data: Box<dyn Any>) -> Result<(), ReflectSetError>;
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

    fn get_field_names(&self) -> &'static [&'static str] {
        &[]
    }

    fn get<T: 'static>(&self, path: &ReflectPath) -> &T {
        self.get_opt(path)
            .unwrap_or_else(|| panic!("field {path:?} not found"))
            .downcast_ref()
            .expect("type mismatch")
    }

    fn downcast<T: 'static>(&self) -> &T {
        self.get(&ReflectPath::End)
    }
}

fn type_mismatch<T>(_: T) -> ReflectSetError {
    ReflectSetError::TypeMismatch
}

macro_rules! impl_reflect_integer {
    ($integer_type:ident) => {
        impl Reflect for $integer_type {
            fn set(
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

impl_reflect_integer!(u8);
impl_reflect_integer!(u16);
impl_reflect_integer!(u32);
impl_reflect_integer!(u64);
impl_reflect_integer!(u128);
impl_reflect_integer!(usize);
impl_reflect_integer!(i8);
impl_reflect_integer!(i16);
impl_reflect_integer!(i32);
impl_reflect_integer!(i64);
impl_reflect_integer!(i128);
impl_reflect_integer!(isize);

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::{Reflect, ReflectPath as R, ReflectSetError};

    #[derive(Debug, PartialEq, Eq)]
    struct Inner {
        b: u32,
    }

    impl Reflect for Inner {
        fn set(&mut self, path: &R, data: Box<dyn Any>) -> Result<(), ReflectSetError> {
            match path {
                R::End => *self = *data.downcast().map_err(|_| ReflectSetError::TypeMismatch)?,
                R::Property("b", R::End) => {
                    self.b = *data.downcast().map_err(|_| ReflectSetError::TypeMismatch)?
                }
                _ => return Err(ReflectSetError::PathNotFound),
            }
            Ok(())
        }

        fn get_opt(&self, path: &R) -> Option<&dyn Any> {
            match path {
                R::End => Some(self),
                R::Property("b", R::End) => Some(&self.b),
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
        fn set(&mut self, path: &R, data: Box<dyn Any>) -> Result<(), ReflectSetError> {
            match path {
                R::End => *self = *data.downcast().map_err(|_| ReflectSetError::TypeMismatch)?,
                R::Property("a", R::End) => {
                    self.a = *data.downcast().map_err(|_| ReflectSetError::TypeMismatch)?
                }
                R::Property("inner", path) => self.inner.set(path, data)?,
                _ => return Err(ReflectSetError::PathNotFound),
            }
            Ok(())
        }

        fn get_opt(&self, path: &R) -> Option<&dyn Any> {
            match path {
                R::End => Some(self),
                R::Property("a", R::End) => Some(&self.a),
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

        test.set(&R::Property("a", &R::End), Box::new(2u32))
            .expect("error setting a");
        assert_eq!(*test.get::<u32>(&R::Property("a", &R::End)), 2);

        test.set(&R::Property("inner", &R::End), Box::new(Inner { b: 3 }))
            .expect("error setting inner");
        assert_eq!(
            *test.get::<Inner>(&R::Property("inner", &R::End)),
            Inner { b: 3 }
        );

        assert_eq!(test.get_name(), "ReflectedStruct");
    }
}
