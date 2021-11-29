use std::{ffi::OsStr, path::Path};

use figment::{
    providers::{Format, Toml},
    Figment,
};
use lsp_async_stub::Context;
use lsp_types::Url;
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use parking_lot::Mutex;

use crate::{
    config::{CargoRhaiConfig, RhaiConfig},
    documents,
    external::spawn,
    World,
};

pub struct WorkspaceWatcher {
    context: Context<World>,
    watcher: Mutex<RecommendedWatcher>,
}

impl WorkspaceWatcher {
    pub fn new(context: Context<World>) -> Result<Self, notify::Error> {
        let ctx = context.clone();
        let watcher = notify::recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| match res {
                Ok(event) => {
                    for p in event.paths {
                        if p.extension() != Some(OsStr::new("rhai")) || !p.is_file() {
                            continue;
                        }
                        tracing::trace!(path = ?p, "source file updated");

                        match format!("file://{}", p.to_string_lossy()).parse::<Url>() {
                            Ok(u) => match event.kind {
                                notify::EventKind::Remove(_) => {
                                    spawn(documents::remove_document(ctx.clone(), u));
                                }
                                _ => {
                                    let context = ctx.clone();
                                    spawn(async move {
                                        match tokio::fs::read_to_string(&p).await {
                                            Ok(text) => {
                                                documents::update_document(context, u, &text).await;
                                            }
                                            Err(err) => {
                                                tracing::error!(error = %err, "failed to read file");
                                            }
                                        }
                                    });
                                }
                            },
                            Err(err) => tracing::debug!(error = %err, "invalid file url"),
                        }
                    }
                }
                Err(e) => tracing::error!(error = %e, "error watching files"),
            },
        )?;

        Ok(Self {
            context,
            watcher: Mutex::new(watcher),
        })
    }

    pub fn add_workspace(&self, path: &Path) {
        if let Ok(config) = Figment::new()
            .merge(Toml::file(path.join("Rhai.toml")))
            .extract::<RhaiConfig>()
        {
            if let Some(dirs) = config.include_directories {
                for dir in dirs {
                    tracing::debug!(path = ?dir, "watching directory");
                    if let Err(err) = self
                        .watcher
                        .lock()
                        .watch(&path.join(&dir), RecursiveMode::Recursive)
                    {
                        tracing::error!(error = %err, "failed to unwatch directory");
                    } else {
                        documents::collect_documents(self.context.clone(), &dir);
                    }
                }
            }

            return;
        }

        if let Ok(cargo_config) = Figment::new()
            .merge(Toml::file(path.join("Cargo.toml")))
            .extract::<CargoRhaiConfig>()
        {
            if let Some(dirs) = cargo_config
                .project
                .and_then(|c| c.rhai)
                .and_then(|r| r.include_directories)
            {
                for dir in dirs {
                    tracing::debug!(path = ?dir, "watching directory");
                    if let Err(err) = self
                        .watcher
                        .lock()
                        .watch(&path.join(&dir), RecursiveMode::Recursive)
                    {
                        tracing::error!(error = %err, "failed to unwatch directory");
                    } else {
                        documents::collect_documents(self.context.clone(), &dir);
                    }
                }
                return;
            }

            if let Some(dirs) = cargo_config
                .workspace
                .and_then(|c| c.rhai)
                .and_then(|r| r.include_directories)
            {
                for dir in dirs {
                    tracing::debug!(path = ?dir, "watching directory");
                    if let Err(err) = self
                        .watcher
                        .lock()
                        .watch(&path.join(&dir), RecursiveMode::Recursive)
                    {
                        tracing::error!(error = %err, "failed to unwatch directory");
                    } else {
                        documents::collect_documents(self.context.clone(), &dir);
                    }
                }
                return;
            }

            if let Some(dirs) = cargo_config
                .package
                .and_then(|c| c.rhai)
                .and_then(|r| r.include_directories)
            {
                for dir in dirs {
                    tracing::debug!(path = ?dir, "watching directory");
                    if let Err(err) = self
                        .watcher
                        .lock()
                        .watch(&path.join(&dir), RecursiveMode::Recursive)
                    {
                        tracing::error!(error = %err, "failed to unwatch directory");
                    } else {
                        documents::collect_documents(self.context.clone(), &dir);
                    }
                }
                return;
            }
        }

        tracing::debug!(path = ?path, "watching directory");
        if let Err(err) = self.watcher.lock().watch(path, RecursiveMode::Recursive) {
            tracing::error!(error = %err, "failed to unwatch directory");
        } else {
            documents::collect_documents(self.context.clone(), path);
        }
    }

    pub fn remove_workspace(&self, path: &Path) {
        if let Err(err) = self.watcher.lock().unwatch(path) {
            tracing::warn!(error = %err, "failed to unwatch directory");
        }
    }
}
