use std::{cell::RefCell, collections::HashMap, rc::Rc};

use acro_ecs::{ComponentId, Tick, World};
use acro_reflect::{ReflectExt, ReflectPath};
#[cfg(target_arch = "wasm32")]
use acro_scripting::wasm_ops;
use acro_scripting::{eyre_to_any_error, get_dyn_reflect};
#[cfg(not(target_arch = "wasm32"))]
use deno_core::op2;
use nalgebra as na;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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

#[cfg(not(target_arch = "wasm32"))]
macro_rules! set_vector_op {
    ($name: ident, $vector_type: ty, $($fields: ident),+) => {
        #[op2(fast)]
        pub fn $name(
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
            )
            .map_err(eyre_to_any_error)?;

            object.set::<$vector_type>(&path, <$vector_type>::new($($fields as Float),+));

            Ok(())
        }
    };
}

#[cfg(not(target_arch = "wasm32"))]
macro_rules! get_vector_op {
    ($name: ident, $vector_type: ty; $new_type_name: ident: $($new_type_fields: ident),+) => {
        #[derive(serde::Serialize)]
        struct $new_type_name {
            $($new_type_fields: Float),+
        }

        #[op2]
        #[serde]
        pub fn $name(
            #[state] world: &Rc<RefCell<World>>,
            #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
            #[state] tick: &Tick,
            generation: u32,
            index: u32,
            component_id: u32,
            #[string] path: &str,
        ) -> Result<$new_type_name, deno_core::error::AnyError> {
            let path = ReflectPath::parse(path);

            let object = get_dyn_reflect(
                world,
                component_ids_to_vtables,
                tick,
                generation,
                index,
                component_id,
                true,
            )
            .map_err(eyre_to_any_error)?;

            let data = *object.get::<$vector_type>(&path);

            Ok($new_type_name {
                $($new_type_fields: data.$new_type_fields),+
            })
        }
    };
}

#[cfg(target_arch = "wasm32")]
macro_rules! set_vector_op {
    ($name: ident, $vector_type: ty, $($fields: ident),+) => {
        #[wasm_bindgen]
        pub fn $name(
            generation: u32,
            index: u32,
            component_id: u32,
            path: &str,
            $($fields: f64),+
        ) -> Result<(), wasm_bindgen::JsError> {
            let path = ReflectPath::parse(path);

            let (world, component_ids_to_vtables, tick) = wasm_ops::get_ecs_state();
            let object = get_dyn_reflect(
                world,
                component_ids_to_vtables,
                tick,
                generation,
                index,
                component_id,
                true,
            )
            .map_err(wasm_ops::into_js_error)?;

            object.set::<$vector_type>(&path, <$vector_type>::new($($fields as Float),+));

            Ok(())
        }
    };
}

#[cfg(target_arch = "wasm32")]
macro_rules! get_vector_op {
    ($name: ident, $vector_type: ty; $new_type_name: ident: $($new_type_fields: ident),+) => {
        #[derive(serde::Serialize)]
        struct $new_type_name {
            $($new_type_fields: Float),+
        }

        #[wasm_bindgen]
        pub fn $name(
            generation: u32,
            index: u32,
            component_id: u32,
            path: &str,
        ) -> Result<JsValue, wasm_bindgen::JsError> {
            let path = ReflectPath::parse(path);

            let (world, component_ids_to_vtables, tick) = wasm_ops::get_ecs_state();
            let object = get_dyn_reflect(
                world,
                component_ids_to_vtables,
                tick,
                generation,
                index,
                component_id,
                true,
            )
            .map_err(wasm_ops::into_js_error)?;

            let data = *object.get::<$vector_type>(&path);

            Ok(serde_wasm_bindgen::to_value(&$new_type_name {
                $($new_type_fields: data.$new_type_fields),+
            })?)
        }
    };
}

set_vector_op!(op_set_property_vec2, Vec2, x, y);
set_vector_op!(op_set_property_vec3, Vec3, x, y, z);
set_vector_op!(op_set_property_vec4, Vec4, x, y, z, w);
get_vector_op!(op_get_property_vec2, Vec2; Vec2NewType: x, y);
get_vector_op!(op_get_property_vec3, Vec3; Vec3NewType: x, y, z);
get_vector_op!(op_get_property_vec4, Vec4; Vec4NewType: x, y, z, w);
