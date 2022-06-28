use std::path::{Path, PathBuf};

use async_trait::async_trait;
use futures::Future;
use lsp_types::Url;

pub mod native;

#[async_trait(?Send)]
pub trait Environment: Clone + Send + Sync + 'static {
    fn spawn<F>(&self, fut: F)
    where
        F: Future + Send + 'static,
        F::Output: Send;

    fn spawn_local<F>(&self, fut: F)
    where
        F: Future + 'static;

    fn env_var(&self, name: &str) -> Option<String>;

    async fn read_file(&self, path: &Path) -> Result<Vec<u8>, anyhow::Error>;

    fn url_to_file_path(&self, url: &Url) -> Option<PathBuf>;

    fn rhai_files(&self, root: &Path) -> Result<Vec<PathBuf>, anyhow::Error>;
}
