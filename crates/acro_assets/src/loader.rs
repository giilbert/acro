use acro_ecs::World;

pub trait Loadable: Send + Sync
where
    Self: Sized + 'static,
{
    // TODO: Handle errors
    fn load(world: &World, path: &str) -> Result<Self, ()>;
}
