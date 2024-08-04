pub mod application;
pub mod archetype;
pub mod bundle;
pub mod entity;
pub mod plugin;
pub mod pointer;
pub mod query;
pub mod registry;
pub mod resource;
pub mod schedule;
pub mod storage;
pub mod systems;
pub mod world;

pub use application::Application;
pub use bundle::Bundle;
pub use entity::EntityId;
pub use plugin::Plugin;
pub use pointer::change_detection::{Mut, Tick};
pub use query::{Changed, Or, Query, With, Without};
pub use registry::{ComponentId, ComponentRegistry, ComponentType};
pub use resource::{Res, ResMut};
pub use schedule::{Schedule, Stage, SystemSchedulingRequirement};
pub use systems::SystemRunContext;
pub use world::World;

#[derive(Debug)]
pub struct Name(pub String);
