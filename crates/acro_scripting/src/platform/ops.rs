use std::{cell::RefCell, collections::HashMap, rc::Rc};

use acro_ecs::{ComponentId, EntityId, Tick, World};
use acro_reflect::Reflect;

pub fn get_dyn_reflect<'a>(
    world: &Rc<RefCell<World>>,
    component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
    tick: &Tick,
    generation: u32,
    index: u32,
    component_id: u32,
    notify_change_detection: bool,
) -> eyre::Result<&'a mut dyn Reflect> {
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
        .ok_or_else(|| eyre::eyre!("entity or component not found"))?;

    Ok(unsafe {
        std::mem::transmute::<(*const (), *const ()), &mut dyn Reflect>((
            data_ptr.as_ptr() as *const (),
            component_ids_to_vtables[&ComponentId(component_id)] as *const (),
        ))
    })
}
