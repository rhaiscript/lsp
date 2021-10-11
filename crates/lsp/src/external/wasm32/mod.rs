#![allow(dead_code)]

use crate::{create_server, create_world, World};
use futures::{Future, Sink};
use lsp_async_stub::{rpc::Message, Server};
use once_cell::sync::Lazy;
use std::{io, task::Poll};
use tracing::trace;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

static SERVER: Lazy<Server<World>> = Lazy::new(|| create_server());
static WORLD: Lazy<World> = Lazy::new(|| create_world());

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = __rhai__, js_name = onMessage)]
    fn js_send_message(message: JsValue);
}

#[cfg(feature = "wasm-export")]
#[wasm_bindgen]
pub async fn initialize() {
    console_error_panic_hook::set_once();

    tracing_wasm::set_as_global_default_with_config(tracing_wasm::WASMLayerConfigBuilder::new()
    .set_max_level(tracing::Level::DEBUG)
    .build());
}

#[cfg(feature = "wasm-export")]
#[wasm_bindgen]
pub fn message(message: JsValue) {
    trace!(?message, "received message");
    let msg = message.into_serde().unwrap();
    spawn(async move {
        SERVER
            .handle_message(WORLD.clone(), msg, MessageWriter)
            .await
            .unwrap();
    });
}

pub(crate) fn spawn<F: Future<Output = ()> + 'static>(fut: F) {
    spawn_local(fut)
}

#[derive(Clone)]
struct MessageWriter;

impl Sink<Message> for MessageWriter {
    type Error = io::Error;

    fn poll_ready(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        let js_msg = JsValue::from_serde(&item).unwrap();
        trace!(message = ?js_msg, "sending message");
        js_send_message(js_msg);
        Ok(())
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}
