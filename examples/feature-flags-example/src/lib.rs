// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Feature Flags Example
//!
//! This example demonstrates the feature flag template in leptos-store:
//!
//! - FeatureFlagStore for managing flags
//! - Feature component for conditional rendering
//! - FeatureVariant for A/B testing

pub mod components;

pub use components::*;

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
