use std::sync::Arc;

use crate::{
    config::{InitConfig, LspConfig},
    environment::Environment,
    IndexMap,
};
use arc_swap::ArcSwap;
use lsp_async_stub::{rpc, util::Mapper};
use lsp_types::Url;
use once_cell::sync::Lazy;
use rhai_hir::Hir;
use rhai_rowan::parser::Parse;
use tokio::sync::RwLock as AsyncRwLock;

pub static DEFAULT_WORKSPACE_URL: Lazy<Url> = Lazy::new(|| Url::parse("root:///").unwrap());

pub type World<E> = Arc<WorldState<E>>;

pub struct WorldState<E: Environment> {
    pub(crate) init_config: ArcSwap<InitConfig>,
    pub(crate) env: E,
    pub(crate) workspaces: AsyncRwLock<Workspaces<E>>,
}

impl<E: Environment> WorldState<E> {
    pub fn new(env: E) -> Self {
        let mut ws = Workspaces(IndexMap::default());

        ws.insert(
            DEFAULT_WORKSPACE_URL.clone(),
            Workspace {
                env: env.clone(),
                root: DEFAULT_WORKSPACE_URL.clone(),
                config: LspConfig::default(),
                documents: Default::default(),
                hir: Default::default(),
            },
        );

        Self {
            init_config: Default::default(),
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
    pub(crate) root: Url,
    pub(crate) documents: IndexMap<lsp_types::Url, Document>,
    pub(crate) hir: Hir,
}

impl<E: Environment> Workspace<E> {
    pub(crate) fn new(env: E, root: Url) -> Self {
        Self {
            env,
            root,
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
}

#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) parse: Parse,
    pub(crate) mapper: Mapper,
}
