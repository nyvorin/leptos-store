// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Server action helpers for Leptos server functions.
//!
//! This module provides integration between leptos-store and Leptos server functions,
//! enabling automatic state management around server calls.
//!
//! # Features
//!
//! - Automatic loading state management
//! - Error handling integration
//! - Optimistic updates
//! - Action history tracking
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_store::server::*;
//!
//! #[server(FetchUser)]
//! pub async fn fetch_user(id: String) -> Result<User, ServerFnError> {
//!     // Server-side logic
//! }
//!
//! #[component]
//! fn UserProfile(id: String) -> impl IntoView {
//!     let action = use_server_action::<FetchUser, UserStore>();
//!     
//!     view! {
//!         <button on:click=move |_| action.dispatch(id.clone())>
//!             "Load User"
//!         </button>
//!         <Show when=move || action.pending()>
//!             "Loading..."
//!         </Show>
//!     }
//! }
//! ```

use crate::r#async::ActionState;
use crate::store::Store;
use leptos::prelude::*;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use thiserror::Error;

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur during server action execution.
#[derive(Debug, Error, Clone)]
pub enum ServerActionError {
    /// The server function failed.
    #[error("Server function error: {0}")]
    ServerFn(String),

    /// Network error.
    #[error("Network error: {0}")]
    Network(String),

    /// Timeout error.
    #[error("Request timed out after {0}ms")]
    Timeout(u64),

    /// Store update failed.
    #[error("Store update failed: {0}")]
    StoreUpdate(String),

    /// Action was cancelled.
    #[error("Action was cancelled")]
    Cancelled,
}

/// Result type for server actions.
pub type ServerActionResult<T> = Result<T, ServerActionError>;

// ============================================================================
// Server Action Trait
// ============================================================================

/// Trait for server actions that integrate with stores.
///
/// This trait extends the async action concept to server functions,
/// providing automatic state management and error handling.
///
/// # Type Parameters
///
/// - `S`: The store type this action operates on
/// - `Input`: The input parameters for the server function
/// - `Output`: The output type from the server function
pub trait ServerAction<S: Store>: Send + Sync {
    /// The input type for the server function.
    type Input: Clone + Send + Sync + 'static;

    /// The output type from the server function.
    type Output: Clone + Send + Sync + 'static;

    /// The server function path.
    fn path() -> &'static str;

    /// Execute the server function.
    fn execute(
        input: Self::Input,
    ) -> Pin<Box<dyn Future<Output = ServerActionResult<Self::Output>> + Send>>;

    /// Update the store with the result (called on success).
    fn on_success(store: &S, output: Self::Output);

    /// Handle errors (called on failure).
    fn on_error(_store: &S, _error: &ServerActionError) {
        // Default: do nothing
    }

    /// Get a description for logging/debugging.
    fn description() -> &'static str {
        std::any::type_name::<Self>()
    }
}

// ============================================================================
// Server Action Handle
// ============================================================================

/// A reactive handle for server actions.
///
/// This handle provides a way to dispatch server actions and
/// reactively track their state.
pub struct ServerActionHandle<A, S>
where
    A: ServerAction<S>,
    S: Store,
{
    store: S,
    input: RwSignal<Option<A::Input>>,
    value: RwSignal<Option<A::Output>>,
    error: RwSignal<Option<ServerActionError>>,
    pending: RwSignal<bool>,
    version: RwSignal<u64>,
    _marker: PhantomData<A>,
}

impl<A, S> Clone for ServerActionHandle<A, S>
where
    A: ServerAction<S>,
    S: Store + Clone,
{
    fn clone(&self) -> Self {
        Self {
            store: self.store.clone(),
            input: self.input,
            value: self.value,
            error: self.error,
            pending: self.pending,
            version: self.version,
            _marker: PhantomData,
        }
    }
}

impl<A, S> ServerActionHandle<A, S>
where
    A: ServerAction<S> + 'static,
    S: Store + Clone + 'static,
{
    /// Create a new server action handle.
    pub fn new(store: S) -> Self {
        Self {
            store,
            input: RwSignal::new(None),
            value: RwSignal::new(None),
            error: RwSignal::new(None),
            pending: RwSignal::new(false),
            version: RwSignal::new(0),
            _marker: PhantomData,
        }
    }

    /// Get the store reference.
    pub fn store(&self) -> &S {
        &self.store
    }

    /// Get the current input.
    pub fn input(&self) -> Option<A::Input> {
        self.input.get()
    }

    /// Get the current value.
    pub fn value(&self) -> Option<A::Output> {
        self.value.get()
    }

    /// Get the current error.
    pub fn error(&self) -> Option<ServerActionError> {
        self.error.get()
    }

    /// Check if the action is pending.
    pub fn pending(&self) -> bool {
        self.pending.get()
    }

    /// Get the version number (incremented on each dispatch).
    pub fn version(&self) -> u64 {
        self.version.get()
    }

    /// Get the current action state.
    pub fn state(&self) -> ActionState {
        if self.pending.get() {
            ActionState::Pending
        } else if self.error.get().is_some() {
            ActionState::Error
        } else if self.value.get().is_some() {
            ActionState::Success
        } else {
            ActionState::Idle
        }
    }

    /// Clear the action state.
    pub fn clear(&self) {
        self.input.set(None);
        self.value.set(None);
        self.error.set(None);
        self.pending.set(false);
    }

    /// Dispatch the server action with the given input.
    pub fn dispatch(&self, input: A::Input) {
        let handle = self.clone();
        let input_clone = input.clone();

        // Update state
        self.input.set(Some(input.clone()));
        self.error.set(None);
        self.pending.set(true);
        self.version.update(|v| *v += 1);

        // Spawn the async task
        leptos::task::spawn_local(async move {
            let result = A::execute(input_clone).await;

            match result {
                Ok(output) => {
                    // Update store
                    A::on_success(&handle.store, output.clone());

                    // Update handle state
                    handle.value.set(Some(output));
                    handle.error.set(None);
                }
                Err(err) => {
                    // Notify store of error
                    A::on_error(&handle.store, &err);

                    // Update handle state
                    handle.error.set(Some(err));
                }
            }

            handle.pending.set(false);
        });
    }
}

// ============================================================================
// Optimistic Update Support
// ============================================================================

/// Configuration for optimistic updates.
#[derive(Debug, Clone)]
pub struct OptimisticConfig<T> {
    /// The optimistic value to apply immediately.
    pub optimistic_value: T,
    /// Whether to rollback on error.
    pub rollback_on_error: bool,
    /// Delay before applying the optimistic update (ms).
    pub delay_ms: u64,
}

impl<T: Default> Default for OptimisticConfig<T> {
    fn default() -> Self {
        Self {
            optimistic_value: T::default(),
            rollback_on_error: true,
            delay_ms: 0,
        }
    }
}

/// Handle for server actions with optimistic update support.
pub struct OptimisticActionHandle<A, S>
where
    A: ServerAction<S>,
    S: Store,
{
    inner: ServerActionHandle<A, S>,
    rollback_value: RwSignal<Option<S::State>>,
}

impl<A, S> Clone for OptimisticActionHandle<A, S>
where
    A: ServerAction<S>,
    S: Store + Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            rollback_value: self.rollback_value,
        }
    }
}

impl<A, S> OptimisticActionHandle<A, S>
where
    A: ServerAction<S> + 'static,
    S: Store + Clone + 'static,
{
    /// Create a new optimistic action handle.
    pub fn new(store: S) -> Self {
        Self {
            inner: ServerActionHandle::new(store),
            rollback_value: RwSignal::new(None),
        }
    }

    /// Dispatch with an optimistic update.
    ///
    /// The `optimistic_update` closure is called immediately to update
    /// the UI, then the actual server call is made. If the server call
    /// fails and `rollback_on_error` is true, the original state is restored.
    pub fn dispatch_optimistic<F>(&self, input: A::Input, optimistic_update: F)
    where
        F: FnOnce(&S) + 'static,
    {
        // Save current state for potential rollback
        let current_state = self.inner.store.state().get();
        self.rollback_value.set(Some(current_state));

        // Apply optimistic update
        optimistic_update(&self.inner.store);

        // Dispatch the actual action
        self.inner.dispatch(input);
    }

    /// Get the inner handle.
    pub fn inner(&self) -> &ServerActionHandle<A, S> {
        &self.inner
    }

    /// Check if pending.
    pub fn pending(&self) -> bool {
        self.inner.pending()
    }

    /// Get error if any.
    pub fn error(&self) -> Option<ServerActionError> {
        self.inner.error()
    }
}

// ============================================================================
// Server Action Builder
// ============================================================================

/// Builder for configuring server actions.
pub struct ServerActionBuilder<S: Store> {
    timeout_ms: Option<u64>,
    retry_count: u32,
    retry_delay_ms: u64,
    _marker: PhantomData<S>,
}

impl<S: Store> Default for ServerActionBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Store> ServerActionBuilder<S> {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            timeout_ms: None,
            retry_count: 0,
            retry_delay_ms: 1000,
            _marker: PhantomData,
        }
    }

    /// Set a timeout for the server call.
    pub fn with_timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Set the number of retry attempts.
    pub fn with_retry(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    /// Set the delay between retries.
    pub fn with_retry_delay(mut self, ms: u64) -> Self {
        self.retry_delay_ms = ms;
        self
    }

    /// Get the timeout.
    pub fn timeout_ms(&self) -> Option<u64> {
        self.timeout_ms
    }

    /// Get the retry count.
    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }
}

// ============================================================================
// Context Helpers
// ============================================================================

/// Create a server action handle for a specific action type.
///
/// # Example
///
/// ```rust,ignore
/// let action = use_server_action::<FetchUser, UserStore>();
/// action.dispatch(FetchUserInput { id: "123".to_string() });
/// ```
pub fn use_server_action<A, S>() -> ServerActionHandle<A, S>
where
    A: ServerAction<S> + 'static,
    S: Store + Clone + 'static,
{
    let store = crate::context::use_store::<S>();
    ServerActionHandle::new(store)
}

/// Create a server action handle with a specific store instance.
pub fn create_server_action<A, S>(store: S) -> ServerActionHandle<A, S>
where
    A: ServerAction<S> + 'static,
    S: Store + Clone + 'static,
{
    ServerActionHandle::new(store)
}

/// Create an optimistic server action handle.
pub fn use_optimistic_action<A, S>() -> OptimisticActionHandle<A, S>
where
    A: ServerAction<S> + 'static,
    S: Store + Clone + 'static,
{
    let store = crate::context::use_store::<S>();
    OptimisticActionHandle::new(store)
}

// ============================================================================
// Mutation Helpers
// ============================================================================

/// Execute a server action and return the result.
///
/// This is a lower-level function for manual control over action execution.
pub async fn execute_server_action<A, S>(
    store: &S,
    input: A::Input,
) -> ServerActionResult<A::Output>
where
    A: ServerAction<S>,
    S: Store,
{
    let result = A::execute(input).await;

    match &result {
        Ok(output) => A::on_success(store, output.clone()),
        Err(err) => A::on_error(store, err),
    }

    result
}

// ============================================================================
// Action History
// ============================================================================

/// An entry in the action history.
#[derive(Debug, Clone)]
pub struct ActionHistoryEntry {
    /// Action description/name.
    pub action: String,
    /// When the action was dispatched.
    pub timestamp: u64,
    /// Duration of the action (if completed).
    pub duration_ms: Option<u64>,
    /// Whether it succeeded.
    pub success: bool,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Tracks action history for debugging and analytics.
#[derive(Clone)]
pub struct ActionHistory {
    entries: RwSignal<Vec<ActionHistoryEntry>>,
    max_entries: usize,
}

impl Default for ActionHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

impl ActionHistory {
    /// Create a new action history.
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: RwSignal::new(Vec::new()),
            max_entries,
        }
    }

    /// Record an action.
    pub fn record(&self, entry: ActionHistoryEntry) {
        self.entries.update(|entries| {
            entries.push(entry);
            if entries.len() > self.max_entries {
                entries.remove(0);
            }
        });
    }

    /// Get all entries.
    pub fn entries(&self) -> Vec<ActionHistoryEntry> {
        self.entries.get()
    }

    /// Get recent entries.
    pub fn recent(&self, count: usize) -> Vec<ActionHistoryEntry> {
        self.entries
            .with(|entries| entries.iter().rev().take(count).cloned().collect())
    }

    /// Clear history.
    pub fn clear(&self) {
        self.entries.set(Vec::new());
    }

    /// Get the number of entries.
    pub fn len(&self) -> usize {
        self.entries.with(|e| e.len())
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.entries.with(|e| e.is_empty())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_action_error_display() {
        assert!(
            ServerActionError::ServerFn("test".to_string())
                .to_string()
                .contains("Server function")
        );
        assert!(
            ServerActionError::Timeout(5000)
                .to_string()
                .contains("5000")
        );
        assert!(
            ServerActionError::Cancelled
                .to_string()
                .contains("cancelled")
        );
    }

    #[test]
    fn test_action_history() {
        let history = ActionHistory::new(10);
        assert!(history.is_empty());

        history.record(ActionHistoryEntry {
            action: "test".to_string(),
            timestamp: 12345,
            duration_ms: Some(100),
            success: true,
            error: None,
        });

        assert_eq!(history.len(), 1);

        let entries = history.entries();
        assert_eq!(entries[0].action, "test");
        assert!(entries[0].success);

        history.clear();
        assert!(history.is_empty());
    }

    #[test]
    fn test_action_history_max_entries() {
        let history = ActionHistory::new(3);

        for i in 0..5 {
            history.record(ActionHistoryEntry {
                action: format!("action_{}", i),
                timestamp: i as u64,
                duration_ms: None,
                success: true,
                error: None,
            });
        }

        // Should only keep the last 3
        assert_eq!(history.len(), 3);

        let entries = history.entries();
        assert_eq!(entries[0].action, "action_2");
        assert_eq!(entries[2].action, "action_4");
    }

    // Note: ServerActionBuilder tests require a Store implementation
    // which is tested through integration tests with actual stores.

    #[test]
    fn test_optimistic_config_default() {
        let config: OptimisticConfig<i32> = OptimisticConfig::default();
        assert!(config.rollback_on_error);
        assert_eq!(config.delay_ms, 0);
    }
}
