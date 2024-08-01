use std::{
    borrow::Cow,
    cell::{RefCell, UnsafeCell},
    rc::Rc,
};

use acro_ecs::World;
use acro_reflect::ReflectPath;
use deno_core::*;

#[op2(fast)]
fn op_get_property_number(
    #[state] world: &Rc<RefCell<World>>,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
    value: f32,
) -> Result<f64, deno_core::error::AnyError> {
    let path = ReflectPath::parse(path);
    todo!()
}

#[op2(fast)]
fn op_set_property_number(
    #[state] world: &Rc<RefCell<World>>,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
    value: f32,
) -> Result<f64, deno_core::error::AnyError> {
    todo!()
}

#[derive(Debug)]
pub struct ScriptingRuntime {}

impl ScriptingRuntime {
    pub fn new(world_handle: Rc<RefCell<World>>) {
        const GET_PROPERTY_NUMBER: OpDecl = op_get_property_number();

        let ext = Extension {
            name: "components",
            ops: Cow::Borrowed(&[GET_PROPERTY_NUMBER]),
            ..Default::default()
        };

        let mut runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![ext],
            ..Default::default()
        });

        runtime.op_state().borrow_mut().put(world_handle);

        runtime
            .execute_script("<usage>", "Deno.core.print(\"hello world\\n\");")
            .unwrap();
    }
}
