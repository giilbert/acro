mod asset;
mod loader;

use std::{
    any::Any,
    collections::{HashMap, VecDeque},
    path::Path,
    sync::{mpsc, Arc},
};

pub use crate::{asset::Asset, loader::Loadable};

use acro_ecs::{Application, Plugin, Stage, SystemRunContext, World};
use asset::AnyAssetData;
use notify::{RecursiveMode, Watcher};
use tracing::{error, info, warn};

#[derive(Debug)]
pub struct Assets {
    queue: VecDeque<QueuedAsset>,
    data: HashMap<String, Arc<dyn Any + Send + Sync>>,
    watcher: notify::RecommendedWatcher,
}

struct QueuedAsset {
    queue_type: QueueType,
    name: String,
    loader: Box<dyn FnOnce(&World) -> AnyAssetData>,
}

impl std::fmt::Debug for QueuedAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueuedAsset")
            .field("queue_type", &self.queue_type)
            .field("name", &self.name)
            .field("loader", &"...")
            .finish()
    }
}

#[derive(Debug)]
enum QueueType {
    Init,
}

impl Assets {
    pub fn new() -> Self {
        let watcher = notify::recommended_watcher(|res| match res {
            Ok(event) => {
                info!("event: {:?}", event);
            }
            Err(e) => {
                error!("watch error: {:?}", e);
            }
        })
        .expect("error initializing file watcher");

        Self {
            queue: VecDeque::new(),
            data: HashMap::new(),
            watcher,
        }
    }

    pub fn queue<T: Loadable>(&mut self, path: &str) {
        let path = path.to_string();

        self.watcher
            .watch(Path::new(path.as_str()), RecursiveMode::NonRecursive)
            .expect("failed to watch file");

        self.queue.push_back(QueuedAsset {
            name: path.clone(),
            queue_type: QueueType::Init,
            loader: Box::new(move |world| {
                Arc::new(T::load(world, &path).expect("failed to load asset"))
            }),
        });
    }

    pub fn process_queue(&mut self, world: &World) {
        while let Some(asset) = self.queue.pop_front() {
            self.data.insert(asset.name, (asset.loader)(world));
        }
    }

    pub fn get<T: 'static>(&self, path: &str) -> Asset<T>
    where
        T: Loadable,
    {
        Asset {
            data: self
                .data
                .get(path)
                .expect("asset not loaded")
                .clone()
                .downcast()
                .expect("failed to downcast asset"),
        }
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
