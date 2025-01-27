mod asset;
mod loader;

use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet, VecDeque},
    path::Path,
    sync::Arc,
};

pub use crate::{
    asset::Asset,
    loader::{Loadable, LoaderContext},
};
pub use serde;

use acro_ecs::{
    systems::NotifyChangeError, Application, ComponentId, EntityId, Plugin, Stage,
    SystemRunContext, World,
};
use notify::{event::AccessKind, EventKind, RecursiveMode, Watcher};
use parking_lot::{Mutex, RwLock};
use tracing::{error, info};

// TODO: better type?
type AssetId = String;

pub struct Assets {
    queue: Arc<Mutex<VecDeque<QueuedAsset>>>,
    data: Arc<RwLock<HashMap<AssetId, AnyAssetData>>>,
    watcher: Option<Mutex<notify::RecommendedWatcher>>,
    asset_loaders: HashMap<TypeId, AssetLoader>,
}

impl std::fmt::Debug for Assets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Assets")
            .field("queue", &self.queue)
            .field("data", &self.data)
            .field("watcher", &self.watcher)
            .field("asset_loaders", &"...")
            .finish()
    }
}

pub type AnyShared = Arc<dyn Any + Send + Sync>;
#[derive(Debug)]
pub struct AnyAssetData {
    id: TypeId,
    config: AnyShared,
    data: AnyShared,
    // When this asset is reloaded, notify these components
    notify_components: Arc<RwLock<HashMap<EntityId, HashSet<ComponentId>>>>,
    // When this asset is reloaded, notify these assets
    notify_assets: Arc<RwLock<HashSet<AssetId>>>,
}
pub type AssetLoader = Arc<dyn Fn(&LoaderContext, Vec<u8>, Vec<u8>) -> eyre::Result<AnyAssetData>>;

struct QueuedAsset {
    type_id: TypeId,
    queue_type: QueueType,
    path: String,
}

impl std::fmt::Debug for QueuedAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueuedAsset")
            .field("queue_type", &self.queue_type)
            .field("path", &self.path)
            .field("loader", &"...")
            .finish()
    }
}

#[derive(Debug)]
enum QueueType {
    Init,
    Reload,
}

impl Assets {
    pub fn new() -> Self {
        let queue = Arc::new(Mutex::new(VecDeque::new()));
        let data = Arc::new(RwLock::new(HashMap::new()));

        let queue_clone = queue.clone();
        let data_clone = data.clone();
        let watcher =
            if !cfg!(target_arch = "wasm32") {
                Some(
                    notify::recommended_watcher(
                        move |res: Result<notify::Event, notify::Error>| match res {
                            Ok(event) => {
                                if matches!(event.kind, EventKind::Access(AccessKind::Close(_))) {
                                    let mut queue = queue_clone.lock();
                                    for path in event.paths {
                                        let mut path = path
                                    .strip_prefix(
                                        std::env::current_dir()
                                            .expect("unable to get working directory"),
                                    )
                                    .expect("unable to get relative path from working directory")
                                    .to_str()
                                    .expect("unable to convert path to string")
                                    .to_string();

                                        if path.ends_with(".meta") {
                                            path = path
                                                .strip_suffix(".meta")
                                                .expect("error stripping .meta suffix")
                                                .to_string();
                                        }

                                        let data = data_clone.read();
                                        let asset_data: &AnyAssetData =
                                            data.get(&path).expect("asset not loaded");
                                        let id = asset_data.id;

                                        queue.push_back(QueuedAsset {
                                            type_id: id,
                                            path,
                                            queue_type: QueueType::Reload,
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                error!("watch error: {:?}", e);
                            }
                        },
                    )
                    .expect("error initializing file watcher"),
                )
            } else {
                None
            };

        Self {
            queue,
            data,
            watcher: watcher.map(Mutex::new),
            asset_loaders: HashMap::new(),
        }
    }

    fn watch(&self, path: &str) {
        if let Some(watcher) = &self.watcher {
            watcher
                .lock()
                .watch(Path::new(path), RecursiveMode::NonRecursive)
                .expect("failed to watch file");
            // Also watch the config file
            watcher
                .lock()
                .watch(
                    Path::new(format!("{}.meta", &path).as_str()),
                    RecursiveMode::NonRecursive,
                )
                .expect("failed to watch file");
        }
    }

    pub fn queue<T: Loadable>(&self, path: &str) {
        self.watch(path);

        // TODO: asset ids
        self.queue.lock().push_back(QueuedAsset {
            type_id: TypeId::of::<T>(),
            path: path.to_string(),
            queue_type: QueueType::Init,
        });
    }

    fn load_data(
        &self,
        system_run_context: &SystemRunContext,
        type_id: TypeId,
        path: &str,
    ) -> eyre::Result<AnyAssetData> {
        let options_file_content =
            std::fs::read(&format!("{}.meta", path)).expect("failed to read options file");
        let asset_file_content = std::fs::read(&path).expect("failed to read asset file");

        (self
            .asset_loaders
            .get(&type_id)
            .expect("asset does not exist"))(
            &LoaderContext {
                current_asset: path,
                assets: self,
                system_run_context,
            },
            options_file_content,
            asset_file_content,
        )
    }

    pub fn process_queue(&self, ctx: &SystemRunContext) {
        loop {
            // Need to lock the queue here and drop the lock to avoid a deadlock
            let asset = match self.queue.lock().pop_front() {
                None => break,
                Some(asset) => asset,
            };

            let new_asset_data = self.load_data(&ctx, asset.type_id, &asset.path);

            let mut data = self.data.write();

            match (new_asset_data, asset.queue_type) {
                (Err(e), QueueType::Init) => {
                    error!(
                        "failed to load asset for the first time: {}:\n{:?}",
                        asset.path, e
                    );
                    std::process::exit(1);
                }
                (Ok(new_asset_data), QueueType::Init) => {
                    data.insert(asset.path.clone(), new_asset_data);
                    info!("asset loaded: {}", asset.path);
                }
                (Err(e), QueueType::Reload) => {
                    error!(
                        "failed to reload asset. keeping old asset: {}:\n{:?}",
                        asset.path, e
                    );
                }
                (Ok(new_asset_data), QueueType::Reload) => {
                    let existing_asset = data.get_mut(&asset.path).expect("asset not loaded");
                    existing_asset.data = new_asset_data.data;

                    // Notify components that this asset changed
                    let mut component_count = 0;
                    for (&entity, components) in existing_asset.notify_components.write().iter_mut()
                    {
                        let mut components_to_remove = vec![];

                        for &component_id in components.iter() {
                            let notify_result =
                                ctx.force_notify_change_with_id(entity, component_id);

                            match notify_result {
                                Ok(_) => {}
                                Err(NotifyChangeError::EntityDeleted) => {
                                    components.clear();
                                    break;
                                }
                                Err(NotifyChangeError::ComponentDeleted) => {
                                    components_to_remove.push(component_id);
                                }
                            }

                            component_count += 1;
                        }

                        for component_id in components_to_remove {
                            components.remove(&component_id);
                        }
                    }

                    // Notify other assets that this asset changed by reloading them
                    let notify_assets = existing_asset.notify_assets.clone();
                    let mut notify_assets_count = 0;
                    for asset_id in notify_assets.write().iter() {
                        notify_assets_count += 1;
                        let to_notify = data.get(asset_id).expect("asset not loaded");
                        self.queue.lock().push_back(QueuedAsset {
                            type_id: to_notify.id,
                            path: asset_id.clone(),
                            queue_type: QueueType::Reload,
                        });
                    }

                    info!(
                        "asset reloaded: {} (notified {component_count} component{} and {notify_assets_count} asset{})",
                        asset.path,
                        if component_count == 1 { "" } else { "s" },
                        if notify_assets_count == 1 { "" } else { "s" },
                    );
                }
            }
        }
    }

    pub fn get<T: 'static>(&self, path: &str) -> Asset<T>
    where
        T: Loadable,
    {
        let data = self.data.read();
        let asset = data.get(path).expect("asset not loaded");

        Asset {
            data: asset
                .data
                .clone()
                .downcast()
                .expect("failed to downcast asset"),
            notify_changes: asset.notify_components.clone(),
        }
    }

    pub fn get_or_load<T: Loadable>(&self, ctx: &LoaderContext, path: &str) -> Asset<T> {
        if !self.data.read().contains_key(path) {
            self.queue::<T>(path);
            self.process_queue(ctx.system_run_context);
        }

        self.get(path)
    }

    pub fn add_notify_asset(&self, asset: &str, notify: &str) {
        let mut data = self.data.write();
        let asset_data = data.get_mut(asset).expect("asset not loaded");
        asset_data.notify_assets.write().insert(notify.to_string());
    }

    pub fn register_loader<T: Loadable>(&mut self) {
        self.asset_loaders.insert(
            TypeId::of::<T>(),
            Arc::new(|ctx, config, data| {
                let config = Arc::new(serde_yml::from_slice::<T::Config>(&config)?);

                T::load(ctx, Arc::clone(&config), data).map(|data| AnyAssetData {
                    data: Arc::new(data),
                    config,
                    id: TypeId::of::<T>(),
                    notify_components: Default::default(),
                    notify_assets: Default::default(),
                })
            }),
        );
    }
}

pub fn load_queued_assets(ctx: SystemRunContext) {
    let world = &ctx.world;
    let assets = world.resources().get::<Assets>();
    assets.process_queue(&ctx);
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&mut self, app: &mut Application) {
        app.add_system(Stage::PreUpdate, [], load_queued_assets)
            .insert_resource(Assets::new());
    }
}
