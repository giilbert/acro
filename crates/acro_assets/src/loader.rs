use acro_ecs::World;

pub trait Loadable: Send + Sync
where
    Self: Sized + 'static,
{
    // TODO: add a way to pass in an option
    // TODO: Handle errors
    fn load(world: &World, data: Vec<u8>) -> Result<Self, ()>;
}
