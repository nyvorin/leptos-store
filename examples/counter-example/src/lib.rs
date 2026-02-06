// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Counter Store Example
//!
//! This example demonstrates how to build a simple counter
//! using leptos-store. It showcases:
//!
//! - Store definition with state, getters, and mutators
//! - Context-based store sharing with `provide_store` and `use_store`
//! - Reactive UI updates
//! - SSR (Server-Side Rendering) support

pub mod components;
pub mod counter_store;

pub use components::*;
pub use counter_store::*;

/// Hydration entry point - called on the client to hydrate the SSR HTML
#[cfg(all(feature = "hydrate", feature = "wasm_entry"))]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(components::App);
}

/// CSR entry point - mounts the app directly (no SSR)
#[cfg(all(feature = "csr", feature = "wasm_entry"))]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(components::App);
}
