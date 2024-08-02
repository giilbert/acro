use std::{cell::RefCell, collections::HashMap, rc::Rc};

use acro_ecs::{ComponentId, EntityId, Tick, World};
use acro_reflect::{Reflect, ReflectExt, ReflectPath};
use acro_scripting::get_dyn_reflect;
use deno_core::op2;
use nalgebra as na;

cfg_if::cfg_if! {
    if #[cfg(feature = "double-precision")] {
        pub type Float = f64;
    } else {
        pub type Float = f32;
    }
}

pub type Vec2 = na::Vector2<Float>;
pub type Vec3 = na::Vector3<Float>;
pub type Vec4 = na::Vector4<Float>;

pub type Mat2 = na::Matrix2<Float>;
pub type Mat3 = na::Matrix3<Float>;
pub type Mat4 = na::Matrix4<Float>;

pub type Quaternion = na::Quaternion<Float>;

macro_rules! set_vector_op {
    ($name: ident, $vector_type: ty, $($fields: ident),+) => {
        #[op2(fast)]
        fn $name(
            #[state] world: &Rc<RefCell<World>>,
            #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
            #[state] tick: &Tick,
            generation: u32,
            index: u32,
            component_id: u32,
            #[string] path: &str,
            $($fields: f64),+
        ) -> Result<(), deno_core::error::AnyError> {
            let path = ReflectPath::parse(path);

            let object = get_dyn_reflect(
                world,
                component_ids_to_vtables,
                tick,
                generation,
                index,
                component_id,
                true,
            )?;

            object.set::<$vector_type>(&path, <$vector_type>::new($($fields as Float),+));

            Ok(())
        }
    };
}

set_vector_op!(op_set_property_vector2, Vec2, x, y);
set_vector_op!(op_set_property_vector3, Vec3, x, y, z);
set_vector_op!(op_set_property_vector4, Vec4, x, y, z, w);

macro_rules! get_vector_op {
    ($name: ident, $vector_type: ty) => {
        #[op2]
        fn $name(
            #[state] world: &Rc<RefCell<World>>,
            #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
            #[state] tick: &Tick,
            generation: u32,
            index: u32,
            component_id: u32,
            #[string] path: &str,
        ) -> Result<$vector_type, deno_core::error::AnyError> {
            let path = ReflectPath::parse(path);

            let object = get_dyn_reflect(
                world,
                component_ids_to_vtables,
                tick,
                generation,
                index,
                component_id,
                true,
            )?;

            Ok(*object.get::<$vector_type>(&path))
        }
    };
}

get_vector_op!(op_get_property_vector2, Vec2);
// get_vector_op!(op_get_property_vector3, Vec3);
// get_vector_op!(op_get_property_vector4, Vec4);
