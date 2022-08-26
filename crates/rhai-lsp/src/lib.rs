#![warn(clippy::pedantic)]
#![allow(
    clippy::unused_async,
    clippy::single_match,
    clippy::wildcard_imports,
    clippy::too_many_lines,
    clippy::enum_glob_use,
    clippy::module_name_repetitions,
    clippy::single_match_else,
    clippy::default_trait_access,
    clippy::missing_errors_doc
)]

use std::sync::Arc;
use rhai_common::environment::Environment;
use lsp_async_stub::Server;
use lsp_types::{notification, request};
pub use world::{World, WorldState};

pub(crate) mod config;
pub(crate) mod diagnostics;
pub(crate) mod lsp_ext;
pub(crate) mod utils;
pub(crate) mod world;

mod handlers;

pub(crate) type IndexMap<K, V> = indexmap::IndexMap<K, V, ahash::RandomState>;

#[must_use]
pub fn create_server<E: Environment>() -> Server<World<E>> {
    Server::new()
        .on_request::<request::Initialize, _>(handlers::initialize)
        .on_request::<request::FoldingRangeRequest, _>(handlers::folding_ranges)
        .on_request::<request::GotoDeclaration, _>(handlers::goto_declaration)
        .on_request::<request::GotoDefinition, _>(handlers::goto_definition)
        .on_request::<request::References, _>(handlers::references)
        .on_request::<request::DocumentSymbolRequest, _>(handlers::document_symbols)
        .on_request::<request::HoverRequest, _>(handlers::hover)
        .on_request::<request::SemanticTokensFullRequest, _>(handlers::semantic_tokens)
        .on_request::<request::Completion, _>(handlers::completion)
        .on_request::<request::PrepareRenameRequest, _>(handlers::prepare_rename)
        .on_request::<request::Rename, _>(handlers::rename)
        .on_notification::<notification::Initialized, _>(handlers::initialized)
        .on_notification::<notification::DidOpenTextDocument, _>(handlers::document_open)
        .on_notification::<notification::DidChangeTextDocument, _>(handlers::document_change)
        .on_notification::<notification::DidSaveTextDocument, _>(handlers::document_save)
        .on_notification::<notification::DidCloseTextDocument, _>(handlers::document_close)
        .on_notification::<notification::DidChangeConfiguration, _>(handlers::configuration_change)
        .on_notification::<notification::DidChangeWorkspaceFolders, _>(handlers::workspace_change)
        .on_notification::<notification::DidChangeWatchedFiles, _>(handlers::watched_file_change)
        .on_request::<lsp_ext::request::HirDump, _>(handlers::hir_dump)
        .on_request::<lsp_ext::request::SyntaxTree, _>(handlers::syntax_tree)
        .on_request::<lsp_ext::request::ConvertOffsets, _>(handlers::convert_offsets)
        .build()
}

#[must_use]
pub fn create_world<E: Environment>(env: E) -> World<E> {
    Arc::new(WorldState::new(env))
}
