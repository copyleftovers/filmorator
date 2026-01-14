// Leptos components return views used in macros, not directly by callers
#![allow(clippy::must_use_candidate)]

pub mod app;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
