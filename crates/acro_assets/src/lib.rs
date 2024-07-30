mod asset;
mod loader;

use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet, VecDeque},
    path::Path,
    sync::{mpsc, Arc},
};

pub use crate::{asset::Asset, loader::Loadable};

use acro_ecs::{
    pointer::change_detection::ChangeDetectionContext, Application, ComponentId, EntityId, Plugin,
    Stage, SystemRunContext, World,
};
use notify::{event::AccessKind, EventKind, RecursiveMode, Watcher};
use parking_lot::{Mutex, RwLock};
use tracing::{error, info, warn};

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

#[derive(Debug)]
pub struct AnyAssetData {
    id: TypeId,
    data: Arc<dyn Any + Send + Sync>,
    notify_changes: HashMap<EntityId, HashSet<ComponentId>>,
}
pub type AssetLoader = Arc<dyn Fn(&World, Vec<u8>) -> AnyAssetData>;

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
                            let path = path
                                .strip_prefix(
                                    std::env::current_dir()
                                        .expect("unable to get working directory"),
                                )
                                .expect("unable to get relative path from working directory")
                                .to_str()
                                .expect("unable to convert path to string")
                                .to_string();

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

        self.queue.lock().push_back(QueuedAsset {
            type_id: TypeId::of::<T>(),
            path: path.clone(),
            queue_type: QueueType::Init,
        });
    }

    pub fn process_queue(&mut self, world: &World) {
        let mut queue = self.queue.lock();
        while let Some(asset) = queue.pop_front() {
            match asset.queue_type {
                QueueType::Init => {
                    info!("loading asset: {}", asset.path);
                }
                QueueType::Reload => {
                    info!("reloading asset: {}", asset.path);
                }
            }

            let file_content = std::fs::read(&asset.path).expect("failed to read file");

            self.data.write().insert(
                asset.path,
                (self
                    .asset_loaders
                    .get(&asset.type_id)
                    .expect("asset does not exist"))(world, file_content),
            );
        }
    }

    pub fn get<T: 'static>(&self, path: &str) -> Asset<T>
    where
        T: Loadable,
    {
        Asset {
            data: self
                .data
                .read()
                .get(path)
                .expect("asset not loaded")
                .data
                .clone()
                .downcast()
                .expect("failed to downcast asset"),
        }
    }

    pub fn register_loader<T: Loadable>(&mut self) {
        self.asset_loaders.insert(
            TypeId::of::<T>(),
            Arc::new(|world, data| AnyAssetData {
                data: Arc::new(T::load(world, data).expect("failed to load asset")),
                id: TypeId::of::<T>(),
                notify_changes: HashMap::new(),
            }),
        );
    }
}

fn load_queued_system(ctx: SystemRunContext) {
    let world = &ctx.world;
    let mut assets = world.resources().get_mut::<Assets>();
    assets.process_queue(world);
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&mut self, app: &mut Application) {
        let world = app.world();
        world.insert_resource(Assets::new());
        app.add_system(Stage::PreUpdate, [], load_queued_system);
    }
}
