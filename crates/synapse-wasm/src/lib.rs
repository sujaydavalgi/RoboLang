use serde::{Deserialize, Serialize};
use synapse_core::{check, run, RunOptions};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
struct CheckResponse {
    ok: bool,
    diagnostics: Vec<synapse_core::Diagnostic>,
}

#[derive(Serialize, Deserialize)]
struct RunResponse {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<synapse_core::RunResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    diagnostics: Option<Vec<synapse_core::Diagnostic>>,
}

fn to_js<T: Serialize>(value: &T) -> JsValue {
    serde_wasm_bindgen::to_value(value).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn wasm_check(source: &str) -> JsValue {
    let resp = match check(source) {
        Ok(()) => CheckResponse {
            ok: true,
            diagnostics: vec![],
        },
        Err(e) => CheckResponse {
            ok: false,
            diagnostics: e.diagnostics(),
        },
    };
    to_js(&resp)
}

#[wasm_bindgen]
pub fn wasm_run(source: &str, max_loop_iterations: u32) -> JsValue {
    let resp = match run(
        source,
        RunOptions {
            max_loop_iterations: max_loop_iterations as usize,
            ..Default::default()
        },
    ) {
        Ok(result) => RunResponse {
            ok: true,
            result: Some(result),
            diagnostics: None,
        },
        Err(e) => RunResponse {
            ok: false,
            result: None,
            diagnostics: Some(e.diagnostics()),
        },
    };
    to_js(&resp)
}

#[wasm_bindgen]
pub fn wasm_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
