// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Leptos Store Examples Showcase
//!
//! A unified showcase for all leptos-store examples.

pub mod components;
pub mod demos;
pub mod showcase_store;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(components::App);
}
