use std::sync::Arc;

use acro_ecs::SystemRunContext;
use serde::de::DeserializeOwned;

use crate::{Asset, Assets};

pub struct LoaderContext<'w, 'a> {
    pub current_asset: &'a str,
    pub(crate) assets: &'a Assets,
    pub system_run_context: &'a SystemRunContext<'w>,
}

pub trait Loadable: Send + Sync
where
    Self: Sized + 'static,
{
    type Config: DeserializeOwned + Send + Sync;

    // TODO: Handle errors
    fn load(ctx: &LoaderContext, config: Arc<Self::Config>, data: Vec<u8>) -> eyre::Result<Self>;
}

impl<'w, 'l> LoaderContext<'w, 'l> {
    pub fn load_dependent<T: Loadable>(&self, ctx: &LoaderContext, path: &str) -> Asset<T> {
        let asset = self.assets.get_or_load(ctx, path);
        self.assets.add_notify_asset(path, self.current_asset);
        asset
    }
}
