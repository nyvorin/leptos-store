// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! # leptos-store
//!
//! Enterprise-grade, type-enforced state management for Leptos.
//!
//! This crate provides a structured, SSR-safe state management architecture
//! inspired by Vuex and Pinia, translated into idiomatic Rust for Leptos.
//!
//! ## Core Concepts
//!
//! Each store is a **domain module** composed of:
//!
//! 1. **State** - Read-only externally, reactive data container
//! 2. **Getters** - Derived, read-only computed values
//! 3. **Mutators** - Pure, synchronous state mutations
//! 4. **Actions** - Synchronous orchestration with side effects
//! 5. **Async Actions** - Asynchronous orchestration
//!
//! ## Mutation Rules
//!
//! | Layer | Can Write State | Async | Side Effects |
//! |-------|-----------------|-------|--------------|
//! | Components | ❌ | ❌ | ❌ |
//! | Getters | ❌ | ❌ | ❌ |
//! | Mutators | ✅ | ❌ | ❌ |
//! | Actions | ❌ | ❌ | ✅ |
//! | Async Actions | ❌ | ✅ | ✅ |
//!
//! **Only mutators may write state.**
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `ssr` | ✅ Yes | Server-side rendering support |
//! | `hydrate` | ❌ No | SSR hydration with automatic state serialization |
//! | `csr` | ❌ No | Client-side rendering only |
//! | `middleware` | ❌ No | Middleware system, audit trail, store coordination |
//! | `devtools` | ❌ No | DevTools integration with time-travel debugging |
//! | `persist-web` | ❌ No | Browser-based state persistence |
//!
//! ### Choosing Features
//!
//! - **CSR only (SPA)**: Use `features = ["csr"]`
//! - **SSR without hydration**: Use default features (`ssr`)
//! - **Full SSR with hydration**: Use `ssr` on server, `hydrate` on client
//!
//! ### Why is `hydrate` opt-in?
//!
//! The `hydrate` feature adds:
//! - `serde` and `serde_json` for state serialization
//! - `web-sys` and `wasm-bindgen` for DOM access
//! - Approximately 50KB to your WASM bundle
//!
//! If you don't need state transfer from server to client, you can skip this overhead.
//!
//! ## Selectors — Fine-Grained Reactivity
//!
//! Selectors create memoized views into specific slices of store state.
//! Only re-compute when their particular slice changes, preventing
//! unnecessary re-renders.
//!
//! ```rust,ignore
//! let user_name = create_selector(&store, |s| s.user.name.clone());
//! let total = combine_selectors(count, discount, |c, d| c * d);
//! let badge = map_selector(count, |n| format!("{n} items"));
//! let active = filter_selector(count, |n| *n > 0);
//! ```
//!
//! See the [`selectors`] module for full documentation.
//!
//! ## Deployment Models
//!
//! | Model | Feature | Description |
//! |-------|---------|-------------|
//! | **SSR** | `ssr` (default) | Store created per-request on server |
//! | **Hydrate** | `hydrate` | Server renders + serializes; client picks up |
//! | **CSR** | `csr` | Browser-only, no server involvement |
//!
//! ## Available Macros
//!
//! | Macro | Purpose | Feature |
//! |-------|---------|---------|
//! | `define_state!` | Define state structs with default values | - |
//! | `define_hydratable_state!` | Define state with serde derives | `hydrate` |
//! | `define_action!` | Define synchronous action structs | - |
//! | `define_async_action!` | Define async action structs with error types | - |
//! | `impl_store!` | Implement Store trait for an existing type | - |
//! | `impl_hydratable_store!` | Implement HydratableStore trait | `hydrate` |
//! | `store!` | Complete store definition in one macro | - |
//! | `selector!` | Batch-create multiple selectors from a store | - |
//! | `namespace!` | Define typed namespace combining multiple stores | - |
//! | `derive_state_diff!` | Generate `StateDiff` impl for field-level diffing | `middleware` |
//!
//! See the [`macros`] module for detailed documentation and examples.
//!
//! ## Hydration Support
//!
//! When building full SSR applications where state needs to transfer from
//! server to client, enable the `hydrate` feature:
//!
//! ```toml
//! [dependencies]
//! leptos-store = { version = "0.1", default-features = false }
//!
//! [features]
//! ssr = ["leptos-store/ssr"]
//! hydrate = ["leptos-store/hydrate"]
//! ```
//!
//! This enables:
//! - `HydratableStore` trait for state serialization
//! - `provide_hydrated_store()` for server-side state embedding
//! - `use_hydrated_store()` for client-side state recovery
//!
//! See the `hydration` module (requires `hydrate` feature) for implementation details.
//!
//! ## Example
//!
//! ```rust
//! use leptos::prelude::*;
//! use leptos_store::prelude::*;
//!
//! // Define your state
//! #[derive(Clone, Debug, Default)]
//! pub struct CounterState {
//!     pub count: i32,
//! }
//!
//! // Define your store
//! #[derive(Clone)]
//! pub struct CounterStore {
//!     state: RwSignal<CounterState>,
//! }
//!
//! impl Store for CounterStore {
//!     type State = CounterState;
//!
//!     fn state(&self) -> ReadSignal<Self::State> {
//!         self.state.read_only()
//!     }
//! }
//!
//! // Define mutators
//! impl CounterStore {
//!     pub fn increment(&self) {
//!         self.state.update(|s| s.count += 1);
//!     }
//!
//!     pub fn decrement(&self) {
//!         self.state.update(|s| s.count -= 1);
//!     }
//! }
//! ```

// Enable doc_cfg for docs.rs to show feature requirements
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

pub mod r#async;
pub mod context;
pub mod macros;
pub mod store;
pub mod selectors;

#[cfg(feature = "hydrate")]
pub mod hydration;

// New modules for extended functionality
#[cfg(feature = "middleware")]
#[cfg_attr(docsrs, doc(cfg(feature = "middleware")))]
pub mod middleware;

#[cfg(feature = "middleware")]
#[cfg_attr(docsrs, doc(cfg(feature = "middleware")))]
pub mod audit;

#[cfg(feature = "middleware")]
#[cfg_attr(docsrs, doc(cfg(feature = "middleware")))]
pub mod coordination;

#[cfg(feature = "devtools")]
#[cfg_attr(docsrs, doc(cfg(feature = "devtools")))]
pub mod devtools;

#[cfg(any(
    feature = "persist-web",
    feature = "persist-idb",
    feature = "persist-server"
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "persist-web",
        feature = "persist-idb",
        feature = "persist-server"
    )))
)]
pub mod persistence;

pub mod composition;

#[cfg(feature = "server-actions")]
#[cfg_attr(docsrs, doc(cfg(feature = "server-actions")))]
pub mod server;

#[cfg(feature = "templates")]
#[cfg_attr(docsrs, doc(cfg(feature = "templates")))]
pub mod templates;

pub mod prelude;

pub use prelude::*;
