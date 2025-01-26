use std::{any::Any, cell::RefCell, collections::HashMap, rc::Rc};

use acro_ecs::{ComponentId, EntityId, Tick, World};
use acro_reflect::{Reflect, ReflectExt, ReflectPath};
use deno_core::{
    serde_json::value,
    v8::{self, Global, Integer, Local, Number},
    OpState,
};
use rustyscript::{
    deno_core::{self, anyhow, error::AnyError, op2},
    js_value::Function,
};
use tracing::info;

use crate::{ops::get_dyn_reflect, platform::FunctionHandle, EventListenerStore, ScriptingRuntime};

#[op2]
#[string]
pub fn op_get_property_string(
    #[state] world: &Rc<RefCell<World>>,
    #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
    #[state] tick: &Tick,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
) -> Result<String, AnyError> {
    let path = ReflectPath::parse(path);
    let object = get_dyn_reflect(
        world,
        component_ids_to_vtables,
        tick,
        generation,
        index,
        component_id,
        false,
    )?;

    Ok(object.get::<String>(&path).clone())
}

#[op2(fast)]
#[string]
pub fn op_set_property_string(
    #[state] world: &Rc<RefCell<World>>,
    #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
    #[state] tick: &Tick,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
    #[string] value: String,
) -> Result<(), AnyError> {
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

    object.set::<String>(&path, value);

    Ok(())
}

#[op2(fast)]
pub fn op_get_property_number(
    #[state] world: &Rc<RefCell<World>>,
    #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
    #[state] tick: &Tick,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
) -> Result<f64, AnyError> {
    let path = ReflectPath::parse(path);
    let object = get_dyn_reflect(
        world,
        component_ids_to_vtables,
        tick,
        generation,
        index,
        component_id,
        false,
    )?;

    Ok(*object.get::<f32>(&path) as f64)
}

#[op2(fast)]
pub fn op_set_property_number(
    #[state] world: &Rc<RefCell<World>>,
    #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
    #[state] tick: &Tick,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
    value: f64,
) -> Result<(), AnyError> {
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

    object.set::<f32>(&path, value as f32);

    Ok(())
}

#[op2(fast)]
pub fn op_get_property_boolean(
    #[state] world: &Rc<RefCell<World>>,
    #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
    #[state] tick: &Tick,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
) -> Result<bool, AnyError> {
    let path = ReflectPath::parse(path);
    let object = get_dyn_reflect(
        world,
        component_ids_to_vtables,
        tick,
        generation,
        index,
        component_id,
        false,
    )?;

    Ok(*object.get::<bool>(&path))
}

#[op2(fast)]
pub fn op_set_property_boolean(
    #[state] world: &Rc<RefCell<World>>,
    #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
    #[state] tick: &Tick,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
    value: bool,
) -> Result<(), AnyError> {
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

    object.set::<bool>(&path, value);

    Ok(())
}

#[op2]
pub fn op_call_function<'s>(
    #[state] world: &Rc<RefCell<World>>,
    #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
    #[state] tick: &Tick,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
    scope: &'s mut v8::HandleScope,
    args: v8::Local<v8::Value>,
) -> Result<(), AnyError> {
    if !args.is_array() {
        panic!("passed args not array");
    }

    let array = args.try_cast::<v8::Array>()?;
    let mut items: Vec<Box<dyn Any>> = Vec::with_capacity(array.length() as usize);

    for i in 0..(array.length() as i32) {
        let index = Integer::new(scope, i).into();
        let item = array.get(scope, index).unwrap();

        if item.is_string() {
            items.push(Box::new(
                item.to_string(scope).unwrap().to_rust_string_lossy(scope),
            ));
        } else if item.is_number() {
            items.push(Box::new(item.to_number(scope).unwrap().value()));
        } else if item.is_boolean() {
            items.push(Box::new(item.to_boolean(scope).boolean_value(scope)));
        } else if item.is_function() {
            let world = world.borrow();
            let event_listener_store = world.resource::<EventListenerStore>();

            let value = Global::new(scope, item);
            let function = FunctionHandle::new_native(
                &event_listener_store,
                Function::try_from_v8(scope, value).expect("failed to convert function"),
            );
            items.push(Box::new(function));
        } else {
            todo!("got unsupported type");
        }
    }

    info!("path: {}, args: {:?}", path, items);

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

    object
        .call_method(&path, items)
        .expect("call_method failed");

    Ok(())
}
