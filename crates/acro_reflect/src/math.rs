use std::any::Any;

use nalgebra::{self as na, Quaternion};

use crate::{type_mismatch, Reflect, ReflectPath, ReflectSetError};

macro_rules! impl_reflect_vector {
    ($type_name: ty: $($fields: expr),+ => $($fields_as_idents:ident),+) => {
        impl<F: Reflect + na::Scalar> Reflect for $type_name {
            fn get_field_names(&self) -> &'static [&'static str] {
                &[$($fields),+]
            }

            fn set_any(
                &mut self,
                path: &ReflectPath,
                data: Box<dyn Any>
            ) -> Result<(), ReflectSetError> {
                use ReflectPath::*;

                match path {
                    $(
                        Property($fields, rest) => self.$fields_as_idents.set_any(rest, data),
                    )+
                    End => {
                        *self = *data.downcast::<$type_name>().map_err(type_mismatch)?;
                        Ok(())
                    },
                    _ => Err(ReflectSetError::PathNotFound),
                }
            }

            fn get_opt(&self, path: &ReflectPath) -> Option<&dyn Any> {
                use ReflectPath::*;
                match path {
                    $(
                        Property($fields, rest) => self.$fields_as_idents.get_opt(rest),
                    )+
                    End => Some(self),
                    _ => None,
                }
            }
        }
    };
}

impl_reflect_vector!(na::Vector2<F>: "x", "y" => x, y);
impl_reflect_vector!(na::Vector3<F>: "x", "y", "z" => x, y, z);
impl_reflect_vector!(na::Vector4<F>: "x", "y", "z", "w" => x, y, z, w);

impl<F: Reflect + na::Scalar> Reflect for Quaternion<F> {
    fn get_field_names(&self) -> &'static [&'static str] {
        &["x", "y", "z", "w"]
    }

    fn get_opt(&self, path: &ReflectPath) -> Option<&dyn Any> {
        use ReflectPath::*;

        match path {
            Property("x", rest) => self.coords.x.get_opt(rest),
            Property("y", rest) => self.coords.y.get_opt(rest),
            Property("z", rest) => self.coords.z.get_opt(rest),
            Property("w", rest) => self.coords.w.get_opt(rest),
            End => Some(self),
            _ => None,
        }
    }

    fn set_any(&mut self, path: &ReflectPath, data: Box<dyn Any>) -> Result<(), ReflectSetError> {
        use ReflectPath::*;

        match path {
            Property("x", rest) => self.coords.x.set_any(rest, data),
            Property("y", rest) => self.coords.y.set_any(rest, data),
            Property("z", rest) => self.coords.z.set_any(rest, data),
            Property("w", rest) => self.coords.w.set_any(rest, data),
            End => {
                *self = *data.downcast::<Quaternion<F>>().map_err(type_mismatch)?;
                Ok(())
            }
            _ => return Err(ReflectSetError::PathNotFound),
        }
    }
}
