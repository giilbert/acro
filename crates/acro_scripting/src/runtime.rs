use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

use acro_assets::Assets;
use acro_ecs::{Changed, ComponentId, EntityId, Query, Res, ResMut, SystemRunContext, Tick, World};
use acro_reflect::Reflect;
use rustyscript::{deno_core, json_args, Module, Runtime as JsRuntime, RuntimeOptions, Undefined};
use tracing::info;

use crate::{
    behavior::{Behavior, BehaviorData},
    reflect::{op_get_property_number, op_set_property_number},
    source_file::SourceFile,
};

pub struct ScriptingRuntime {
    world_handle: Rc<RefCell<World>>,
    behavior_id: u32,
    name_to_component_id: HashMap<String, ComponentId>,
    decl: Option<Vec<deno_core::OpDecl>>,
    inner: Option<JsRuntime>,
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
        Self {
            behavior_id: 0,
            world_handle,
            name_to_component_id: HashMap::new(),
            decl: Some(vec![]),
            inner: None,
            component_vtables: Some(HashMap::new()),
        }
    }

    pub fn inner(&self) -> &JsRuntime {
        self.inner
            .as_ref()
            .expect("js runtime has not been initialized")
    }

    pub fn inner_mut(&mut self) -> &mut JsRuntime {
        self.inner
            .as_mut()
            .expect("js runtime has not been initialized")
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

        self.name_to_component_id
            .insert(name.to_string(), component_info.id);

        Ok(())
    }

    pub fn init_source_file(&mut self, source_file: &SourceFile) -> eyre::Result<()> {
        if self.component_vtables.is_some() {
            let component_vtables = self
                .component_vtables
                .take()
                .expect("component vtables taken");
            self.inner_mut()
                .deno_runtime()
                .op_state()
                .borrow_mut()
                .put(component_vtables);
        }

        info!("initializing source file: {:?}", source_file.config.name);

        let module_handle = self.inner_mut().load_module(&Module::new(
            &format!("./lib/{}.ts", source_file.config.name),
            &source_file.code,
        ))?;
        self.inner_mut()
            .call_entrypoint::<Undefined>(&module_handle, json_args!())?;

        // self.inner_mut()
        //     .deno_runtime()
        //     .execute_script(
        //         "<source>",
        //         format!("(() => {{{}}})()", source_file.code.clone()),
        //     )
        //     .map_err(|e| eyre::eyre!("failed to execute source init script: {e:?}"))?;

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

        self.inner_mut()
            .deno_runtime()
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

    pub fn update(&mut self, tick: Tick) -> eyre::Result<()> {
        self.inner_mut()
            .deno_runtime()
            .op_state()
            .borrow_mut()
            .put(tick);

        self.inner_mut()
            .deno_runtime()
            .execute_script("<update>", "acro.update()")
            .map_err(|e| eyre::eyre!("failed to execute update script: {e:?}"))?;

        Ok(())
    }

    pub fn add_op(&mut self, decl: deno_core::OpDecl) -> &mut Self {
        self.decl
            .as_mut()
            .expect("ops already initialized")
            .push(decl);
        self
    }

    pub fn take_ops(&mut self) -> Vec<deno_core::OpDecl> {
        self.decl.take().expect("ops already initialized")
    }
}

pub fn init_scripting_runtime(
    _ctx: SystemRunContext,
    mut runtime: ResMut<ScriptingRuntime>,
) -> eyre::Result<()> {
    if runtime.inner.is_some() {
        return Ok(());
    }

    let mut registered_ops = runtime.take_ops();

    registered_ops.push(op_get_property_number());
    registered_ops.push(op_set_property_number());

    let ext = deno_core::Extension {
        name: "reflect",
        ops: Cow::Owned(registered_ops),
        ..Default::default()
    };

    let mut inner_runtime = JsRuntime::new(RuntimeOptions {
        extensions: vec![ext],
        default_entrypoint: Some("init".to_string()),
        ..Default::default()
    })?;

    inner_runtime
        .deno_runtime()
        .op_state()
        .borrow_mut()
        .put(runtime.world_handle.clone());

    let main_module = Module::load("lib/init.ts")?;
    let main_module_handle = inner_runtime
        .load_module(&main_module)
        .expect("error loading init module");

    inner_runtime.call_entrypoint::<Undefined>(&main_module_handle, json_args!())?;

    // inner_runtime
    //     .deno_runtime()
    //     .execute_script("<init>", include_str!("js/bootstrap.js"))
    //     .map_err(|e| eyre::eyre!("failed to execute init script: {e:?}"))?;

    info!(
        "registered {} component(s)",
        runtime.name_to_component_id.len()
    );
    inner_runtime
        .deno_runtime()
        .execute_script(
            "<register-component>",
            runtime
                .name_to_component_id
                .iter()
                .map(|(name, ComponentId(component_id))| {
                    format!("acro.COMPONENT_IDS['{}']={};", name, component_id)
                })
                .collect::<String>(),
        )
        .map_err(|e| eyre::eyre!("failed to register component: {e:?}"))?;

    runtime.inner = Some(inner_runtime);

    Ok(())
}

pub fn init_behavior(
    ctx: SystemRunContext,
    behaviors: Query<(EntityId, &mut Behavior), Changed<Behavior>>,
    assets: Res<Assets>,
    mut runtime: ResMut<ScriptingRuntime>,
) -> eyre::Result<()> {
    for (entity, mut behavior) in behaviors.over(&ctx) {
        let source_file = assets.get::<SourceFile>(&behavior.source);
        source_file.notify_changes::<Behavior>(&ctx, entity);
        runtime.init_behavior(entity, &source_file, &mut behavior)?;
    }

    Ok(())
}

pub fn update_behaviors(
    ctx: SystemRunContext,
    mut runtime: ResMut<ScriptingRuntime>,
) -> eyre::Result<()> {
    runtime.update(ctx.tick)
}
