use std::{cell::RefCell, collections::HashMap, rc::Rc};

use acro_assets::Assets;
use acro_ecs::{Changed, ComponentId, EntityId, Query, Res, ResMut, SystemRunContext, Tick, World};
use acro_reflect::Reflect;

pub trait Platform {
    fn new() -> Self;

    fn init_source_file(&mut self, source_file: &SourceFile) -> eyre::Result<()>;
    fn init_behavior(
        &mut self,
        id: u32,
        attached_to: EntityId,
        source_file: &SourceFile,
    ) -> eyre::Result<()>;
    fn update(&mut self, last_update: DateTime<Utc>, tick: Tick) -> eyre::Result<DateTime<Utc>>;
    fn late_init(
        &mut self,
        component_vtables: &mut ComponentVTables,
        world_handle: Rc<RefCell<World>>,
        name_to_component_id: &HashMap<String, ComponentId>,
    ) -> eyre::Result<()>;
    fn call_function<T: DeserializeOwned>(
        &mut self,
        function: &FunctionHandle,
        arguments: &impl serde::Serialize,
    ) -> eyre::Result<T>;
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
    use serde::de::DeserializeOwned;
    pub use NativePlatform as Platform;

    use crate::SourceFile;

    use super::ComponentVTables;

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

        pub fn add_op(&mut self, op: deno_core::OpDecl) {
            self.ops_dec.as_mut().expect("ops already taken").push(op);
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

        fn init_source_file(&mut self, source_file: &SourceFile) -> eyre::Result<()> {
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
        ) -> eyre::Result<()> {
            let module_handle = self.init_module_handle.as_ref().map(|h| h.clone());
            self.inner_mut().call_function::<()>(
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

        fn update(
            &mut self,
            last_update: DateTime<Utc>,
            tick: Tick,
        ) -> eyre::Result<DateTime<Utc>> {
            self.inner_mut()
                .deno_runtime()
                .op_state()
                .borrow_mut()
                .put(tick);

            let delta_time = Utc::now().timestamp_subsec_nanos() as f64 / 1_000_000_000.0
                - last_update.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;

            let module_handle = self.init_module_handle.as_ref().map(|h| h.clone());
            self.inner_mut().call_function::<()>(
                module_handle.as_ref(),
                "update",
                json_args!(delta_time),
            )?;

            Ok(Utc::now())
        }

        fn late_init(
            &mut self,
            component_vtables: &mut ComponentVTables,
            world_handle: Rc<RefCell<World>>,
            name_to_component_id: &HashMap<String, ComponentId>,
        ) -> eyre::Result<()> {
            if self.inner.is_some() {
                return Ok(());
            }

            {
                let mut registered_ops = self.ops_dec.take().expect("ops already taken");
                use crate::platform::deno_ops::{
                    op_call_function, op_get_property_boolean, op_get_property_number,
                    op_get_property_string, op_set_property_boolean, op_set_property_number,
                    op_set_property_string,
                };

                registered_ops.push(op_get_property_string());
                registered_ops.push(op_set_property_string());
                registered_ops.push(op_get_property_number());
                registered_ops.push(op_set_property_number());
                registered_ops.push(op_get_property_boolean());
                registered_ops.push(op_set_property_boolean());
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

                // TODO: add a way to reference the lib/core/init.ts module
                let init_module = Module::load(
                    env!("CARGO_MANIFEST_DIR")
                        .to_string()
                        .replace("crates/acro_scripting", "lib/core/init.ts"),
                )?;

                let init_module_handle = runtime
                    .load_module(&init_module)
                    .expect("error loading init module");

                runtime.call_function::<()>(Some(&init_module_handle), "init", json_args!())?;

                info!("registered {} component(s)", name_to_component_id.len());

                runtime.call_function::<()>(
                    Some(&init_module_handle),
                    "registerComponents",
                    json_args!(name_to_component_id),
                )?;

                self.inner = Some(runtime);
                self.init_module_handle = Some(init_module_handle);
            }

            if component_vtables.is_some() {
                let component_vtables = component_vtables.take().expect("component vtables taken");
                self.inner_mut()
                    .deno_runtime()
                    .op_state()
                    .borrow_mut()
                    .put(component_vtables);
            }

            Ok(())
        }

        fn call_function<T: DeserializeOwned>(
            &mut self,
            function: &FunctionHandle,
            arguments: &impl serde::Serialize,
        ) -> eyre::Result<T> {
            let runtime = self.inner_mut();
            // TODO: figure out what a module_context is
            let results = function.inner.call::<T>(runtime, None, arguments)?;
            Ok(results)
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
                    // let mut cwd = std::env::current_dir().expect("failed to get current directory");
                    // TODO: allow for a way to reference lib
                    let mut cwd = std::path::PathBuf::from(
                        env!("CARGO_MANIFEST_DIR")
                            .to_string()
                            .replace("crates/acro_scripting", "lib"),
                    );
                    cwd.push(&format!(
                        "{}/mod.ts",
                        specifier.path().replace("@acro/", "")
                    ));

                    let path = cwd.to_str().expect("failed to convert path to string");
                    let url = Url::parse(&format!("file://{}", path)).expect("failed to parse url");

                    tracing::info!("resolved: {} -> {}", specifier, url);

                    Some(Ok(url))
                }
                _ => None,
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod runtime_impl {
    use acro_ecs::{utils::TimeDeltaExt, Tick};
    use chrono::{DateTime, Utc};
    use js_sys::{Object, Reflect};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = acro, js_name = update)]
        fn js_update(delta_time: f64);

        #[wasm_bindgen(js_namespace = acro, js_name = registerComponents)]
        // components: Record<string, number>
        fn js_register_components(components: JsValue);

        #[wasm_bindgen(js_namespace = acro, js_name = createBehavior)]
        fn js_create_behavior(generation: u32, index: u32, id: u32, name: &str);
    }

    pub struct WasmPlatform {
        has_late_init: bool,
    }

    impl super::Platform for WasmPlatform {
        fn new() -> Self {
            Self {
                has_late_init: false,
            }
        }

        // This is a no-op for the wasm platform
        fn init_source_file(&mut self, _source_file: &crate::SourceFile) -> eyre::Result<()> {
            Ok(())
        }

        fn init_behavior(
            &mut self,
            id: u32,
            attached_to: acro_ecs::EntityId,
            source_file: &crate::SourceFile,
        ) -> eyre::Result<()> {
            tracing::info!(
                "init_behavior({}, {:?}, {})",
                id,
                attached_to,
                source_file.config.name
            );

            js_create_behavior(
                attached_to.generation,
                attached_to.index,
                id,
                &source_file.config.name,
            );

            Ok(())
        }

        fn update(
            &mut self,
            last_update: DateTime<Utc>,
            tick: Tick,
        ) -> eyre::Result<DateTime<Utc>> {
            WASM_OPS_STATE.insert(tick);

            let delta_time = Utc::now()
                .signed_duration_since(last_update)
                .get_frac_secs() as f64;

            js_update(delta_time);

            Ok(Utc::now())
        }

        fn late_init(
            &mut self,
            component_vtables: &mut super::ComponentVTables,
            world_handle: std::rc::Rc<std::cell::RefCell<acro_ecs::World>>,
            name_to_component_id: &std::collections::HashMap<String, acro_ecs::ComponentId>,
        ) -> eyre::Result<()> {
            if self.has_late_init {
                return Ok(());
            }

            if component_vtables.is_some() {
                let component_vtables = component_vtables.take().expect("component vtables taken");
                WASM_OPS_STATE.insert(component_vtables);
            }

            WASM_OPS_STATE.insert(world_handle);
            WASM_OPS_STATE.insert(Tick::new(0));

            let components = Object::new().into();

            for (name, id) in name_to_component_id {
                Reflect::set(
                    &components,
                    &JsValue::from_str(name),
                    &JsValue::from_f64(id.0 as f64),
                )
                .expect("failed to set component");
            }

            js_register_components(components);

            self.has_late_init = true;

            Ok(())
        }

        fn call_function<T: serde::de::DeserializeOwned>(
            &mut self,
            function: &crate::platform::FunctionHandle,
            arguments: &impl serde::Serialize,
        ) -> eyre::Result<T> {
            let ret = Reflect::apply(
                &function.inner,
                &JsValue::NULL,
                &serde_wasm_bindgen::to_value(arguments)
                    .map_err(|e| eyre::eyre!("failed to serialize function parameters: {:?}", e))?
                    .into(),
            )
            .map_err(|e| eyre::eyre!("failed to call function: {:?}", e))?;

            serde_wasm_bindgen::from_value(ret)
                .map_err(|e| eyre::eyre!("failed to deserialize function return value: {:?}", e))
        }
    }

    use tracing::info;
    pub use WasmPlatform as Platform;

    use crate::{behavior::BehaviorData, wasm_ops::WASM_OPS_STATE};
}

use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use tracing::info;

use crate::{
    behavior::{Behavior, BehaviorData},
    platform::FunctionHandle,
    source_file::SourceFile,
    EventListenerStore,
};

type ComponentVTables = Option<HashMap<ComponentId, *const ()>>;

pub struct ScriptingRuntime {
    last_update: DateTime<Utc>,
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
            last_update: Utc::now(),
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
        self.platform.init_source_file(source_file)
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
        self.platform.init_behavior(id, attached_to, source_file)
    }

    pub fn update(&mut self, tick: Tick) -> eyre::Result<()> {
        self.last_update = self.platform.update(self.last_update, tick)?;
        Ok(())
    }

    pub fn late_init(&mut self) {
        self.platform
            .late_init(
                &mut self.component_vtables,
                self.world_handle.clone(),
                &self.name_to_component_id,
            )
            .expect("failed to late init");
    }

    pub fn call_function<T: DeserializeOwned>(
        &mut self,
        function: &FunctionHandle,
        arguments: &impl serde::Serialize,
    ) -> eyre::Result<T> {
        Ok(self.platform.call_function(function, arguments)?)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn native_add_op(&mut self, op: deno_core::OpDecl) {
        self.platform.add_op(op);
    }
}

pub fn late_init_scripting_runtime(_ctx: SystemRunContext, mut runtime: ResMut<ScriptingRuntime>) {
    runtime.late_init();
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
    // let now = Utc::now();
    runtime.update(ctx.tick)?;
    // info!(
    //     "update_behaviors: {:?}",
    //     Utc::now().signed_duration_since(now).pretty()
    // );
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
