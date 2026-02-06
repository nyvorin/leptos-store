// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Devtools Example
//!
//! This example demonstrates the devtools integration in leptos-store:
//!
//! - Console API (window.__LEPTOS_STORE__)
//! - StoreInspector component
//! - Event tracking

pub mod components;
pub mod counter_store;

pub use components::*;
pub use counter_store::*;

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
