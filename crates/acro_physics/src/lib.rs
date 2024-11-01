use acro_ecs::{Application, Plugin, Stage};
use acro_math::Vec3;
use acro_scene::ComponentLoaders;
use integrator::{integrate_velocity_and_acceleration, PhysicsContext};
use properties::{Force, Mass, Rigidbody3D, Velocity};

mod integrator;
mod properties;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&mut self, app: &mut Application) {
        app.init_component::<Mass>()
            .init_component::<Velocity>()
            .init_component::<Force>()
            .init_component::<Rigidbody3D>()
            .add_system(Stage::Update, [], integrate_velocity_and_acceleration)
            .insert_resource(PhysicsContext::default())
            .with_resource::<ComponentLoaders>(|loaders| {
                loaders.register("Rigidbody3D", |world, entity, serialized| {
                    world.insert(entity, Rigidbody3D);
                    world.insert(entity, Mass(1.0));
                    world.insert(entity, Velocity::default());
                    world.insert(entity, Force(Vec3::new(0.0, -10.0, 0.0)));

                    Ok(())
                })
            });
    }
}
