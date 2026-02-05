// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Context management for stores.
//!
//! This module provides utilities for integrating stores with Leptos'
//! context system, enabling stores to be shared across component trees.
//!
//! # Example
//!
//! ```rust,no_run
//! use leptos::prelude::*;
//! use leptos_store::prelude::*;
//!
//! #[derive(Clone, Debug, Default)]
//! struct MyState { name: String }
//!
//! #[derive(Clone)]
//! struct MyStore { state: RwSignal<MyState> }
//!
//! impl MyStore {
//!     fn new() -> Self {
//!         Self { state: RwSignal::new(MyState::default()) }
//!     }
//! }
//!
//! impl Store for MyStore {
//!     type State = MyState;
//!     fn state(&self) -> ReadSignal<Self::State> {
//!         self.state.read_only()
//!     }
//! }
//!
//! #[component]
//! pub fn App() -> impl IntoView {
//!     // Provide the store to all descendants
//!     provide_store(MyStore::new());
//!
//!     view! {
//!         <p>"Hello"</p>
//!     }
//! }
//! ```

use crate::store::{Store, StoreError};
use leptos::prelude::*;
use std::marker::PhantomData;

#[cfg(feature = "hydrate")]
use crate::hydration::{HydratableStore, StoreHydrationError, has_hydration_data, hydrate_store};

/// Provide a store to the component tree via Leptos context.
///
/// This function wraps the store in a way that makes it accessible
/// to all descendant components via `use_store`.
///
/// # Type Parameters
///
/// - `S`: The store type to provide. Must implement [`Store`].
///
/// # Example
///
/// ```rust,no_run
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
///
/// #[derive(Clone, Default)]
/// struct CounterState { count: i32 }
///
/// #[derive(Clone)]
/// struct CounterStore { state: RwSignal<CounterState> }
///
/// impl CounterStore {
///     fn new() -> Self { Self { state: RwSignal::new(CounterState::default()) } }
/// }
///
/// impl Store for CounterStore {
///     type State = CounterState;
///     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
/// }
///
/// #[component]
/// pub fn App() -> impl IntoView {
///     let store = CounterStore::new();
///     provide_store(store);
///     view! { <p>"Counter"</p> }
/// }
/// ```
pub fn provide_store<S: Store + Clone + Send + Sync + 'static>(store: S) {
    provide_context(StoreProvider::new(store));
}

/// Access a store from the Leptos context.
///
/// This function retrieves a store that was previously provided
/// via `provide_store`. It returns a clone of the store.
///
/// # Type Parameters
///
/// - `S`: The store type to retrieve. Must implement [`Store`].
///
/// # Panics
///
/// Panics if the store was not provided in the component tree.
/// Use `try_use_store` for a non-panicking alternative.
///
/// # Example
///
/// ```rust,no_run
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
///
/// #[derive(Clone, Default)]
/// struct CounterState { count: i32 }
///
/// #[derive(Clone)]
/// struct CounterStore { state: RwSignal<CounterState> }
///
/// impl Store for CounterStore {
///     type State = CounterState;
///     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
/// }
///
/// #[component]
/// fn Counter() -> impl IntoView {
///     let store = use_store::<CounterStore>();
///     let count = move || store.state().get().count;
///     view! { <span>{count}</span> }
/// }
/// ```
pub fn use_store<S: Store + Clone + Send + Sync + 'static>() -> S {
    use_context::<StoreProvider<S>>()
        .expect("Store not found in context. Did you forget to call provide_store?")
        .get()
}

/// Try to access a store from the Leptos context.
///
/// This is a non-panicking alternative to `use_store`.
///
/// # Returns
///
/// - `Ok(store)` if the store was found
/// - `Err(StoreError::ContextNotAvailable)` if the store was not provided
///
/// # Example
///
/// ```rust,no_run
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
/// use leptos_store::context::try_use_store;
///
/// #[derive(Clone, Default)]
/// struct CounterState { count: i32 }
///
/// #[derive(Clone)]
/// struct CounterStore { state: RwSignal<CounterState> }
///
/// impl Store for CounterStore {
///     type State = CounterState;
///     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
/// }
///
/// #[component]
/// fn MaybeCounter() -> impl IntoView {
///     match try_use_store::<CounterStore>() {
///         Ok(store) => view! { <span>{move || store.state().get().count}</span> }.into_any(),
///         Err(_) => view! { <span>"No counter store"</span> }.into_any(),
///     }
/// }
/// ```
pub fn try_use_store<S: Store + Clone + Send + Sync + 'static>() -> Result<S, StoreError> {
    use_context::<StoreProvider<S>>()
        .map(|p| p.get())
        .ok_or_else(|| {
            StoreError::ContextNotAvailable(format!(
                "Store {} not found in context",
                std::any::type_name::<S>()
            ))
        })
}

/// Initialize a store for client-side only (CSR) rendering.
///
/// This is the simplest way to set up a store — no server serialization,
/// no hydration, just provide the store into the Leptos context tree.
///
/// # When to Use
///
/// Use this for single-page applications (SPAs) built with `trunk` or similar
/// tools where there is no server-side rendering.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::prelude::*;
///
/// #[component]
/// fn App() -> impl IntoView {
///     let store = MyStore::new();
///     mount_csr_store(store);
///
///     view! { <MainContent /> }
/// }
/// ```
///
/// # CSR vs SSR vs Hydrate
///
/// | Model | Server | Client | State Transfer |
/// |-------|--------|--------|----------------|
/// | CSR | None | `mount_csr_store()` | None needed |
/// | SSR | `provide_store()` | `provide_store()` | Fresh state |
/// | Hydrate | `provide_hydrated_store()` | `use_hydrated_store()` | JSON in HTML |
#[cfg(feature = "csr")]
#[cfg_attr(docsrs, doc(cfg(feature = "csr")))]
pub fn mount_csr_store<S: Store + Clone + Send + Sync + 'static>(store: S) {
    provide_store(store);
}

/// Wrapper for stores in Leptos context.
///
/// This struct wraps a store for use in Leptos' context system.
/// It handles cloning and provides a clean API for store access.
#[derive(Clone)]
pub struct StoreProvider<S: Store> {
    store: S,
    _marker: PhantomData<S>,
}

impl<S: Store + Clone + Send + Sync> StoreProvider<S> {
    /// Create a new store provider.
    pub fn new(store: S) -> Self {
        Self {
            store,
            _marker: PhantomData,
        }
    }

    /// Get a clone of the stored store.
    pub fn get(&self) -> S {
        self.store.clone()
    }
}

impl<S: Store + Clone + Send + Sync> AsRef<S> for StoreProvider<S> {
    fn as_ref(&self) -> &S {
        &self.store
    }
}

/// Extension trait for stores to integrate with context.
pub trait StoreContextExt: Store + Sized {
    /// Provide this store to the component tree.
    fn provide(self)
    where
        Self: Clone + 'static,
    {
        provide_store(self);
    }
}

impl<S: Store> StoreContextExt for S {}

/// A scoped store provider that can be used for nested store instances.
///
/// This is useful when you need multiple instances of the same store type
/// in different parts of your component tree.
#[derive(Clone)]
pub struct ScopedStoreProvider<S: Store, const ID: u64 = 0> {
    store: S,
    _marker: PhantomData<S>,
}

impl<S: Store + Clone + Send + Sync, const ID: u64> ScopedStoreProvider<S, ID> {
    /// Create a new scoped store provider.
    pub fn new(store: S) -> Self {
        Self {
            store,
            _marker: PhantomData,
        }
    }

    /// Provide this scoped store to the context.
    pub fn provide(self) {
        provide_context(self);
    }

    /// Get a clone of the stored store.
    pub fn get(&self) -> S {
        self.store.clone()
    }
}

/// Access a scoped store from context.
///
/// # Type Parameters
///
/// - `S`: The store type
/// - `ID`: The scope identifier (const generic)
pub fn use_scoped_store<S: Store + Clone + Send + Sync + 'static, const ID: u64>() -> S {
    use_context::<ScopedStoreProvider<S, ID>>()
        .expect("Scoped store not found in context")
        .get()
}

/// Provide a scoped store to the context.
pub fn provide_scoped_store<S: Store + Clone + Send + Sync + 'static, const ID: u64>(store: S) {
    provide_context(ScopedStoreProvider::<S, ID>::new(store));
}

// ============================================================================
// Hydration-aware context functions
// ============================================================================

/// Provide a hydratable store to the component tree and render its hydration script.
///
/// This function is used during SSR to:
/// 1. Provide the store to the component tree via context
/// 2. Serialize the store's state to JSON
/// 3. Render a `<script>` tag containing the serialized state
///
/// On the client, use [`use_hydrated_store`] to hydrate the store from this data.
///
/// # Type Parameters
///
/// - `S`: The store type. Must implement [`HydratableStore`].
///
/// # Returns
///
/// An `impl IntoView` that renders the hydration script tag.
///
/// # Example
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
///
/// #[component]
/// pub fn App() -> impl IntoView {
///     let store = MyStore::new();
///     let hydration_script = provide_hydrated_store(store);
///
///     view! {
///         {hydration_script}
///         <MainContent />
///     }
/// }
/// ```
///
/// [`HydratableStore`]: crate::hydration::HydratableStore
#[cfg(feature = "hydrate")]
pub fn provide_hydrated_store<S: HydratableStore + Clone + Send + Sync + 'static>(
    store: S,
) -> impl IntoView {
    use crate::hydration::hydration_script_id;

    // Serialize the state before providing
    let serialized = store.serialize_state();

    // Provide the store to context
    provide_store(store);

    // Return the hydration script
    match serialized {
        Ok(data) => {
            // Escape any script closing tags in the data
            let escaped_data = data.replace("</script>", r"<\/script>");
            leptos::html::script()
                .id(hydration_script_id(S::store_key()))
                .attr("type", "application/json")
                .inner_html(escaped_data)
                .into_any()
        }
        Err(e) => {
            // Log error but don't fail rendering
            leptos::logging::error!("Failed to serialize store for hydration: {}", e);
            ().into_any()
        }
    }
}

/// Access a hydratable store, hydrating from serialized data if available.
///
/// This function is used on the client during hydration to:
/// 1. Check if hydration data exists in the DOM
/// 2. If yes, deserialize and create the store from that data
/// 3. If no, fall back to the regular context lookup
///
/// # Type Parameters
///
/// - `S`: The store type. Must implement [`HydratableStore`].
///
/// # Panics
///
/// Panics if:
/// - Hydration fails and no store was provided via `provide_store`
/// - The store was not found in context at all
///
/// Use [`try_use_hydrated_store`] for a non-panicking alternative.
///
/// # Example
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
///
/// #[component]
/// fn Counter() -> impl IntoView {
///     let store = use_hydrated_store::<CounterStore>();
///     view! { <span>{move || store.state().get().count}</span> }
/// }
/// ```
///
/// [`HydratableStore`]: crate::hydration::HydratableStore
#[cfg(feature = "hydrate")]
pub fn use_hydrated_store<S: HydratableStore + Clone + Send + Sync + 'static>() -> S {
    // First, try to hydrate from DOM
    if has_hydration_data(S::store_key()) {
        match hydrate_store::<S>() {
            Ok(store) => {
                // Provide the hydrated store to context for subsequent uses
                provide_store(store.clone());
                return store;
            }
            Err(e) => {
                leptos::logging::warn!("Hydration failed, falling back to context: {}", e);
            }
        }
    }

    // Fall back to regular context lookup
    use_store::<S>()
}

/// Try to access a hydratable store, hydrating from serialized data if available.
///
/// This is a non-panicking alternative to [`use_hydrated_store`].
///
/// # Returns
///
/// - `Ok(store)` if the store was successfully hydrated or found in context
/// - `Err(StoreHydrationError)` if hydration failed and store not in context
///
/// # Example
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
///
/// #[component]
/// fn MaybeCounter() -> impl IntoView {
///     match try_use_hydrated_store::<CounterStore>() {
///         Ok(store) => view! { <span>{move || store.state().get().count}</span> }.into_any(),
///         Err(_) => view! { <span>"No counter"</span> }.into_any(),
///     }
/// }
/// ```
///
/// [`HydratableStore`]: crate::hydration::HydratableStore
#[cfg(feature = "hydrate")]
pub fn try_use_hydrated_store<S: HydratableStore + Clone + Send + Sync + 'static>()
-> Result<S, StoreHydrationError> {
    // First, try to hydrate from DOM
    if has_hydration_data(S::store_key()) {
        match hydrate_store::<S>() {
            Ok(store) => {
                // Provide the hydrated store to context for subsequent uses
                provide_store(store.clone());
                return Ok(store);
            }
            Err(e) => {
                leptos::logging::warn!("Hydration failed: {}", e);
                // Fall through to context lookup
            }
        }
    }

    // Fall back to regular context lookup
    try_use_store::<S>().map_err(|e| StoreHydrationError::NotFound(e.to_string()))
}

/// Extension trait for hydratable stores to integrate with context.
#[cfg(feature = "hydrate")]
pub trait HydratableStoreContextExt: HydratableStore + Sized {
    /// Provide this store with hydration support.
    ///
    /// Returns a view that renders the hydration script.
    fn provide_hydrated(self) -> impl IntoView
    where
        Self: Clone + 'static,
    {
        provide_hydrated_store(self)
    }
}

#[cfg(feature = "hydrate")]
impl<S: HydratableStore> HydratableStoreContextExt for S {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    #[derive(Clone, Debug, Default, PartialEq)]
    struct TestState {
        value: i32,
    }

    #[derive(Clone)]
    struct TestStore {
        state: RwSignal<TestState>,
    }

    impl TestStore {
        fn new(initial: i32) -> Self {
            Self {
                state: RwSignal::new(TestState { value: initial }),
            }
        }
    }

    impl Store for TestStore {
        type State = TestState;

        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    #[test]
    fn test_store_provider_creation() {
        let store = TestStore::new(42);
        let provider = StoreProvider::new(store);

        let retrieved = provider.get();
        assert_eq!(retrieved.state.get().value, 42);
    }

    #[test]
    fn test_store_provider_as_ref() {
        let store = TestStore::new(100);
        let provider = StoreProvider::new(store);

        assert_eq!(provider.as_ref().state.get().value, 100);
    }

    #[test]
    fn test_scoped_store_provider() {
        let store = TestStore::new(50);
        let scoped: ScopedStoreProvider<TestStore, 1> = ScopedStoreProvider::new(store);

        let retrieved = scoped.get();
        assert_eq!(retrieved.state.get().value, 50);
    }

    #[test]
    fn test_store_error_context_not_available() {
        let err = StoreError::ContextNotAvailable("TestStore not found".to_string());
        assert!(err.to_string().contains("not available"));
    }
}
