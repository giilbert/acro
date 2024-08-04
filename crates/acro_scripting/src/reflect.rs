use std::{cell::RefCell, collections::HashMap, rc::Rc};

use acro_ecs::{ComponentId, EntityId, Tick, World};
use acro_reflect::{Reflect, ReflectExt, ReflectPath};
use rustyscript::deno_core::{self, anyhow, error::AnyError, op2};
use tracing::info;

pub fn get_dyn_reflect<'a>(
    world: &Rc<RefCell<World>>,
    component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
    tick: &Tick,
    generation: u32,
    index: u32,
    component_id: u32,
    notify_change_detection: bool,
) -> Result<&'a mut dyn Reflect, AnyError> {
    let entity = EntityId::new(generation, index);
    let component = ComponentId(component_id);

    let data_ptr = world
        .borrow()
        .get_ptr(
            entity,
            component,
            if notify_change_detection {
                Some(*tick)
            } else {
                None
            },
        )
        .ok_or_else(|| anyhow::anyhow!("entity or component not found"))?;

    Ok(unsafe {
        std::mem::transmute::<(*const (), *const ()), &mut dyn Reflect>((
            data_ptr.as_ptr() as *const (),
            component_ids_to_vtables[&ComponentId(component_id)] as *const (),
        ))
    })
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
