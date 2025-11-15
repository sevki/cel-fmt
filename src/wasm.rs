use wasm_bindgen::prelude::*;

use crate::{format_cel, FormatOptions};

/// Initialize the WASM module with panic hook for better error messages
#[wasm_bindgen(start)]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// Format a CEL expression with default options
#[wasm_bindgen]
pub fn format(source: &str) -> Result<String, String> {
    let options = FormatOptions::default();
    format_cel(source, &options).map_err(|e| e.to_string())
}

/// Format a CEL expression with custom options
#[wasm_bindgen]
pub fn format_with_options(
    source: &str,
    max_width: usize,
    indent_width: usize,
    use_tabs: bool,
    trailing_comma: bool,
) -> Result<String, String> {
    let mut options = FormatOptions::default()
        .with_max_width(max_width)
        .with_indent_width(indent_width)
        .with_trailing_comma(trailing_comma);

    if use_tabs {
        options = options.with_tabs();
    }

    format_cel(source, &options).map_err(|e| e.to_string())
}

/// Get the version of the formatter
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
