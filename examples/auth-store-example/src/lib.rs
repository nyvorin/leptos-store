// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Auth Store Example
//!
//! This example demonstrates how to build a complete authentication
//! flow using leptos-store. It showcases:
//!
//! - Store definition with state, getters, and mutators
//! - Async actions for login/logout
//! - Context-based store sharing
//! - Reactive UI updates
//! - SSR (Server-Side Rendering) support

pub mod auth_store;
pub mod components;

pub use auth_store::*;
pub use components::*;

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
