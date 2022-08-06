use environment::WasmEnvironment;
use rhai_common::log::setup_stderr_logging;
use serde::Serialize;
use wasm_bindgen::prelude::*;

mod environment;
mod lsp;

#[derive(Serialize)]
struct Range {
    start: u32,
    end: u32,
}

#[derive(Serialize)]
struct LintError {
    #[serde(skip_serializing_if = "Option::is_none")]
    range: Option<Range>,
    error: String,
}

#[derive(Serialize)]
struct LintResult {
    errors: Vec<LintError>,
}

#[wasm_bindgen]
pub fn initialize() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn create_lsp(env: JsValue, lsp_interface: JsValue) -> lsp::RhaiWasmLsp {
    let env = WasmEnvironment::from(env);
    setup_stderr_logging(env.clone(), false, false, None);

    lsp::RhaiWasmLsp {
        server: rhai_lsp::create_server(),
        world: rhai_lsp::create_world(env),
        lsp_interface: lsp::WasmLspInterface::from(lsp_interface),
    }
}
