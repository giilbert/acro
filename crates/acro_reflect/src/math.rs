use std::any::Any;

use nalgebra::{self as na, Quaternion};

use crate::{Reflect, ReflectPath, ReflectSetError};

impl<F: Reflect + na::Scalar> Reflect for na::Vector3<F> {
    fn get_field_names(&self) -> &'static [&'static str] {
        &["x", "y", "z"]
    }

    fn get_opt(&self, path: &ReflectPath) -> Option<&dyn Any> {
        use ReflectPath::*;

        match path {
            Property("x", rest) => self.x.get_opt(rest),
            Property("y", rest) => self.y.get_opt(rest),
            Property("z", rest) => self.z.get_opt(rest),
            _ => None,
        }
    }

    fn set_any(&mut self, path: &ReflectPath, data: Box<dyn Any>) -> Result<(), ReflectSetError> {
        use ReflectPath::*;

        match path {
            Property("x", rest) => self.x.set_any(rest, data),
            Property("y", rest) => self.y.set_any(rest, data),
            Property("z", rest) => self.z.set_any(rest, data),
            _ => return Err(ReflectSetError::PathNotFound),
        }
    }
}

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
            _ => return Err(ReflectSetError::PathNotFound),
        }
    }
}
