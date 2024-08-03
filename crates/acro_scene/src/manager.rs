use acro_ecs::{ResMut, SystemRunContext};

use crate::scene::Scene;

#[derive(Debug, Default)]
pub struct SceneManager {
    pub current_scene: Option<String>,
    queued_scene: Option<String>,
}

impl SceneManager {
    pub fn queue(&mut self, scene: &str) {
        self.queued_scene = Some(scene.to_string());
    }
}

pub fn load_queued_scene(
    mut ctx: SystemRunContext,
    scene_manager: ResMut<SceneManager>,
) -> eyre::Result<()> {
    if let Some(scene_path) = &scene_manager.queued_scene {
        let scene = ron::from_str::<Scene>(&std::fs::read_to_string(scene_path)?)?;
        ctx.world.queue_swap(move |world| {
            scene.load(world);
        });
    }

    Ok(())
}
