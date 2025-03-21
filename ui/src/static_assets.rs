use std::path::{Path, PathBuf};

use axum::Router;
use shipwright_config::Config;
use tower_http::services::ServeDir;

use crate::Error;

pub struct StaticAssetsInitializer {
    path: PathBuf,
}

impl StaticAssetsInitializer {
    pub fn init(config: &Config) -> Self {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(&config.static_assets.path));

        Self { path }
    }
}

impl StaticAssetsInitializer {
    pub fn name(&self) -> String {
        "static-assets".to_string()
    }

    pub fn before_run(&self) -> Result<(), Error> {
        tracing::info!("Initializing static assets handler");
        Ok(())
    }

    pub fn after_routes(self, mut router: Router) -> Result<Router, Error> {
        router = router.nest_service("/static", ServeDir::new(self.path.as_path()));

        Ok(router)
    }
}
