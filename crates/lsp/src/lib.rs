#![warn(clippy::pedantic)]
#![allow(
    clippy::unused_async,
    clippy::single_match,
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::enum_glob_use,
    clippy::cast_possible_truncation,
    clippy::cast_lossless,
    clippy::module_name_repetitions,
    clippy::single_match_else,
    clippy::default_trait_access,
    clippy::missing_errors_doc
)]

use lsp_async_stub::Server;
use lsp_types::{notification, request, Url};
use mapper::Mapper;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use rhai_hir::Hir;
use rhai_rowan::parser::Parse;
use std::{collections::HashMap, sync::Arc};
use watcher::WorkspaceWatcher;

#[cfg(not(target_arch = "wasm32"))]
#[path = "external/native/mod.rs"]
pub mod external;

#[cfg(target_arch = "wasm32")]
#[path = "external/wasm32/mod.rs"]
pub mod external;

mod diagnostics;
mod handlers;

pub mod config;
mod documents;
pub mod lsp_ext;
pub mod mapper;
mod util;
pub mod watcher;

#[derive(Debug, Clone)]
pub struct Document {
    parse: Parse,
    mapper: Mapper,
}

#[derive(Default)]
pub struct WorldState {
    documents: HashMap<Url, Document>,
    hir: Hir,
    watcher: OnceCell<WorkspaceWatcher>,
}

pub type World = Arc<RwLock<WorldState>>;

#[must_use]
pub fn create_server() -> Server<World> {
    Server::new()
        .on_request::<request::Initialize, _>(handlers::initialize)
        .on_request::<request::DocumentSymbolRequest, _>(handlers::document_symbols)
        .on_request::<request::FoldingRangeRequest, _>(handlers::folding_ranges)
        .on_request::<lsp_ext::request::SyntaxTree, _>(handlers::syntax_tree)
        .on_request::<lsp_ext::request::ConvertOffsets, _>(handlers::convert_offsets)
        .on_request::<request::HoverRequest, _>(handlers::hover)
        .on_request::<request::GotoDeclaration, _>(handlers::goto_declaration)
        .on_request::<request::GotoDefinition, _>(handlers::goto_definition)
        .on_request::<request::References, _>(handlers::references)
        .on_request::<request::CodeLensRequest, _>(handlers::code_lens)
        .on_request::<request::SemanticTokensFullRequest, _>(handlers::semantic_tokens)
        .on_request::<request::Completion, _>(handlers::completion)
        .on_notification::<notification::DidOpenTextDocument, _>(handlers::document_open)
        .on_notification::<notification::DidChangeTextDocument, _>(handlers::document_change)
        .on_notification::<notification::DidChangeWorkspaceFolders, _>(handlers::workspace_folders)
        .build()
}

#[must_use]
pub fn create_world() -> World {
    Arc::new(RwLock::new(WorldState::default()))
}
