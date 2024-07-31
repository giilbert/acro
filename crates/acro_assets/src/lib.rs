mod asset;
mod loader;

use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet, VecDeque},
    path::Path,
    sync::Arc,
};

pub use crate::{asset::Asset, loader::Loadable};
pub use serde;

use acro_ecs::{
    systems::NotifyChangeError, Application, ComponentId, EntityId, Plugin, Stage,
    SystemRunContext, World,
};
use notify::{event::AccessKind, EventKind, RecursiveMode, Watcher};
use parking_lot::{Mutex, RwLock};
use tracing::{error, info};

pub struct Assets {
    queue: Arc<Mutex<VecDeque<QueuedAsset>>>,
    data: Arc<RwLock<HashMap<String, AnyAssetData>>>,
    watcher: notify::RecommendedWatcher,
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
    notify_changes: Arc<RwLock<HashMap<EntityId, HashSet<ComponentId>>>>,
}
pub type AssetLoader = Arc<dyn Fn(&World, Vec<u8>, Vec<u8>) -> AnyAssetData>;

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
        let watcher = notify::recommended_watcher(
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
        .expect("error initializing file watcher");

        Self {
            queue,
            data,
            watcher,
            asset_loaders: HashMap::new(),
        }
    }

    pub fn queue<T: Loadable>(&mut self, path: &str) {
        let path = path.to_string();

        self.watcher
            .watch(Path::new(path.as_str()), RecursiveMode::NonRecursive)
            .expect("failed to watch file");
        // Also watch the config file
        self.watcher
            .watch(
                Path::new(format!("{}.meta", &path).as_str()),
                RecursiveMode::NonRecursive,
            )
            .expect("failed to watch file");

        self.queue.lock().push_back(QueuedAsset {
            type_id: TypeId::of::<T>(),
            path: path.clone(),
            queue_type: QueueType::Init,
        });
    }

    pub fn process_queue(&self, ctx: &SystemRunContext) {
        let mut queue = self.queue.lock();
        while let Some(asset) = queue.pop_front() {
            let options_file_content = std::fs::read(&format!("{}.meta", asset.path))
                .expect("failed to read options file");
            let asset_file_content = std::fs::read(&asset.path).expect("failed to read asset file");

            let new_asset = (self
                .asset_loaders
                .get(&asset.type_id)
                .expect("asset does not exist"))(
                ctx.world,
                options_file_content,
                asset_file_content,
            );

            let mut data = self.data.write();

            match asset.queue_type {
                QueueType::Init => {
                    data.insert(
                        asset.path.clone(),
                        AnyAssetData {
                            data: new_asset.data,
                            config: new_asset.config,
                            id: asset.type_id,
                            notify_changes: Default::default(),
                        },
                    );

                    info!("asset loaded: {}", asset.path);
                }
                QueueType::Reload => {
                    let old_asset = data.get_mut(&asset.path).expect("asset not loaded");
                    old_asset.data = new_asset.data;

                    let mut component_count = 0;
                    for (&entity, components) in old_asset.notify_changes.write().iter_mut() {
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

                    info!(
                        "asset reloaded: {} (notified {component_count} component{})",
                        asset.path,
                        if component_count == 1 { "" } else { "s" },
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
            notify_changes: asset.notify_changes.clone(),
        }
    }

    pub fn register_loader<T: Loadable>(&mut self) {
        self.asset_loaders.insert(
            TypeId::of::<T>(),
            Arc::new(|world, config, data| {
                let config = Arc::new(
                    ron::de::from_bytes::<T::Config>(&config)
                        .expect("failed to deserialize config"),
                );

                AnyAssetData {
                    data: Arc::new(
                        T::load(world, Arc::clone(&config), data).expect("failed to load asset"),
                    ),
                    config,
                    id: TypeId::of::<T>(),
                    notify_changes: Default::default(),
                }
            }),
        );
    }
}

fn load_queued_system(ctx: SystemRunContext) {
    let world = &ctx.world;
    let mut assets = world.resources().get::<Assets>();
    assets.process_queue(&ctx);
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&mut self, app: &mut Application) {
        let world = app.world();
        world.insert_resource(Assets::new());
        app.add_system(Stage::PreUpdate, [], load_queued_system);
    }
}
