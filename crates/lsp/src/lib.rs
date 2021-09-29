use lsp_async_stub::Server;
use lsp_types::{notification, request, Url};
use mapper::Mapper;
use rhai_rowan::parser::Parse;
use std::{collections::HashMap, sync::{Arc, Mutex}};

#[cfg(not(target_arch = "wasm32"))]
#[path = "external/native/mod.rs"]
pub mod external;

#[cfg(target_arch = "wasm32")]
#[path = "external/wasm32/mod.rs"]
pub mod external;

mod diagnostics;
mod handlers;
mod mapper;

#[derive(Debug, Clone)]
pub struct Document {
    parse: Parse,
    mapper: Mapper,
}

#[derive(Default)]
pub struct WorldState {
    documents: HashMap<Url, Document>,
}

pub type World = Arc<Mutex<WorldState>>;

pub fn create_server() -> Server<World> {
    Server::new()
        .on_request::<request::Initialize, _>(handlers::initialize)
        .on_notification::<notification::DidOpenTextDocument, _>(handlers::document_open)
        .on_notification::<notification::DidChangeTextDocument, _>(handlers::document_change)
        .on_notification::<notification::DidCloseTextDocument, _>(handlers::document_close)
        .build()
}

pub fn create_world() -> World {
    Arc::new(Mutex::new(WorldState::default()))
}
