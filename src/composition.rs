// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Store composition patterns.
//!
//! This module provides patterns for composing multiple stores together:
//!
//! - **Aggregation**: Combine multiple stores into a root store
//! - **Dependencies**: Allow stores to depend on other stores
//! - **Derived Views**: Create read-only views combining multiple stores
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos_store::composition::*;
//!
//! // Create a root store aggregating multiple domain stores
//! let root = RootStore::builder()
//!     .with_store(AuthStore::new())
//!     .with_store(CartStore::new())
//!     .with_store(UiStore::new())
//!     .build();
//!
//! // Access individual stores
//! let auth = root.get::<AuthStore>().unwrap();
//! ```

use crate::store::{Store, StoreError, StoreId};
use leptos::prelude::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;

// ============================================================================
// Type-erased Store
// ============================================================================

/// Trait for type-erased store access.
///
/// This allows storing heterogeneous stores in collections.
pub trait AnyStore: Send + Sync {
    /// Get the store ID.
    fn id(&self) -> StoreId;

    /// Get the store name.
    fn name(&self) -> &'static str;

    /// Get the store as Any for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Get the store's type ID.
    fn type_id(&self) -> TypeId;
}

impl<S: Store + 'static> AnyStore for S {
    fn id(&self) -> StoreId {
        Store::id(self)
    }

    fn name(&self) -> &'static str {
        Store::name(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<S>()
    }
}

// ============================================================================
// Composite Store
// ============================================================================

/// Trait for stores that aggregate multiple child stores.
///
/// Composite stores provide a unified interface for managing multiple
/// domain stores while maintaining type safety.
pub trait CompositeStore: Send + Sync {
    /// Get all child stores.
    fn stores(&self) -> Vec<&dyn AnyStore>;

    /// Get a store by type.
    fn get<S: Store + 'static>(&self) -> Option<&S>;

    /// Check if a store type is present.
    fn contains<S: Store + 'static>(&self) -> bool {
        self.get::<S>().is_some()
    }

    /// Get the number of stores.
    fn len(&self) -> usize {
        self.stores().len()
    }

    /// Check if empty.
    fn is_empty(&self) -> bool {
        self.stores().is_empty()
    }
}

// ============================================================================
// Root Store
// ============================================================================

/// A root store that aggregates multiple domain stores.
///
/// The root store provides a central access point for all stores
/// in an application, enabling dependency injection and unified
/// state management.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::composition::*;
///
/// let root = RootStore::builder()
///     .with_store(AuthStore::new())
///     .with_store(CartStore::new())
///     .build();
///
/// provide_context(root.clone());
///
/// // Later, access a specific store
/// let root = use_context::<RootStore>().unwrap();
/// let auth = root.get::<AuthStore>().unwrap();
/// ```
#[derive(Clone)]
pub struct RootStore {
    stores: Arc<HashMap<TypeId, Arc<dyn AnyStore>>>,
}

impl Default for RootStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RootStore {
    /// Create a new empty root store.
    pub fn new() -> Self {
        Self {
            stores: Arc::new(HashMap::new()),
        }
    }

    /// Create a builder for constructing a root store.
    pub fn builder() -> RootStoreBuilder {
        RootStoreBuilder::new()
    }

    /// Get a store by type.
    pub fn get<S: Store + 'static>(&self) -> Option<&S> {
        self.stores
            .get(&TypeId::of::<S>())
            .and_then(|s| s.as_any().downcast_ref::<S>())
    }

    /// Get a store by type, or panic with a helpful error.
    pub fn expect<S: Store + 'static>(&self) -> &S {
        self.get::<S>().unwrap_or_else(|| {
            panic!(
                "Store {} not found in RootStore. Did you forget to add it?",
                std::any::type_name::<S>()
            )
        })
    }

    /// Check if a store type is registered.
    pub fn contains<S: Store + 'static>(&self) -> bool {
        self.stores.contains_key(&TypeId::of::<S>())
    }

    /// Get the number of registered stores.
    pub fn len(&self) -> usize {
        self.stores.len()
    }

    /// Check if the root store is empty.
    pub fn is_empty(&self) -> bool {
        self.stores.is_empty()
    }

    /// Get all store type IDs.
    pub fn store_types(&self) -> Vec<TypeId> {
        self.stores.keys().copied().collect()
    }

    /// Get all store names.
    pub fn store_names(&self) -> Vec<&'static str> {
        self.stores.values().map(|s| s.name()).collect()
    }
}

impl CompositeStore for RootStore {
    fn stores(&self) -> Vec<&dyn AnyStore> {
        self.stores.values().map(|s| s.as_ref()).collect()
    }

    fn get<S: Store + 'static>(&self) -> Option<&S> {
        RootStore::get(self)
    }
}

impl fmt::Debug for RootStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RootStore")
            .field("stores", &self.store_names())
            .finish()
    }
}

// ============================================================================
// Root Store Builder
// ============================================================================

/// Builder for constructing root stores.
pub struct RootStoreBuilder {
    stores: HashMap<TypeId, Arc<dyn AnyStore>>,
}

impl Default for RootStoreBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RootStoreBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            stores: HashMap::new(),
        }
    }

    /// Add a store to the root.
    pub fn with_store<S: Store + 'static>(mut self, store: S) -> Self {
        self.stores.insert(TypeId::of::<S>(), Arc::new(store));
        self
    }

    /// Add a store wrapped in Arc.
    pub fn with_arc_store<S: Store + 'static>(mut self, store: Arc<S>) -> Self {
        self.stores.insert(TypeId::of::<S>(), store);
        self
    }

    /// Build the root store.
    pub fn build(self) -> RootStore {
        RootStore {
            stores: Arc::new(self.stores),
        }
    }
}

// ============================================================================
// Store Dependencies
// ============================================================================

/// A store that depends on other stores.
///
/// This pattern allows a store to access other stores' state
/// for cross-domain operations.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::composition::*;
///
/// struct CartStore {
///     state: RwSignal<CartState>,
///     auth: StoreDependency<AuthStore>,
/// }
///
/// impl CartStore {
///     pub fn checkout(&self) {
///         // Check if user is authenticated
///         if let Some(auth) = self.auth.get() {
///             if auth.is_authenticated() {
///                 // Process checkout
///             }
///         }
///     }
/// }
/// ```
pub struct StoreDependency<S: Store> {
    store: Option<S>,
    _marker: PhantomData<S>,
}

impl<S: Store> Default for StoreDependency<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Store> Clone for StoreDependency<S> {
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            _marker: PhantomData,
        }
    }
}

impl<S: Store> StoreDependency<S> {
    /// Create an unresolved dependency.
    pub fn new() -> Self {
        Self {
            store: None,
            _marker: PhantomData,
        }
    }

    /// Create a resolved dependency.
    pub fn resolved(store: S) -> Self {
        Self {
            store: Some(store),
            _marker: PhantomData,
        }
    }

    /// Resolve the dependency with a store instance.
    pub fn resolve(&mut self, store: S) {
        self.store = Some(store);
    }

    /// Get the store if resolved.
    pub fn get(&self) -> Option<&S> {
        self.store.as_ref()
    }

    /// Check if the dependency is resolved.
    pub fn is_resolved(&self) -> bool {
        self.store.is_some()
    }

    /// Get the store or panic.
    pub fn expect(&self) -> &S {
        self.store.as_ref().unwrap_or_else(|| {
            panic!(
                "Store dependency {} is not resolved",
                std::any::type_name::<S>()
            )
        })
    }
}

impl<S: Store + fmt::Debug> fmt::Debug for StoreDependency<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoreDependency")
            .field("resolved", &self.is_resolved())
            .field("store", &self.store)
            .finish()
    }
}

// ============================================================================
// Derived Views
// ============================================================================

/// A derived view combining data from multiple stores.
///
/// Derived views are read-only computed values that combine state
/// from multiple stores into a single reactive value.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::composition::*;
///
/// let dashboard = DerivedView::new()
///     .with_source(move || auth_store.display_name())
///     .with_source(move || cart_store.item_count())
///     .compute(|(name, count)| {
///         DashboardData { user: name, cart_items: count }
///     });
/// ```
pub struct DerivedView<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    value: Memo<T>,
}

impl<T> Clone for DerivedView<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for DerivedView<T> where T: Clone + Send + Sync + PartialEq + 'static {}

impl<T> DerivedView<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    /// Create a derived view from a computation function.
    pub fn new(f: impl Fn() -> T + Send + Sync + 'static) -> Self {
        Self {
            value: Memo::new(move |_prev| f()),
        }
    }

    /// Get the current value.
    pub fn get(&self) -> T {
        self.value.get()
    }

    /// Access the value with a closure.
    pub fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.value.with(f)
    }

    /// Get the underlying memo.
    pub fn memo(&self) -> Memo<T> {
        self.value
    }
}

impl<T> fmt::Debug for DerivedView<T>
where
    T: Clone + Send + Sync + PartialEq + fmt::Debug + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DerivedView")
            .field("value", &self.value.get())
            .finish()
    }
}

// ============================================================================
// Multi-Store Selector
// ============================================================================

/// A selector that combines data from multiple stores.
///
/// Selectors provide a way to efficiently compute derived state
/// from multiple stores with automatic memoization.
pub struct MultiStoreSelector<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    value: Memo<T>,
}

impl<T> Clone for MultiStoreSelector<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for MultiStoreSelector<T> where T: Clone + Send + Sync + PartialEq + 'static {}

impl<T> MultiStoreSelector<T>
where
    T: Clone + Send + Sync + PartialEq + 'static,
{
    /// Create a selector from two stores.
    pub fn from_two<S1, S2, F>(store1: &S1, store2: &S2, selector: F) -> Self
    where
        S1: Store,
        S2: Store,
        F: Fn(&S1::State, &S2::State) -> T + Send + Sync + 'static,
    {
        let s1 = store1.state();
        let s2 = store2.state();

        Self {
            value: Memo::new(move |_prev| {
                let state1 = s1.get();
                let state2 = s2.get();
                selector(&state1, &state2)
            }),
        }
    }

    /// Create a selector from three stores.
    pub fn from_three<S1, S2, S3, F>(store1: &S1, store2: &S2, store3: &S3, selector: F) -> Self
    where
        S1: Store,
        S2: Store,
        S3: Store,
        F: Fn(&S1::State, &S2::State, &S3::State) -> T + Send + Sync + 'static,
    {
        let s1 = store1.state();
        let s2 = store2.state();
        let s3 = store3.state();

        Self {
            value: Memo::new(move |_prev| {
                let state1 = s1.get();
                let state2 = s2.get();
                let state3 = s3.get();
                selector(&state1, &state2, &state3)
            }),
        }
    }

    /// Get the current value.
    pub fn get(&self) -> T {
        self.value.get()
    }

    /// Access the value with a closure.
    pub fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U {
        self.value.with(f)
    }
}

// ============================================================================
// Store Group
// ============================================================================

/// A group of related stores.
///
/// Store groups provide a way to organize stores by domain or feature
/// while maintaining type-safe access.
pub struct StoreGroup {
    name: &'static str,
    stores: HashMap<TypeId, Arc<dyn AnyStore>>,
}

impl StoreGroup {
    /// Create a new store group.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            stores: HashMap::new(),
        }
    }

    /// Add a store to the group.
    pub fn add<S: Store + 'static>(&mut self, store: S) {
        self.stores.insert(TypeId::of::<S>(), Arc::new(store));
    }

    /// Get a store from the group.
    pub fn get<S: Store + 'static>(&self) -> Option<&S> {
        self.stores
            .get(&TypeId::of::<S>())
            .and_then(|s| s.as_any().downcast_ref::<S>())
    }

    /// Get the group name.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Get the number of stores in the group.
    pub fn len(&self) -> usize {
        self.stores.len()
    }

    /// Check if the group is empty.
    pub fn is_empty(&self) -> bool {
        self.stores.is_empty()
    }
}

impl fmt::Debug for StoreGroup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoreGroup")
            .field("name", &self.name)
            .field("count", &self.stores.len())
            .finish()
    }
}

// ============================================================================
// Context Helpers
// ============================================================================

/// Provide a root store to the component tree.
pub fn provide_root_store(root: RootStore) {
    provide_context(root);
}

/// Access the root store from context.
pub fn use_root_store() -> RootStore {
    use_context::<RootStore>().expect("RootStore not found in context")
}

/// Try to access the root store from context.
pub fn try_use_root_store() -> Result<RootStore, StoreError> {
    use_context::<RootStore>()
        .ok_or_else(|| StoreError::ContextNotAvailable("RootStore not found".to_string()))
}

/// Access a specific store from the root store context.
pub fn use_store_from_root<S>() -> S
where
    S: Store + Clone + 'static,
{
    let root = use_root_store();
    root.get::<S>().cloned().unwrap_or_else(|| {
        panic!(
            "Store {} not found in RootStore",
            std::any::type_name::<S>()
        )
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Default)]
    struct AuthState {
        user: Option<String>,
    }

    #[derive(Clone)]
    struct AuthStore {
        state: RwSignal<AuthState>,
    }

    impl AuthStore {
        fn new() -> Self {
            Self {
                state: RwSignal::new(AuthState::default()),
            }
        }

        #[allow(dead_code)]
        fn is_authenticated(&self) -> bool {
            self.state.with(|s| s.user.is_some())
        }
    }

    impl Store for AuthStore {
        type State = AuthState;

        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    #[derive(Clone, Debug, Default)]
    struct CartState {
        items: Vec<String>,
    }

    #[derive(Clone)]
    struct CartStore {
        state: RwSignal<CartState>,
    }

    impl CartStore {
        fn new() -> Self {
            Self {
                state: RwSignal::new(CartState::default()),
            }
        }

        #[allow(dead_code)]
        fn item_count(&self) -> usize {
            self.state.with(|s| s.items.len())
        }
    }

    impl Store for CartStore {
        type State = CartState;

        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    #[test]
    fn test_root_store_builder() {
        let root = RootStore::builder()
            .with_store(AuthStore::new())
            .with_store(CartStore::new())
            .build();

        assert_eq!(root.len(), 2);
        assert!(root.contains::<AuthStore>());
        assert!(root.contains::<CartStore>());
    }

    #[test]
    fn test_root_store_get() {
        let root = RootStore::builder().with_store(AuthStore::new()).build();

        let auth = root.get::<AuthStore>();
        assert!(auth.is_some());

        let cart = root.get::<CartStore>();
        assert!(cart.is_none());
    }

    #[test]
    fn test_root_store_expect() {
        let root = RootStore::builder().with_store(AuthStore::new()).build();

        let _auth = root.expect::<AuthStore>();
        // Should not panic
    }

    #[test]
    #[should_panic(expected = "not found")]
    fn test_root_store_expect_missing() {
        let root = RootStore::new();
        let _auth = root.expect::<AuthStore>();
    }

    #[test]
    fn test_store_dependency() {
        let mut dep: StoreDependency<AuthStore> = StoreDependency::new();
        assert!(!dep.is_resolved());

        dep.resolve(AuthStore::new());
        assert!(dep.is_resolved());
        assert!(dep.get().is_some());
    }

    #[test]
    fn test_store_dependency_resolved() {
        let dep = StoreDependency::resolved(AuthStore::new());
        assert!(dep.is_resolved());
    }

    #[test]
    fn test_derived_view() {
        let view = DerivedView::new(|| 42);
        assert_eq!(view.get(), 42);
    }

    #[test]
    fn test_store_group() {
        let mut group = StoreGroup::new("domain");
        assert!(group.is_empty());

        group.add(AuthStore::new());
        assert_eq!(group.len(), 1);
        assert_eq!(group.name(), "domain");

        let auth = group.get::<AuthStore>();
        assert!(auth.is_some());
    }

    #[test]
    fn test_composite_store_trait() {
        let root = RootStore::builder()
            .with_store(AuthStore::new())
            .with_store(CartStore::new())
            .build();

        let stores = root.stores();
        assert_eq!(stores.len(), 2);

        assert!(root.contains::<AuthStore>());
        assert!(!root.is_empty());
    }

    #[test]
    fn test_root_store_store_names() {
        let root = RootStore::builder().with_store(AuthStore::new()).build();

        let names = root.store_names();
        assert_eq!(names.len(), 1);
        assert!(names[0].contains("AuthStore"));
    }

    #[test]
    fn test_multi_store_selector() {
        let auth = AuthStore::new();
        let cart = CartStore::new();

        let selector = MultiStoreSelector::from_two(&auth, &cart, |auth_state, cart_state| {
            (auth_state.user.is_some(), cart_state.items.len())
        });

        let (is_auth, count) = selector.get();
        assert!(!is_auth);
        assert_eq!(count, 0);
    }
}
