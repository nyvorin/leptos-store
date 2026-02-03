// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Middleware Example
//!
//! This example demonstrates the middleware system in leptos-store:
//!
//! - Interceptor pattern with before/after hooks
//! - Event bus pattern for observing store operations
//! - Custom middleware for logging and metrics

pub mod components;
#[cfg(any(feature = "hydrate", feature = "ssr"))]
pub mod middleware;
pub mod task_store;

pub use components::*;
pub use task_store::*;

/// Hydration entry point
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(components::App);
}

/// CSR entry point
#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(components::App);
}
