// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Thin wrapper components that mount each example's `Demo()` component.
//!
//! Each wrapper simply calls the example crate's exported `Demo` function.

use leptos::prelude::*;

#[component]
pub fn CounterDemo() -> impl IntoView {
    counter_example::components::Demo()
}

#[component]
pub fn AuthDemo() -> impl IntoView {
    auth_store_example::components::Demo()
}

#[component]
pub fn TokenExplorerDemo() -> impl IntoView {
    token_explorer_example::components::Demo()
}

#[component]
pub fn MiddlewareDemo() -> impl IntoView {
    middleware_example::components::Demo()
}

#[component]
pub fn PersistenceDemo() -> impl IntoView {
    persistence_example::components::Demo()
}

#[component]
pub fn CompositionDemo() -> impl IntoView {
    composition_example::components::Demo()
}

#[component]
pub fn FeatureFlagsDemo() -> impl IntoView {
    feature_flags_example::components::Demo()
}

#[component]
pub fn DevtoolsDemo() -> impl IntoView {
    devtools_example::components::Demo()
}

#[component]
pub fn CsrDemo() -> impl IntoView {
    csr_example::Demo()
}

#[component]
pub fn SelectorsDemo() -> impl IntoView {
    selectors_example::Demo()
}
