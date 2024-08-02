use std::{
    borrow::Cow,
    cell::{RefCell, UnsafeCell},
    collections::HashMap,
    rc::Rc,
    time::Instant,
};

use acro_assets::Assets;
use acro_ecs::{Changed, ComponentId, EntityId, Query, Res, ResMut, SystemRunContext, World};
use acro_reflect::{Reflect, ReflectExt, ReflectPath};
use deno_core::*;
use tracing::info;

use crate::{
    behavior::{Behavior, BehaviorData},
    source_file::SourceFile,
};

#[op2(fast)]
fn op_get_property_number(
    #[state] world: &Rc<RefCell<World>>,
    #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
) -> Result<f64, deno_core::error::AnyError> {
    let path = ReflectPath::parse(path);
    let data_ptr = world
        .borrow()
        .get_ptr(EntityId::new(generation, index), ComponentId(component_id))
        .ok_or_else(|| deno_core::anyhow::anyhow!("entity or component not found"))?;

    let trait_object = unsafe {
        std::mem::transmute::<(*const (), *const ()), &dyn Reflect>((
            data_ptr.as_ptr() as *const (),
            component_ids_to_vtables[&ComponentId(component_id)] as *const (),
        ))
    };

    Ok(*trait_object.get::<f32>(&path) as f64)
}

#[op2(fast)]
fn op_set_property_number(
    #[state] world: &Rc<RefCell<World>>,
    #[state] component_ids_to_vtables: &HashMap<ComponentId, *const ()>,
    generation: u32,
    index: u32,
    component_id: u32,
    #[string] path: &str,
    value: f64,
) -> Result<(), deno_core::error::AnyError> {
    todo!()
}

pub struct ScriptingRuntime {
    inner: JsRuntime,
    behavior_id: u32,
    component_vtables: Option<HashMap<ComponentId, *const ()>>,
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
            inner: runtime,
            component_vtables: Some(HashMap::new()),
        }
    }

    pub fn register_component<T: Reflect + 'static>(
        &mut self,
        world: &World,
        name: &str,
    ) -> eyre::Result<()> {
        let (_data, vtable_ptr) = unsafe {
            std::mem::transmute::<&dyn Reflect, (*const (), *const ())>(
                (&std::mem::MaybeUninit::<T>::uninit().assume_init()) as &dyn Reflect,
            )
        };

        let component_info = world.get_component_info::<T>();
        self.component_vtables
            .as_mut()
            .unwrap()
            .insert(component_info.id, vtable_ptr);

        let component_id = component_info.id.0;

        self.inner
            .execute_script(
                "<register-component>",
                format!("acro.COMPONENT_IDS[\"{}\"] = {};", name, component_id),
            )
            .map_err(|e| eyre::eyre!("failed to register component: {e:?}"))?;

        Ok(())
    }

    pub fn init_source_file(&mut self, source_file: &SourceFile) -> eyre::Result<()> {
        if self.component_vtables.is_some() {
            let component_vtables = self
                .component_vtables
                .take()
                .expect("component vtables taken");
            self.inner.op_state().borrow_mut().put(component_vtables);
        }

        info!("initializing source file: {:?}", source_file.config.name);

        self.inner
            .execute_script("<source>", source_file.code.clone())
            .map_err(|e| eyre::eyre!("failed to execute source init script: {e:?}"))?;

        Ok(())
    }

    pub fn init_behavior(
        &mut self,
        attached_to: EntityId,
        source_file: &SourceFile,
        behavior: &mut Behavior,
    ) -> eyre::Result<()> {
        let id = self.behavior_id;
        self.behavior_id += 1;
        behavior.data = Some(BehaviorData { id });

        self.inner
            .execute_script(
                "<create-behavior>",
                format!(
                    "acro.createBehavior({}, {}, {}, \"{}\")",
                    attached_to.generation, attached_to.index, id, source_file.config.name
                ),
            )
            .map_err(|e| eyre::eyre!("failed to execute behavior init script: {e:?}"))?;

        Ok(())
    }

    pub fn update(&mut self) -> eyre::Result<()> {
        self.inner
            .execute_script("<update>", "acro.update()")
            .map_err(|e| eyre::eyre!("failed to execute update script: {e:?}"))?;

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
        runtime.init_behavior(entity, &source_file, &mut behavior)?;
    }

    Ok(())
}

pub fn update_behaviors(
    ctx: SystemRunContext,
    mut runtime: ResMut<ScriptingRuntime>,
) -> eyre::Result<()> {
    runtime.update()
}
