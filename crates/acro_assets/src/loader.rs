use acro_ecs::World;

pub trait Loadable: Send + Sync
where
    Self: Sized + 'static,
{
    // TODO: Handle errors
    fn load(world: &World, data: Vec<u8>) -> Result<Self, ()>;
}
