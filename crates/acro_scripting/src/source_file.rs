use std::sync::Arc;

use acro_assets::{Loadable, LoaderContext};
use tracing::info;

use crate::runtime::ScriptingRuntime;

#[derive(Debug)]
pub struct SourceFile {
    pub(crate) code: String,
    pub(crate) config: Arc<SourceFileConfig>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SourceFileConfig {
    pub(crate) name: String,
}

impl Loadable for SourceFile {
    type Config = SourceFileConfig;

    fn load(ctx: &LoaderContext, config: Arc<Self::Config>, data: Vec<u8>) -> eyre::Result<Self> {
        let mut runtime = ctx
            .system_run_context
            .world
            .resources()
            .get_mut::<ScriptingRuntime>();

        let source_file = Self {
            code: String::from_utf8_lossy(&data).to_string(),
            config,
        };

        runtime.init_source_file(&source_file)?;

        Ok(source_file)
    }
}
