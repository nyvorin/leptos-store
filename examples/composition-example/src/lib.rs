// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Composition Example
//!
//! This example demonstrates store composition in leptos-store:
//!
//! - RootStore for aggregating multiple domain stores
//! - Derived state from multiple stores
//! - Context-based store access

pub mod components;
pub mod stores;

pub use components::*;
pub use stores::*;

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
