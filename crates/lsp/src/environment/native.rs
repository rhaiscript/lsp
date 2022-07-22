use std::{path::{Path, PathBuf}, time::Duration};

use async_trait::async_trait;
use lsp_types::Url;

use super::Environment;

#[derive(Debug, Clone)]
pub struct NativeEnvironment;

#[async_trait(?Send)]
impl Environment for NativeEnvironment {
    fn spawn<F>(&self, fut: F)
    where
        F: futures::Future + Send + 'static,
        F::Output: Send,
    {
        tokio::spawn(fut);
    }

    fn spawn_local<F>(&self, fut: F)
    where
        F: futures::Future + 'static,
    {
        tokio::task::spawn_local(fut);
    }

    fn env_var(&self, name: &str) -> Option<String> {
        std::env::var(name).ok()
    }

    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, anyhow::Error> {
        Ok(tokio::fs::read(path).await?)
    }

    fn url_to_file_path(&self, url: &Url) -> Option<std::path::PathBuf> {
        url.to_file_path().ok()
    }

    fn cwd(&self) -> Option<std::path::PathBuf> {
        std::env::current_dir().ok()
    }

    fn glob_files(&self, pattern: &str) -> Result<Vec<std::path::PathBuf>, anyhow::Error> {
        let paths = glob::glob_with(
            pattern,
            glob::MatchOptions {
                case_sensitive: true,
                ..Default::default()
            },
        )?;
        Ok(paths.filter_map(Result::ok).collect())
    }

    fn is_absolute(&self, base: &std::path::Path) -> bool {
        base.is_absolute()
    }

    fn discover_rhai_config(&self, root: &Path) -> Option<PathBuf> {
        let path = root.join("Rhai.toml");

        if let Ok(meta) = std::fs::metadata(root.join("Rhai.toml")) {
            if meta.is_file() {
                return Some(path);
            }
        }

        None
    }

    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}
