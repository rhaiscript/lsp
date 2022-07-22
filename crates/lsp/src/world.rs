use std::{sync::Arc, time::Duration};

use crate::{
    config::{InitConfig, LspConfig, RhaiConfig},
    environment::Environment,
    utils::{Debouncer, Normalize},
    IndexMap,
};
use anyhow::anyhow;
use arc_swap::ArcSwap;
use lsp_async_stub::{rpc, util::Mapper};
use lsp_types::Url;
use once_cell::sync::Lazy;
use rhai_hir::Hir;
use rhai_rowan::{
    parser::{Parse, Parser},
    util::is_rhai_def,
};
use tokio::sync::RwLock as AsyncRwLock;

pub static DEFAULT_WORKSPACE_URL: Lazy<Url> = Lazy::new(|| Url::parse("root:///").unwrap());

pub type World<E> = Arc<WorldState<E>>;

pub struct WorldState<E: Environment> {
    pub(crate) init_config: ArcSwap<InitConfig>,
    pub(crate) env: E,
    pub(crate) workspaces: AsyncRwLock<Workspaces<E>>,
    pub(crate) all_diagnostics_debouncer: Debouncer<E>,
}

impl<E: Environment> WorldState<E> {
    pub fn new(env: E) -> Self {
        let mut ws = Workspaces(IndexMap::default());

        ws.insert(
            DEFAULT_WORKSPACE_URL.clone(),
            Workspace::new(env.clone(), DEFAULT_WORKSPACE_URL.clone()),
        );

        Self {
            init_config: Default::default(),
            all_diagnostics_debouncer: Debouncer::new(Duration::from_secs(1), env.clone()),
            env,
            workspaces: AsyncRwLock::new(ws),
        }
    }
}

#[repr(transparent)]
pub struct Workspaces<E: Environment>(IndexMap<Url, Workspace<E>>);

impl<E: Environment> std::ops::Deref for Workspaces<E> {
    type Target = IndexMap<Url, Workspace<E>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Environment> std::ops::DerefMut for Workspaces<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<E: Environment> Workspaces<E> {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn by_document(&self, url: &Url) -> &Workspace<E> {
        self.0
            .iter()
            .filter(|(key, _)| url.as_str().starts_with(key.as_str()))
            .max_by(|(a, _), (b, _)| a.as_str().len().cmp(&b.as_str().len()))
            .map_or_else(
                || {
                    tracing::warn!(document_url = %url, "using detached workspace");
                    self.0.get(&*DEFAULT_WORKSPACE_URL).unwrap()
                },
                |(_, ws)| ws,
            )
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn by_document_mut(&mut self, url: &Url) -> &mut Workspace<E> {
        self.0
            .iter_mut()
            .filter(|(key, _)| {
                url.as_str().starts_with(key.as_str()) || *key == &*DEFAULT_WORKSPACE_URL
            })
            .max_by(|(a, _), (b, _)| a.as_str().len().cmp(&b.as_str().len()))
            .map(|(k, ws)| {
                if k == &*DEFAULT_WORKSPACE_URL {
                    tracing::warn!(document_url = %url, "using detached workspace");
                }

                ws
            })
            .unwrap()
    }
}

#[allow(dead_code)]
pub struct Workspace<E: Environment> {
    pub(crate) env: E,
    pub(crate) config: LspConfig,
    pub(crate) rhai_config: RhaiConfig,
    pub(crate) root: Url,
    pub(crate) documents: IndexMap<lsp_types::Url, Document>,
    pub(crate) hir: Hir,
}

impl<E: Environment> Workspace<E> {
    pub(crate) fn new(env: E, root: Url) -> Self {
        tracing::info!(%root, "created workspace");
        Self {
            env,
            root,
            rhai_config: Default::default(),
            config: LspConfig::default(),
            documents: Default::default(),
            hir: Default::default(),
        }
    }
}

impl<E: Environment> Workspace<E> {
    pub(crate) fn document(&self, url: &Url) -> Result<&Document, rpc::Error> {
        self.documents
            .get(url)
            .ok_or_else(rpc::Error::invalid_params)
    }

    pub(crate) fn is_detached(&self) -> bool {
        self.root == *DEFAULT_WORKSPACE_URL
    }

    pub(crate) async fn load_rhai_config(&mut self) -> anyhow::Result<()> {
        self.rhai_config = Default::default();

        let root_path = match self.env.url_to_file_path(&self.root) {
            Some(p) => p.normalize(),
            None => return Err(anyhow!("workspace root is not a valid file path")),
        };

        if let Some(config_path) = self.env.discover_rhai_config(&root_path) {
            tracing::info!(path = ?config_path, "found Rhai.toml");
            match self
                .env
                .read_file(&config_path.normalize())
                .await
                .and_then(|v| toml::from_slice(&v).map_err(Into::into))
            {
                Ok(c) => self.rhai_config = c,
                Err(error) => {
                    tracing::error!(%error, "failed to read configuration");
                }
            }
        } else {
            tracing::debug!("no config file found");
        }

        self.rhai_config.prepare(&self.env, &root_path)
    }

    pub(crate) async fn load_all_files(&mut self) {
        let includes = self.rhai_config.source.include.as_ref().unwrap();

        let mut paths = Vec::new();

        let workspace_root = match self.env.url_to_file_path(&self.root) {
            Some(root) => root.normalize(),
            None => {
                tracing::debug!("workspace is not in a valid filesystem");
                return;
            }
        };

        for include_pattern in includes {
            let pattern_paths = match self
                .env
                .glob_files(&workspace_root.join(include_pattern).to_string_lossy())
            {
                Ok(paths) => paths.normalize(),
                Err(error) => {
                    tracing::error!(%error, "failed to load files");
                    continue;
                }
            };

            paths.extend(pattern_paths);
        }

        paths.dedup();

        let all = paths.len();

        if let Some(rule) = &self.rhai_config.source.file_rule {
            paths.retain(|p| rule.is_match(p));
        }

        let excluded = all - paths.len();

        tracing::info!(count = all, excluded, "found files");

        for path in paths {
            tracing::debug!(?path, "found file");

            let document_url = Url::parse(&format!("file://{}", path.to_string_lossy())).unwrap();

            let source = match self.env.read_file(&path).await {
                Ok(src) => src,
                Err(error) => {
                    tracing::error!(%error, "failed to read file");
                    continue;
                }
            };

            let source_text = match String::from_utf8(source) {
                Ok(s) => s,
                Err(error) => {
                    tracing::error!(%error, "given source is not valid UTF-8");
                    continue;
                }
            };

            self.add_document(document_url, &source_text);
        }
    }

    pub fn add_document(&mut self, url: Url, text: &str) {
        let parse = if is_rhai_def(text) {
            Parser::new(text).parse_def()
        } else {
            Parser::new(text).parse_script()
        };

        let mapper = Mapper::new_utf16(text, false);

        let url = url.normalize();

        self.hir.add_source(&url, &parse.clone_syntax());
        self.hir.resolve_references();
        self.documents.insert(url, Document { parse, mapper });
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) parse: Parse,
    pub(crate) mapper: Mapper,
}
