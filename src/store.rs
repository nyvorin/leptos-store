// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Core store traits and types.
//!
//! This module provides the foundational abstractions for building stores:
//!
//! - [`Store`] - The main trait that all stores implement
//! - [`StoreBuilder`] - Builder pattern for constructing stores
//! - [`Getter`] - Trait for derived, read-only computed values
//! - [`Mutator`] - Trait for pure, synchronous state mutations
//! - [`StoreRegistry`] - Registry for managing multiple stores

use leptos::prelude::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;
use std::sync::Arc;
use thiserror::Error;

/// Unique identifier for a store instance.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct StoreId {
    type_id: TypeId,
    instance_id: u64,
}

impl StoreId {
    /// Create a new store ID for a given type.
    pub fn new<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            instance_id: 0,
        }
    }

    /// Create a new store ID with a specific instance ID.
    pub fn with_instance<T: 'static>(instance_id: u64) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            instance_id,
        }
    }
}

impl fmt::Debug for StoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoreId")
            .field("type_id", &self.type_id)
            .field("instance_id", &self.instance_id)
            .finish()
    }
}

/// Errors that can occur when working with stores.
#[derive(Debug, Error)]
pub enum StoreError {
    /// Store not found in registry.
    #[error("Store not found: {0}")]
    NotFound(String),

    /// Store already exists in registry.
    #[error("Store already exists: {0}")]
    AlreadyExists(String),

    /// Invalid state transition.
    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    /// Mutation failed.
    #[error("Mutation failed: {0}")]
    MutationFailed(String),

    /// Context not available (SSR/hydration issue).
    #[error("Context not available: {0}")]
    ContextNotAvailable(String),
}

/// The core Store trait that all stores must implement.
///
/// A store encapsulates reactive state and provides a read-only view
/// to external consumers. State mutations happen through mutators,
/// not direct signal access.
///
/// # Type Parameters
///
/// - `State`: The type of state this store manages. Must be `Clone` for
///   reactive updates and `'static` for Leptos compatibility.
///
/// # Hydration Support
///
/// For SSR hydration support, implement the `HydratableStore` trait
/// (available with the `hydrate` feature). Your state type will need
/// to derive `serde::Serialize` and `serde::Deserialize`:
///
/// ```rust,ignore
/// #[derive(Clone, Debug, Default, Serialize, Deserialize)]
/// pub struct MyState { ... }
/// ```
///
/// See the `hydration` module (requires `hydrate` feature) for details.
///
/// # Example
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
///
/// #[derive(Clone, Debug, Default)]
/// pub struct UserState {
///     pub name: String,
///     pub email: Option<String>,
/// }
///
/// #[derive(Clone)]
/// pub struct UserStore {
///     state: RwSignal<UserState>,
/// }
///
/// impl Store for UserStore {
///     type State = UserState;
///
///     fn state(&self) -> ReadSignal<Self::State> {
///         self.state.read_only()
///     }
/// }
/// ```
pub trait Store: Clone + Send + Sync + 'static {
    /// The state type managed by this store.
    ///
    /// For hydration support, this type should implement `serde::Serialize`
    /// and `serde::Deserialize` when used with `HydratableStore`.
    type State: Clone + Send + Sync + 'static;

    /// Returns a read-only signal to the store's state.
    ///
    /// This is the only way external code should access state.
    /// Direct write access is prohibited by design.
    fn state(&self) -> ReadSignal<Self::State>;

    /// Returns the store's unique identifier.
    fn id(&self) -> StoreId {
        StoreId::new::<Self>()
    }

    /// Returns the store's name for debugging and logging.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// A read-only view into a store.
///
/// This wrapper ensures that consumers can only read state,
/// not mutate it directly. Mutations must go through the store's
/// mutator methods.
#[derive(Clone)]
pub struct ReadonlyStore<S: Store> {
    inner: S,
}

impl<S: Store> ReadonlyStore<S> {
    /// Create a new read-only store wrapper.
    pub fn new(store: S) -> Self {
        Self { inner: store }
    }

    /// Get the current state.
    pub fn get(&self) -> S::State {
        self.inner.state().get()
    }

    /// Subscribe to state changes with a closure.
    pub fn with<U>(&self, f: impl FnOnce(&S::State) -> U) -> U {
        self.inner.state().with(f)
    }

    /// Get the underlying store (for testing/advanced use).
    pub fn inner(&self) -> &S {
        &self.inner
    }
}

/// Trait for derived, read-only computed values.
///
/// Getters compute derived state from the store's base state.
/// They are memoized and only recompute when dependencies change.
///
/// # Rules
///
/// - Getters **cannot** write state
/// - Getters **cannot** be async
/// - Getters **cannot** have side effects
///
/// # Example
///
/// ```rust
/// use leptos_store::prelude::*;
///
/// #[derive(Clone, Default)]
/// pub struct UserState {
///     pub first_name: String,
///     pub last_name: String,
/// }
///
/// pub struct FullNameGetter;
///
/// impl Getter<UserState, String> for FullNameGetter {
///     fn get(&self, state: &UserState) -> String {
///         format!("{} {}", state.first_name, state.last_name)
///     }
/// }
///
/// // Test it
/// let state = UserState {
///     first_name: "John".to_string(),
///     last_name: "Doe".to_string(),
/// };
/// let getter = FullNameGetter;
/// assert_eq!(getter.get(&state), "John Doe");
/// ```
pub trait Getter<State, Output> {
    /// Compute the derived value from state.
    fn get(&self, state: &State) -> Output;
}

/// Implement Getter for closures.
impl<State, Output, F> Getter<State, Output> for F
where
    F: Fn(&State) -> Output,
{
    fn get(&self, state: &State) -> Output {
        self(state)
    }
}

/// Context provided to mutators during execution.
///
/// This context provides controlled access to state mutation
/// and ensures mutations are tracked and predictable.
pub struct MutatorContext<'a, State> {
    state: &'a mut State,
}

impl<'a, State> MutatorContext<'a, State> {
    /// Create a new mutator context.
    pub fn new(state: &'a mut State) -> Self {
        Self { state }
    }

    /// Get mutable access to state.
    pub fn state_mut(&mut self) -> &mut State {
        self.state
    }

    /// Get read-only access to state.
    pub fn state(&self) -> &State {
        self.state
    }
}

/// Trait for pure, synchronous state mutations.
///
/// Mutators are the **only** way to modify store state.
/// They must be pure functions with no side effects.
///
/// # Rules
///
/// - Mutators **can** write state
/// - Mutators **cannot** be async
/// - Mutators **cannot** have side effects
///
/// # Example
///
/// ```rust
/// use leptos_store::prelude::*;
///
/// #[derive(Clone, Default)]
/// pub struct UserState {
///     pub name: String,
/// }
///
/// pub struct SetNameMutator {
///     pub name: String,
/// }
///
/// impl Mutator<UserState> for SetNameMutator {
///     fn mutate(&self, ctx: &mut MutatorContext<UserState>) {
///         ctx.state_mut().name = self.name.clone();
///     }
/// }
///
/// // Test it
/// let mut state = UserState::default();
/// let mutator = SetNameMutator { name: "Alice".to_string() };
/// {
///     let mut ctx = MutatorContext::new(&mut state);
///     mutator.mutate(&mut ctx);
/// }
/// assert_eq!(state.name, "Alice");
/// ```
pub trait Mutator<State> {
    /// Execute the mutation.
    fn mutate(&self, ctx: &mut MutatorContext<State>);
}

/// Implement Mutator for closures.
impl<State, F> Mutator<State> for F
where
    F: Fn(&mut MutatorContext<State>),
{
    fn mutate(&self, ctx: &mut MutatorContext<State>) {
        self(ctx)
    }
}

/// Builder for constructing stores with fluent API.
///
/// # Example
///
/// ```rust
/// use leptos_store::prelude::*;
///
/// #[derive(Clone, Default)]
/// struct MyState {
///     count: i32,
/// }
///
/// let signal = StoreBuilder::new()
///     .with_state(MyState { count: 42 })
///     .build();
/// ```
pub struct StoreBuilder<State> {
    initial_state: Option<State>,
    _marker: PhantomData<State>,
}

impl<State: Clone + Send + Sync + 'static> Default for StoreBuilder<State> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State: Clone + Send + Sync + 'static> StoreBuilder<State> {
    /// Create a new store builder.
    pub fn new() -> Self {
        Self {
            initial_state: None,
            _marker: PhantomData,
        }
    }

    /// Set the initial state.
    pub fn with_state(mut self, state: State) -> Self {
        self.initial_state = Some(state);
        self
    }

    /// Build the store, returning the reactive signal.
    ///
    /// # Panics
    ///
    /// Panics if no initial state was provided and State doesn't implement Default.
    pub fn build(self) -> RwSignal<State>
    where
        State: Default + Send + Sync,
    {
        let state = self.initial_state.unwrap_or_default();
        RwSignal::new(state)
    }

    /// Build the store with a required initial state.
    ///
    /// # Errors
    ///
    /// Returns an error if no initial state was provided.
    pub fn try_build(self) -> Result<RwSignal<State>, StoreError>
    where
        State: Send + Sync,
    {
        let state = self
            .initial_state
            .ok_or_else(|| StoreError::NotFound("Initial state not provided".to_string()))?;
        Ok(RwSignal::new(state))
    }
}

/// Registry for managing multiple stores.
///
/// The registry provides a central location for storing and retrieving
/// store instances, useful for debugging and hot-reloading.
#[derive(Default)]
pub struct StoreRegistry {
    stores: HashMap<StoreId, Arc<dyn Any + Send + Sync>>,
}

impl StoreRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a store in the registry.
    pub fn register<S: Store + Send + Sync>(&mut self, store: S) -> Result<StoreId, StoreError> {
        let id = store.id();
        if self.stores.contains_key(&id) {
            return Err(StoreError::AlreadyExists(store.name().to_string()));
        }
        self.stores.insert(id, Arc::new(store));
        Ok(id)
    }

    /// Get a store from the registry.
    pub fn get<S: Store + Send + Sync>(&self) -> Option<Arc<S>> {
        let id = StoreId::new::<S>();
        self.stores
            .get(&id)
            .and_then(|s| s.clone().downcast::<S>().ok())
    }

    /// Remove a store from the registry.
    pub fn unregister<S: Store>(&mut self) -> bool {
        let id = StoreId::new::<S>();
        self.stores.remove(&id).is_some()
    }

    /// Check if a store is registered.
    pub fn contains<S: Store>(&self) -> bool {
        let id = StoreId::new::<S>();
        self.stores.contains_key(&id)
    }

    /// Get the number of registered stores.
    pub fn len(&self) -> usize {
        self.stores.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.stores.is_empty()
    }
}

impl fmt::Debug for StoreRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoreRegistry")
            .field("count", &self.stores.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, Default, PartialEq)]
    struct TestState {
        count: i32,
        name: String,
    }

    #[derive(Clone)]
    struct TestStore {
        state: RwSignal<TestState>,
    }

    impl Store for TestStore {
        type State = TestState;

        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    #[test]
    fn test_store_id_creation() {
        let id1 = StoreId::new::<TestStore>();
        let id2 = StoreId::new::<TestStore>();
        assert_eq!(id1, id2);

        let id3 = StoreId::with_instance::<TestStore>(1);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_store_builder() {
        let state: RwSignal<TestState> = StoreBuilder::new()
            .with_state(TestState {
                count: 42,
                name: "test".to_string(),
            })
            .build();

        assert_eq!(state.get().count, 42);
        assert_eq!(state.get().name, "test");
    }

    #[test]
    fn test_store_builder_default() {
        let state: RwSignal<TestState> = StoreBuilder::new().build();

        assert_eq!(state.get().count, 0);
        assert_eq!(state.get().name, "");
    }

    #[test]
    fn test_getter_closure() {
        let state = TestState {
            count: 10,
            name: "Alice".to_string(),
        };

        let doubled = |s: &TestState| s.count * 2;
        assert_eq!(doubled.get(&state), 20);
    }

    #[test]
    fn test_mutator_closure() {
        let mut state = TestState::default();
        let mut ctx = MutatorContext::new(&mut state);

        let increment = |ctx: &mut MutatorContext<TestState>| {
            ctx.state_mut().count += 1;
        };

        increment.mutate(&mut ctx);
        assert_eq!(ctx.state().count, 1);
    }

    #[test]
    fn test_mutator_context() {
        let mut state = TestState {
            count: 5,
            name: "Bob".to_string(),
        };

        {
            let mut ctx = MutatorContext::new(&mut state);
            ctx.state_mut().count = 10;
            ctx.state_mut().name = "Charlie".to_string();
        }

        assert_eq!(state.count, 10);
        assert_eq!(state.name, "Charlie");
    }

    #[test]
    fn test_store_error_display() {
        let err = StoreError::NotFound("TestStore".to_string());
        assert_eq!(err.to_string(), "Store not found: TestStore");

        let err = StoreError::AlreadyExists("TestStore".to_string());
        assert_eq!(err.to_string(), "Store already exists: TestStore");
    }
}
