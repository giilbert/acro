use std::sync::Arc;

use acro_ecs::World;
use serde::de::DeserializeOwned;

pub trait Loadable: Send + Sync
where
    Self: Sized + 'static,
{
    type Config: DeserializeOwned + Send + Sync;

    // TODO: add a way to pass in an option
    // TODO: Handle errors
    fn load(world: &World, config: Arc<Self::Config>, data: Vec<u8>) -> Result<Self, ()>;
}
