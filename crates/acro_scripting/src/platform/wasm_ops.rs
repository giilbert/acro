use std::{
    any::{Any, TypeId},
    cell::{LazyCell, RefCell},
    collections::HashMap,
    rc::Rc,
};

use acro_ecs::{ComponentId, Tick, World};
use acro_reflect::{ReflectExt, ReflectPath};
use eyre::OptionExt;
use fnv::FnvHashMap;
use js_sys::{Boolean, Function, JsString, Number};
use wasm_bindgen::prelude::*;

use crate::{platform::FunctionHandle, EventListenerStore};

use super::ops::get_dyn_reflect;

pub static mut WASM_OPS_STATE: LazyCell<WasmOpsState> = LazyCell::new(|| WasmOpsState::default());

#[derive(Default)]
pub struct WasmOpsState {
    storage: FnvHashMap<TypeId, Box<dyn Any>>,
}

impl WasmOpsState {
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.storage
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref())
    }

    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.storage
            .get_mut(&TypeId::of::<T>())
            .and_then(|v| v.downcast_mut())
    }

    pub fn insert<T: Any>(&mut self, value: T) {
        self.storage.insert(TypeId::of::<T>(), Box::new(value));
    }
}

pub fn get_ecs_state() -> (
    &'static Rc<RefCell<World>>,
    &'static HashMap<ComponentId, *const ()>,
    &'static Tick,
) {
    unsafe {
        (
            WASM_OPS_STATE.get().unwrap(),
            WASM_OPS_STATE.get().unwrap(),
            WASM_OPS_STATE.get().unwrap(),
        )
    }
}

type JsResult<T> = Result<T, JsError>;

pub fn into_js_error(error: impl std::fmt::Debug) -> JsError {
    JsError::new(&format!("{:?}", error))
}

#[wasm_bindgen]
pub fn op_get_property_number(
    generation: u32,
    index: u32,
    component_id: u32,
    path: &str,
) -> JsResult<f64> {
    let (world, component_ids_to_vtables, tick) = get_ecs_state();
    let path = ReflectPath::parse(path);
    let object = get_dyn_reflect(
        world,
        component_ids_to_vtables,
        tick,
        generation,
        index,
        component_id,
        false,
    )
    .map_err(into_js_error)?;

    Ok(*object.get::<f32>(&path) as f64)
}

#[wasm_bindgen]
pub fn op_set_property_number(
    generation: u32,
    index: u32,
    component_id: u32,
    path: &str,
    value: f64,
) -> JsResult<()> {
    let path = ReflectPath::parse(path);

    let (world, component_ids_to_vtables, tick) = get_ecs_state();
    let object = get_dyn_reflect(
        world,
        component_ids_to_vtables,
        tick,
        generation,
        index,
        component_id,
        true,
    )
    .map_err(into_js_error)?;

    object.set::<f32>(&path, value as f32);

    Ok(())
}

#[wasm_bindgen]
pub fn op_get_property_boolean(
    generation: u32,
    index: u32,
    component_id: u32,
    path: &str,
) -> JsResult<bool> {
    let path = ReflectPath::parse(path);
    let (world, component_ids_to_vtables, tick) = get_ecs_state();
    let object = get_dyn_reflect(
        world,
        component_ids_to_vtables,
        tick,
        generation,
        index,
        component_id,
        false,
    )
    .map_err(into_js_error)?;

    Ok(*object.get::<bool>(&path))
}

#[wasm_bindgen]
pub fn op_set_property_boolean(
    generation: u32,
    index: u32,
    component_id: u32,
    path: &str,
    value: bool,
) -> JsResult<()> {
    let path = ReflectPath::parse(path);

    let (world, component_ids_to_vtables, tick) = get_ecs_state();
    let object = get_dyn_reflect(
        world,
        component_ids_to_vtables,
        tick,
        generation,
        index,
        component_id,
        true,
    )
    .map_err(into_js_error)?;

    object.set::<bool>(&path, value);

    Ok(())
}

#[wasm_bindgen]
pub fn op_call_function(
    generation: u32,
    index: u32,
    component_id: u32,
    path: &str,
    args: JsValue,
) -> JsResult<()> {
    let (world, component_ids_to_vtables, tick) = get_ecs_state();

    if !args.is_array() {
        panic!("passed args not array");
    }

    let array = js_sys::Array::from(&args);
    let mut items: Vec<Box<dyn Any>> = Vec::with_capacity(array.length() as usize);

    for index in 0..(array.length() as u32) {
        let item = array.get(index);

        if let Some(string) = item.dyn_ref::<JsString>() {
            items.push(Box::new(
                string.as_string().expect("failed to convert string"),
            ));
        } else if let Some(number) = item.dyn_ref::<Number>() {
            items.push(Box::new(number.as_f64().expect("failed to convert number")));
        } else if let Some(boolean) = item.dyn_ref::<Boolean>() {
            items.push(Box::new(
                boolean.as_bool().expect("failed to convert boolean"),
            ));
        } else if let Some(function) = item.dyn_ref::<Function>() {
            let world = world.borrow();
            let event_listener_store = world.resource::<EventListenerStore>();

            let function = FunctionHandle::new_wasm(&event_listener_store, function.clone());
            items.push(Box::new(function));
        } else {
            todo!("got unsupported type");
        }
    }

    tracing::info!("path: {}, args: {:?}", path, items);

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
    .map_err(into_js_error)?;

    object
        .call_method(&path, items)
        .expect("call_method failed");

    Ok(())
}
