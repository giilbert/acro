use std::any::Any;

// a_property.b_property.c_property
// => Property("a..", Property("b..", Property("c..", End)))
// Nested properties can be passsed down to further objects that are Reflect
pub enum ReflectPath<'a> {
    Property(&'a str, &'a ReflectPath<'a>),
    End,
}

pub trait Reflect {
    fn get_field_opt(&self, path: &ReflectPath) -> Option<&dyn Any>;

    fn get_field<T: 'static>(&self, path: &ReflectPath) -> &T {
        self.get_field_opt(path)
            .expect("field not found")
            .downcast_ref()
            .expect("type mismatch")
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use super::{Reflect, ReflectPath as R};

    struct Inner {
        b: u32,
    }

    impl Reflect for Inner {
        fn get_field_opt(&self, path: &R) -> Option<&dyn Any> {
            match path {
                R::End => Some(self),
                R::Property("b", R::End) => Some(&self.b),
                _ => None,
            }
        }
    }

    struct ReflectedStruct {
        a: u32,
        inner: Inner,
    }

    impl Reflect for ReflectedStruct {
        fn get_field_opt(&self, path: &R) -> Option<&dyn Any> {
            match path {
                R::End => Some(self),
                R::Property("a", R::End) => Some(&self.a),
                R::Property("b", rest) => self.inner.get_field_opt(rest),
                _ => None,
            }
        }
    }

    #[test]
    fn reflect_test() {}
}
