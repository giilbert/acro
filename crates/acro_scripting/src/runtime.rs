use core::panic;
use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc, time::SystemTime};

use acro_assets::Assets;
use acro_ecs::{Changed, ComponentId, EntityId, Query, Res, ResMut, SystemRunContext, Tick, World};
use acro_reflect::Reflect;
use deno_core::url::Url;
use fnv::FnvHashMap;
use rustyscript::{
    deno_core, js_value::Function, json_args, module_loader::ImportProvider, Module, ModuleHandle,
    Runtime as JsRuntime, RuntimeOptions, Undefined,
};
use tracing::info;

use crate::{
    behavior::{Behavior, BehaviorData},
    ops::{
        op_get_property_number, op_get_property_string, op_set_property_number,
        op_set_property_string,
    },
    source_file::SourceFile,
    AnyEventQueue, WeakEventQueueRef,
};

pub struct ScriptingRuntime {
    last_update: SystemTime,
    world_handle: Rc<RefCell<World>>,
    behavior_id: u32,
    name_to_component_id: HashMap<String, ComponentId>,
    decl: Option<Vec<deno_core::OpDecl>>,
    inner: Option<JsRuntime>,
    component_vtables: Option<HashMap<ComponentId, *const ()>>,
    init_module_handle: Option<ModuleHandle>,

    event_listener_id: EventListenerId,
    event_listeners: FnvHashMap<EventListenerId, BoundEventQueue>,
}

struct BoundEventQueue {
    pub queue: WeakEventQueueRef,
    pub function: Function,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct EventListenerId(pub u32);

impl EventListenerId {
    pub fn next_id(&mut self) -> Self {
        let id = self.0;
        self.0 += 1;
        Self(id)
    }
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
            decl: Some(vec![]),
            inner: None,
            component_vtables: Some(HashMap::new()),
            init_module_handle: None,

            event_listener_id: EventListenerId(0),
            event_listeners: FnvHashMap::default(),
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

        // TODO: cleanup module after it has been unloaded
        let module_handle = self.inner_mut().load_module(&Module::new(
            &format!("./lib/{}.ts", source_file.config.name),
            &source_file.code,
        ))?;
        self.inner_mut()
            .call_entrypoint::<Undefined>(&module_handle, json_args!())?;

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

    pub fn update(&mut self, tick: Tick) -> eyre::Result<()> {
        self.inner_mut()
            .deno_runtime()
            .op_state()
            .borrow_mut()
            .put(tick);

        let delta_time = self.last_update.elapsed()?.as_secs_f64();

        let module_handle = self.init_module_handle.as_ref().map(|h| h.clone());
        self.inner_mut()
            .call_function(module_handle.as_ref(), "update", json_args!(delta_time))?;

        self.last_update = SystemTime::now();

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

    pub fn create_event_listener_function(&mut self, function: Function) -> EventListenerId {
        let queue = AnyEventQueue::new();
        let id = self.event_listener_id.next_id();

        let weak_ref = queue.into_weak();
        self.event_listeners.insert(
            id,
            BoundEventQueue {
                queue: weak_ref,
                function,
            },
        );

        id
    }

    pub fn remove_event_listener(&mut self, id: EventListenerId) {}

    pub fn update_active_event_listeners(&self) -> eyre::Result<()> {
        let mut dead_listeners = vec![];

        for (id, bound_queue) in &self.event_listeners {
            match bound_queue.queue.upgrade() {
                Some(queue) => {
                    while let Some(data) = queue.next() {
                        // TODO: call function with data
                    }
                }
                None => dead_listeners.push(*id),
            }
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

pub fn init_scripting_runtime(
    _ctx: SystemRunContext,
    mut runtime: ResMut<ScriptingRuntime>,
) -> eyre::Result<()> {
    if runtime.inner.is_some() {
        return Ok(());
    }

    let mut registered_ops = runtime.take_ops();

    registered_ops.push(op_get_property_string());
    registered_ops.push(op_set_property_string());
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
        import_provider: Some(Box::new(AcroLibImportProvider)),
        ..Default::default()
    })?;

    inner_runtime
        .deno_runtime()
        .op_state()
        .borrow_mut()
        .put(runtime.world_handle.clone());

    let init_module = Module::load("lib/core/init.ts")?;
    let init_module_handle = inner_runtime
        .load_module(&init_module)
        .expect("error loading init module");

    inner_runtime.call_function(Some(&init_module_handle), "init", json_args!())?;

    info!(
        "registered {} component(s)",
        runtime.name_to_component_id.len()
    );

    inner_runtime.call_function(
        Some(&init_module_handle),
        "registerComponents",
        json_args!(runtime.name_to_component_id),
    )?;

    runtime.inner = Some(inner_runtime);
    runtime.init_module_handle = Some(init_module_handle);

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
