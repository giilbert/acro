use acro_ecs::Application;
pub use acro_ecs::Plugin;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

pub struct LogPlugin;

impl Plugin for LogPlugin {
    fn build(&mut self, _app: &mut Application) {
        let subscriber =
            tracing_subscriber::FmtSubscriber::new().with(EnvFilter::from_default_env());

        tracing::subscriber::set_global_default(subscriber)
            .expect("Failed to set global subscriber");
    }
}
