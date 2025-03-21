use std::{
    collections::HashMap,
    future::pending,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use axum::{Extension, Router, extract::FromRequestParts, http::request::Parts};
use minijinja::path_loader;
use minijinja_autoreload::AutoReloader;
use shipwright_config::{Config, Environment};
use notify::Watcher as _;
use serde::Serialize;
use tokio::time::Instant;
use tower_livereload::{LiveReloadLayer, Reloader};

use crate::{Error, components::ComponentEngine};

pub trait ViewRenderer {
    /// Render a view template located by `key`
    ///
    /// # Errors
    ///
    /// This function will return an error if render fails
    fn render<S: Serialize>(&self, key: &str, data: S) -> Result<String, Error>;
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ViewEngine<E>(pub E);

impl<E> ViewEngine<E> {
    /// Creates a new [`Engine`] that wraps the given engine
    pub fn new(engine: E) -> Self {
        Self(engine)
    }
}

impl<S, E> FromRequestParts<S> for ViewEngine<E>
where
    S: Send + Sync,
    E: Clone + Send + Sync + 'static,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let Extension(tl): Extension<Self> = Extension::from_request_parts(parts, state)
            .await
            .expect("view_engine missing. Is the view_engine initialized?");

        Ok(tl)
    }
}

/// A struct representing an inline Minijinja view renderer.
///
/// This struct provides functionality to render templates using the Minijinja templating engine
/// directly from raw template strings.
pub fn template<S>(template: &str, data: S) -> Result<String, Error>
where
    S: Serialize,
{
    let minijinja = minijinja::Environment::new();
    Ok(minijinja.render_str(template, minijinja::Value::from_serialize(data))?)
}

impl<E> From<E> for ViewEngine<E> {
    fn from(inner: E) -> Self {
        Self::new(inner)
    }
}

#[derive(Clone)]
pub struct View {
    pub reloader: Arc<AutoReloader>,
    pub component_engine: ComponentEngine,
}

impl View {
    pub fn build(config: &Config) -> Result<Self, Error> {
        let templates_path = get_base_path(&config.view.templates_path);

        let reloader = AutoReloader::new(move |notifier| {
            let templates_path = templates_path.clone();
            let mut env = minijinja::Environment::new();
            // Watch the template directory for changes in debug mode
            if cfg!(debug_assertions) {
                notifier.set_fast_reload(true);
                notifier.watch_path(&templates_path, true);
            }
            // Load in the templates from the specified directory
            env.set_loader(path_loader(templates_path));
            Ok(env)
        });
        let component_engine = ComponentEngine::build(config)?;
        Ok(Self {
            reloader: Arc::new(reloader),
            component_engine,
        })
    }
}

impl ViewRenderer for View {
    fn render<S: Serialize>(&self, key: &str, data: S) -> Result<String, Error> {
        let env = self.reloader.acquire_env()?;
        let template = env.get_template(key)?;
        let base_html = template.render(minijinja::Value::from_serialize(data))?;
        let rendered = self.clone().component_engine.inject(&base_html)?;
        Ok(rendered)
    }
}

#[derive(Clone)]
pub struct ViewEngineInitializer {
    pub live_reload_layer: LiveReloadLayer,
    pub browser_reloader: Reloader,
}

impl Default for ViewEngineInitializer {
    fn default() -> Self {
        let live_reload_layer = LiveReloadLayer::new();
        let browser_reloader = live_reload_layer.reloader();
        Self {
            live_reload_layer,
            browser_reloader,
        }
    }
}

/// Initializers are used to hook in before and after the app is run
///
/// This is used to setup necessary components before the app is run
///
/// Then it can attach necessary components to the app after the routes are defined
impl ViewEngineInitializer {
    pub fn name(&self) -> String {
        "view-engine".to_string()
    }

    pub fn before_run(&self, config: Config) -> Result<(), Error> {
        let last_events = Arc::new(Mutex::new(HashMap::new()));

        let browser_reloader = self.browser_reloader.clone();

        // Spawn a task to keep the watcher alive
        tokio::spawn(async move {
            let mut watcher = notify::recommended_watcher({
                let last_events = Arc::clone(&last_events);
                move |res: Result<notify::Event, _>| {
                    match res {
                        Ok(event) => {
                            if let Some(path) = event.paths.first() {
                                let mut last_events = last_events.lock().unwrap();

                                // Ignore temp/backup files
                                // This stops the reloader
                                // from re-running
                                // unnecessarily
                                if path.to_string_lossy().ends_with('~')
                                    || path
                                        .extension()
                                        .map(|ext| ext == "swp" || ext == "swx" || ext == "bak")
                                        .unwrap_or(false)
                                {
                                    return;
                                }

                                let now = Instant::now();

                                // Only reload if enough time has passed since the last accepted reload
                                match last_events.get(path) {
                                    Some(last_time)
                                        if now.duration_since(*last_time)
                                            < std::time::Duration::from_millis(300) =>
                                    {
                                        // Too soon, skip this reload
                                    }
                                    _ => {
                                        // Accept this event and record time *after* accepting it
                                        tracing::info!("File changed: {:?}", path);

                                        browser_reloader.reload();

                                        last_events.insert(path.clone(), now);
                                    }
                                }
                            }
                        }
                        Err(e) => tracing::error!("Watch error: {:?}", e),
                    }
                }
            })
            .expect("Failed to create watcher");

            for path_str in &[
                config.view.templates_path,
                config.view.components_path,
                config.static_assets.path,
            ] {
                let base_path = get_base_path(path_str);
                let _ = watcher.watch(base_path.as_path(), notify::RecursiveMode::Recursive);
            }
            // Keep the task running indefinitely to keep the watcher alive
            pending::<()>().await;
        });
        Ok(())
    }

    pub fn after_routes(
        self,
        mut router: Router,
        config: &Config,
        env: &Environment,
    ) -> Result<Router, Error> {
        let minijinja_engine = View::build(config)?;

        if env == &Environment::Development {
            tracing::info!("live reload enabled in development mode");
            router = router.layer(self.live_reload_layer);
        }

        router = router.layer(Extension(ViewEngine::from(minijinja_engine)));

        Ok(router)
    }
}

pub fn get_base_path(path_str: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(Path::new(path_str))
}
