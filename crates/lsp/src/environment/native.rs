use std::{ffi::OsStr, path::Path};

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

    fn rhai_files(&self, root: &Path) -> Result<Vec<std::path::PathBuf>, anyhow::Error> {
        Ok(ignore::WalkBuilder::new(root)
            .git_ignore(false)
            .hidden(false)
            .build()
            .filter_map(Result::ok)
            .filter_map(|e| {
                if e.path().is_file() && e.path().extension() == Some(OsStr::new("rhai")) {
                    Some(e.path().into())
                } else {
                    None
                }
            })
            .collect())
    }
}
