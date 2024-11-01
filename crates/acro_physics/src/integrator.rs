use std::time::SystemTime;

use acro_ecs::{Query, ResMut, SystemRunContext, With};
use acro_math::{Float, GlobalTransform, Transform};
use tracing::info;

use crate::properties::{Force, Mass, Rigidbody3D, Velocity};

#[derive(Debug, Default)]
pub struct PhysicsContext {
    pub last_integrate: Option<f64>,
}

pub fn integrate_velocity_and_acceleration(
    ctx: SystemRunContext,
    mut context: ResMut<PhysicsContext>,
    rigidbodies: Query<
        (
            &GlobalTransform,
            &mut Transform,
            &mut Velocity,
            &Mass,
            &Force,
        ),
        With<Rigidbody3D>,
    >,
) {
    let now = std::time::SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64() as f64;

    let last_integrate = match context.last_integrate {
        Some(last_integrate) => last_integrate,
        None => {
            context.last_integrate = Some(now);
            return;
        }
    };

    let dt = (now - last_integrate) as Float;

    for (global_transform, mut transform, mut velocity, mass, force) in rigidbodies.over(&ctx) {
        let force = force.0;
        let mass = mass.0;

        let acceleration = force / mass;

        transform.position += velocity.0.scale(dt);
        velocity.0 += acceleration.scale(dt);

        info!("position: {:?}", transform.position);
        info!("velocity: {:?}", velocity.0);
    }

    context.last_integrate = Some(now);
}
