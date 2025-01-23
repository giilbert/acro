use std::{cell::RefCell, collections::HashMap, rc::Rc, time::SystemTime};

use acro_assets::Assets;
use acro_ecs::{Changed, ComponentId, EntityId, Query, Res, ResMut, SystemRunContext, Tick, World};
use acro_reflect::Reflect;

pub trait Platform {
    fn new() -> Self;

    fn init_source_file(
        &mut self,
        component_vtables: &mut ComponentVTables,
        source_file: &SourceFile,
    ) -> eyre::Result<()>;
    fn init_behavior(
        &mut self,
        id: u32,
        attached_to: EntityId,
        source_file: &SourceFile,
        behavior: &mut Behavior,
    ) -> eyre::Result<()>;
    fn update(&mut self, last_update: SystemTime, tick: Tick) -> eyre::Result<SystemTime>;
    fn late_init(&self) -> eyre::Result<()>;
}

#[cfg(not(target_arch = "wasm32"))]
mod runtime_impl {
    use std::borrow::Cow;

    use super::*;
    use deno_core::url::Url;
    use rustyscript::{
        deno_core, json_args, module_loader::ImportProvider, Module, ModuleHandle,
        Runtime as JsRuntime, RuntimeOptions, Undefined,
    };

    pub struct NativePlatform {
        ops_dec: Option<Vec<deno_core::OpDecl>>,
        inner: Option<JsRuntime>,
        init_module_handle: Option<ModuleHandle>,
    }

    impl NativePlatform {
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
    }

    impl super::Platform for NativePlatform {
        fn new() -> Self {
            Self {
                ops_dec: Some(Vec::new()),
                inner: None,
                init_module_handle: None,
            }
        }

        fn init_source_file(
            &mut self,
            component_vtables: &mut ComponentVTables,
            source_file: &SourceFile,
        ) -> eyre::Result<()> {
            if component_vtables.is_some() {
                let component_vtables = component_vtables.take().expect("component vtables taken");
                self.inner_mut()
                    .deno_runtime()
                    .op_state()
                    .borrow_mut()
                    .put(component_vtables);
            }

            tracing::info!("initializing source file: {:?}", source_file.config.name);

            // TODO: cleanup module after it has been unloaded
            let module_handle = self.inner_mut().load_module(&Module::new(
                &format!("./lib/{}.ts", source_file.config.name),
                &source_file.code,
            ))?;
            self.inner_mut()
                .call_entrypoint::<Undefined>(&module_handle, json_args!())?;

            Ok(())
        }

        fn init_behavior(
            &mut self,
            id: u32,
            attached_to: EntityId,
            source_file: &SourceFile,
            behavior: &mut Behavior,
        ) -> eyre::Result<()> {
            behavior.data = Some(BehaviorData { id });

            let module_handle = self.init_module_handle.as_ref().map(|h| h.clone());
            self.inner_mut().call_function(
                module_handle.as_ref(),
                "createBehavior",
                json_args!(
                    attached_to.generation,
                    attached_to.index,
                    id,
                    source_file.config.name.as_str()
                ),
            )?;

            Ok(())
        }

        fn update(&mut self, last_update: SystemTime, tick: Tick) -> eyre::Result<SystemTime> {
            self.inner_mut()
                .deno_runtime()
                .op_state()
                .borrow_mut()
                .put(tick);

            let delta_time = last_update.elapsed()?.as_secs_f64();

            let module_handle = self.init_module_handle.as_ref().map(|h| h.clone());
            self.inner_mut().call_function(
                module_handle.as_ref(),
                "update",
                json_args!(delta_time),
            )?;

            Ok(SystemTime::now())
        }

        fn late_init(&self, world_handle: Rc<RefCell<World>>) -> eyre::Result<()> {
            if self.inner.is_some() {
                return Ok(());
            }

            {
                let mut registered_ops = self.ops_dec.take().expect("ops already taken");
                use crate::platform::deno_ops::{
                    op_call_function, op_get_property_number, op_get_property_string,
                    op_set_property_number, op_set_property_string,
                };
                registered_ops.push(op_get_property_string());
                registered_ops.push(op_set_property_string());
                registered_ops.push(op_get_property_number());
                registered_ops.push(op_set_property_number());
                registered_ops.push(op_call_function());

                let ext = deno_core::Extension {
                    name: "reflect",
                    ops: Cow::Owned(registered_ops),
                    ..Default::default()
                };

                let mut runtime = JsRuntime::new(RuntimeOptions {
                    extensions: vec![ext],
                    default_entrypoint: Some("init".to_string()),
                    import_provider: Some(Box::new(AcroLibImportProvider)),
                    ..Default::default()
                })?;

                {
                    let op_state = runtime.deno_runtime().op_state();
                    let mut op_state = op_state.borrow_mut();

                    op_state.put(world_handle.clone());
                    op_state.put(Tick::new(0));
                }

                let init_module = Module::load("lib/core/init.ts")?;
                let init_module_handle = runtime
                    .load_module(&init_module)
                    .expect("error loading init module");

                runtime.call_function(Some(&init_module_handle), "init", json_args!())?;

                info!(
                    "registered {} component(s)",
                    self.name_to_component_id.len()
                );

                runtime.call_function(
                    Some(&init_module_handle),
                    "registerComponents",
                    json_args!(name_to_component_id),
                )?;

                runtime.inner = Some(runtime);
                self.init_module_handle = Some(init_module_handle);
            }

            Ok(())
        }
    }

    #[derive(Default)]
    pub struct AcroLibImportProvider;

    impl ImportProvider for AcroLibImportProvider {
        fn resolve(
            &mut self,
            specifier: &deno_core::ModuleSpecifier,
            _referrer: &str,
            _kind: deno_core::ResolutionKind,
        ) -> Option<Result<deno_core::ModuleSpecifier, deno_core::anyhow::Error>> {
            match specifier.scheme() {
                "jsr" if specifier.path().starts_with("@acro/") => {
                    let mut cwd = std::env::current_dir().expect("failed to get current directory");
                    cwd.push(&format!(
                        "lib/{}/mod.ts",
                        specifier.path().replace("@acro/", "")
                    ));
                    cwd.to_str()
                        .map(|path| Ok(Url::parse(&format!("file://{}", path)).unwrap()))
                }
                _ => None,
            }
        }
    }

    pub use NativePlatform as Platform;

    use crate::SourceFile;

    use super::ComponentVTables;
}

#[cfg(target_arch = "wasm32")]
mod runtime_impl {
    use js_sys::Function;

    pub struct WasmPlatform {}

    impl super::Platform for WasmPlatform {
        fn new() -> Self {
            todo!();
        }

        fn init_source_file(
            &mut self,
            component_vtables: &mut super::ComponentVTables,
            source_file: &crate::SourceFile,
        ) -> eyre::Result<()> {
            todo!();
        }
    }

    pub use WasmPlatform as Platform;
}

use tracing::info;

use crate::{
    behavior::{Behavior, BehaviorData},
    source_file::SourceFile,
    EventListenerStore,
};

type ComponentVTables = Option<HashMap<ComponentId, *const ()>>;

pub struct ScriptingRuntime {
    last_update: SystemTime,
    world_handle: Rc<RefCell<World>>,
    behavior_id: u32,
    name_to_component_id: HashMap<String, ComponentId>,
    component_vtables: ComponentVTables,
    platform: runtime_impl::Platform,
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
            last_update: SystemTime::now(),
            behavior_id: 0,
            world_handle,
            name_to_component_id: HashMap::new(),
            component_vtables: Some(HashMap::new()),

            platform: runtime_impl::Platform::new(),
        }
    }

    pub fn register_component<T: Reflect + 'static>(&mut self, name: &str) {
        let world = self.world_handle.borrow();
        let (_data, vtable_ptr) = unsafe {
            std::mem::transmute::<&dyn Reflect, (*const (), *const ())>(
                (std::mem::MaybeUninit::<T>::uninit().assume_init_ref()) as &dyn Reflect,
            )
        };

        let component_info = world.get_component_info::<T>();
        self.component_vtables
            .as_mut()
            .expect("component vtables already taken")
            .insert(component_info.id, vtable_ptr);

        self.name_to_component_id
            .insert(name.to_string(), component_info.id);
    }

    pub fn init_source_file(&mut self, source_file: &SourceFile) -> eyre::Result<()> {
        self.platform
            .init_source_file(&mut self.component_vtables, source_file)
    }

    pub fn init_behavior(
        &mut self,
        attached_to: EntityId,
        source_file: &SourceFile,
        behavior: &mut Behavior,
    ) -> eyre::Result<()> {
        let id = self.behavior_id;
        self.behavior_id += 1;

        self.platform
            .init_behavior(id, attached_to, source_file, behavior)
    }

    pub fn update(&mut self, tick: Tick) -> eyre::Result<()> {
        self.last_update = self.platform.update(self.last_update, tick)?;
        Ok(())
    }

    pub fn late_init(&self) {}

    // pub fn add_op(&mut self, decl: deno_core::OpDecl) -> &mut Self {
    //     self.decl
    //         .as_mut()
    //         .expect("ops already initialized")
    //         .push(decl);
    //     self
    // }

    // pub fn take_ops(&mut self) -> Vec<deno_core::OpDecl> {
    //     self.decl.take().expect("ops already initialized")
    // }
}

pub fn init_scripting_runtime(
    _ctx: SystemRunContext,
    mut runtime: ResMut<ScriptingRuntime>,
) -> eyre::Result<()> {
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
    // let now = std::time::Instant::now();
    runtime.update(ctx.tick)?;
    // info!("update_behaviors: {:?}", now.elapsed());
    Ok(())
}

pub fn flush_events(
    ctx: SystemRunContext,
    mut event_listeners: ResMut<EventListenerStore>,
    mut runtime: ResMut<ScriptingRuntime>,
) -> eyre::Result<()> {
    event_listeners
        .inner_mut()
        .update_active_event_listeners(&mut runtime)?;

    Ok(())
}
