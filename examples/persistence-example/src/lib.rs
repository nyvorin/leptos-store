// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Persistence Example
//!
//! This example demonstrates localStorage persistence in leptos-store:
//!
//! - Automatic saving on state changes
//! - Loading persisted state on mount
//! - Serialization with serde

pub mod components;
pub mod notes_store;

pub use components::*;
pub use notes_store::*;

/// Hydration entry point
#[cfg(all(feature = "hydrate", feature = "wasm_entry"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(components::App);
}

/// CSR entry point
#[cfg(all(feature = "csr", feature = "wasm_entry"))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(components::App);
}
