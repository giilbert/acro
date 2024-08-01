use std::{
    borrow::Cow,
    cell::{RefCell, UnsafeCell},
    rc::Rc,
};

use acro_assets::Assets;
use acro_ecs::{Changed, EntityId, Query, Res, ResMut, SystemRunContext, World};
use acro_reflect::ReflectPath;
use deno_core::*;
use tracing::info;

use crate::{
    behavior::{Behavior, BehaviorData},
    source_file::SourceFile,
};

#[op2(fast)]
fn op_get_property_number(
    #[state] world: &Rc<RefCell<World>>,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
    value: f64,
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
    value: f64,
) -> Result<(), deno_core::error::AnyError> {
    todo!()
}

pub struct ScriptingRuntime {
    behavior_id: u32,
    runtime: JsRuntime,
}

impl std::fmt::Debug for ScriptingRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptingRuntime")
            .field("runtime", &"...")
            .finish()
    }
}

impl ScriptingRuntime {
    pub fn new(world_handle: Rc<RefCell<World>>) -> Self {
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
            .execute_script("<init>", include_str!("js/bootstrap.js"))
            .expect("failed to execute bootstrap script");

        Self {
            behavior_id: 0,
            runtime,
        }
    }

    pub fn init_source_file(&mut self, source_file: &SourceFile) -> eyre::Result<()> {
        info!("initializing source file: {:?}", source_file.config.name);

        self.runtime
            .execute_script("<source>", source_file.code.clone())
            .map_err(|e| eyre::eyre!("failed to execute source init script: {e:?}"))?;

        Ok(())
    }

    pub fn init_behavior(
        &mut self,
        source_file: &SourceFile,
        behavior: &mut Behavior,
    ) -> eyre::Result<()> {
        let id = self.behavior_id;
        self.behavior_id += 1;
        behavior.data = Some(BehaviorData { id });

        self.runtime
            .execute_script(
                "<create-behavior>",
                format!(
                    "acro.createBehavior({}, \"{}\")",
                    id, source_file.config.name
                ),
            )
            .map_err(|e| eyre::eyre!("failed to execute behavior init script: {e:?}"))?;

        Ok(())
    }
}

pub fn init_behavior(
    ctx: SystemRunContext,
    behaviors: Query<(EntityId, &mut Behavior), Changed<Behavior>>,
    assets: Res<Assets>,
    mut runtime: ResMut<ScriptingRuntime>,
) -> eyre::Result<()> {
    for (entity, mut behavior) in behaviors.over(&ctx) {
        let source_file = assets.get::<SourceFile>(&behavior.source_file_path);
        source_file.notify_changes::<Behavior>(&ctx, entity);
        runtime.init_behavior(&source_file, &mut behavior)?;
    }

    Ok(())
}
