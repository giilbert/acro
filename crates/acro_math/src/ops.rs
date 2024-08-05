use std::{cell::RefCell, rc::Rc};

use acro_ecs::{EntityId, World};
use deno_core::op2;

use crate::tree::WorldTreeExt;

#[op2]
#[serde]
pub fn op_get_entity_by_absolute_path(
    #[state] world: &Rc<RefCell<World>>,
    #[string] path: &str,
) -> Option<EntityId> {
    world.borrow().get_entity_by_absolute_path(&path)
}
