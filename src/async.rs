// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Async actions support for stores.
//!
//! This module provides infrastructure for async operations in stores,
//! including action builders, state tracking, and error handling.
//!
//! # Conceptual Model
//!
//! Async actions are orchestrators that can:
//! - Perform async operations (API calls, timers, etc.)
//! - Have side effects (logging, analytics, etc.)
//! - Dispatch mutations to update state
//!
//! Async actions **cannot** directly modify state - they must go through
//! mutators to ensure predictable state updates.
//!
//! # Action States
//!
//! ```rust
//! use leptos_store::prelude::*;
//!
//! let state = ActionState::Idle;
//! assert!(state.is_idle());
//!
//! let state = ActionState::Pending;
//! assert!(state.is_pending());
//! assert!(!state.is_finished());
//!
//! let state = ActionState::Success;
//! assert!(state.is_success());
//! assert!(state.is_finished());
//! ```

use futures::future::BoxFuture;
use leptos::prelude::*;
use pin_project_lite::pin_project;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use thiserror::Error;

use crate::store::Store;

/// Errors that can occur during action execution.
#[derive(Debug, Error)]
pub enum ActionError {
    /// The action was cancelled.
    #[error("Action cancelled")]
    Cancelled,

    /// The action timed out.
    #[error("Action timed out after {0}ms")]
    Timeout(u64),

    /// The action failed with a custom error.
    #[error("Action failed: {0}")]
    Failed(String),

    /// Network error during action execution.
    #[error("Network error: {0}")]
    Network(String),

    /// Validation error before action execution.
    #[error("Validation error: {0}")]
    Validation(String),
}

impl ActionError {
    /// Create a failed error with a message.
    pub fn failed(msg: impl Into<String>) -> Self {
        Self::Failed(msg.into())
    }

    /// Create a network error.
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create a validation error.
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
}

/// Result type for actions.
pub type ActionResult<T, E = ActionError> = Result<T, E>;

/// State of an async action.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum ActionState {
    /// Action has not been executed yet.
    #[default]
    Idle,
    /// Action is currently running.
    Pending,
    /// Action completed successfully.
    Success,
    /// Action failed with an error.
    Error,
}

impl ActionState {
    /// Check if the action is idle.
    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    /// Check if the action is pending.
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    /// Check if the action completed successfully.
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if the action failed.
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }

    /// Check if the action is finished (success or error).
    pub fn is_finished(&self) -> bool {
        matches!(self, Self::Success | Self::Error)
    }
}

/// Trait for synchronous actions.
///
/// Actions orchestrate state changes and side effects but do not
/// directly modify state.
///
/// # Rules
///
/// - Actions **cannot** write state directly
/// - Actions **can** dispatch mutators
/// - Actions **can** have side effects
/// - Actions are synchronous
pub trait Action<S: Store> {
    /// The output type produced by this action.
    type Output;

    /// Execute the action.
    fn execute(&self, store: &S) -> Self::Output;
}

/// Trait for async actions.
///
/// Async actions can perform asynchronous operations like API calls,
/// database queries, or timed operations.
///
/// # Rules
///
/// - Async actions **cannot** write state directly
/// - Async actions **can** dispatch mutators
/// - Async actions **can** have side effects
/// - Async actions are asynchronous
///
/// # Example
///
/// ```rust,no_run
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
/// use std::error::Error;
/// use std::fmt;
///
/// // Define store
/// #[derive(Clone, Default)]
/// struct AuthState { token: Option<String> }
///
/// #[derive(Clone)]
/// struct AuthStore { state: RwSignal<AuthState> }
///
/// impl Store for AuthStore {
///     type State = AuthState;
///     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
/// }
///
/// // Define error type
/// #[derive(Debug)]
/// struct AuthError(String);
/// impl fmt::Display for AuthError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "{}", self.0)
///     }
/// }
/// impl Error for AuthError {}
///
/// // Define async action
/// pub struct LoginAction {
///     pub email: String,
///     pub password: String,
/// }
///
/// impl AsyncAction<AuthStore> for LoginAction {
///     type Output = String;
///     type Error = AuthError;
///
///     async fn execute(&self, _store: &AuthStore) -> ActionResult<Self::Output, Self::Error> {
///         // Simulate API call
///         Ok("token123".to_string())
///     }
/// }
/// ```
pub trait AsyncAction<S: Store>: Send + Sync {
    /// The output type produced by this action on success.
    type Output: Send;

    /// The error type that can be returned on failure.
    type Error: Send + std::error::Error;

    /// Execute the action asynchronously.
    fn execute(
        &self,
        store: &S,
    ) -> impl Future<Output = ActionResult<Self::Output, Self::Error>> + Send;
}

/// A boxed async action for type erasure.
pub type BoxedAsyncAction<S, O, E> =
    Box<dyn Fn(&S) -> BoxFuture<'static, ActionResult<O, E>> + Send + Sync>;

/// Builder for constructing async actions with fluent API.
///
/// # Example
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
///
/// #[derive(Clone, Default)]
/// struct MyState { value: i32 }
///
/// #[derive(Clone)]
/// struct MyStore { state: RwSignal<MyState> }
///
/// impl Store for MyStore {
///     type State = MyState;
///     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
/// }
///
/// let builder: AsyncActionBuilder<MyStore, (), ActionError> = AsyncActionBuilder::new()
///     .with_timeout(5000)
///     .with_retry(3);
///
/// assert_eq!(builder.timeout_ms(), Some(5000));
/// assert_eq!(builder.retry_count(), 3);
/// ```
pub struct AsyncActionBuilder<S: Store, O, E> {
    timeout_ms: Option<u64>,
    retry_count: u32,
    _marker: PhantomData<(S, O, E)>,
}

impl<S: Store, O, E> Default for AsyncActionBuilder<S, O, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Store, O, E> AsyncActionBuilder<S, O, E> {
    /// Create a new async action builder.
    pub fn new() -> Self {
        Self {
            timeout_ms: None,
            retry_count: 0,
            _marker: PhantomData,
        }
    }

    /// Set a timeout for the action in milliseconds.
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }

    /// Set the number of retry attempts.
    pub fn with_retry(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }

    /// Get the configured timeout.
    pub fn timeout_ms(&self) -> Option<u64> {
        self.timeout_ms
    }

    /// Get the configured retry count.
    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }
}

pin_project! {
    /// A future that wraps an async action execution.
    pub struct ActionFuture<F> {
        #[pin]
        inner: F,
        state: ActionState,
    }
}

impl<F> ActionFuture<F> {
    /// Create a new action future.
    pub fn new(inner: F) -> Self {
        Self {
            inner,
            state: ActionState::Pending,
        }
    }

    /// Get the current state of the action.
    pub fn state(&self) -> &ActionState {
        &self.state
    }
}

impl<F, T, E> Future for ActionFuture<F>
where
    F: Future<Output = ActionResult<T, E>>,
{
    type Output = ActionResult<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        match this.inner.poll(cx) {
            Poll::Ready(Ok(value)) => {
                *this.state = ActionState::Success;
                Poll::Ready(Ok(value))
            }
            Poll::Ready(Err(err)) => {
                *this.state = ActionState::Error;
                Poll::Ready(Err(err))
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Reactive action handle for use in components.
///
/// This provides a way to track action state reactively and
/// dispatch actions from event handlers.
#[derive(Clone)]
pub struct ReactiveAction<I, O>
where
    I: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    input: RwSignal<Option<I>>,
    value: RwSignal<Option<O>>,
    pending: RwSignal<bool>,
    version: RwSignal<usize>,
}

impl<I, O> Default for ReactiveAction<I, O>
where
    I: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<I, O> ReactiveAction<I, O>
where
    I: Clone + Send + Sync + 'static,
    O: Clone + Send + Sync + 'static,
{
    /// Create a new reactive action.
    pub fn new() -> Self {
        Self {
            input: RwSignal::new(None),
            value: RwSignal::new(None),
            pending: RwSignal::new(false),
            version: RwSignal::new(0),
        }
    }

    /// Get the current input value.
    pub fn input(&self) -> Option<I> {
        self.input.get()
    }

    /// Get the current output value.
    pub fn value(&self) -> Option<O> {
        self.value.get()
    }

    /// Check if the action is pending.
    pub fn pending(&self) -> bool {
        self.pending.get()
    }

    /// Get the version number (incremented on each dispatch).
    pub fn version(&self) -> usize {
        self.version.get()
    }

    // ========================================================================
    // Mutators - PRIVATE
    // ========================================================================
    //
    // These methods are internal only. External code must use dispatch().
    // This enforces the Enterprise Mode pattern.

    /// Set the input value. (PRIVATE)
    #[allow(dead_code)]
    fn set_input(&self, input: I) {
        self.input.set(Some(input));
    }

    /// Set the output value and mark as not pending. (PRIVATE)
    #[allow(dead_code)]
    fn set_value(&self, value: O) {
        self.value.set(Some(value));
        self.pending.set(false);
    }

    /// Mark the action as pending. (PRIVATE)
    fn set_pending(&self) {
        self.pending.set(true);
        self.version.update(|v| *v += 1);
    }

    /// Clear internal state. (PRIVATE)
    fn clear_internal(&self) {
        self.input.set(None);
        self.value.set(None);
        self.pending.set(false);
    }

    // ========================================================================
    // Actions - PUBLIC API
    // ========================================================================
    //
    // These are the only methods external code should call to modify state.

    /// Dispatch an action with the given input.
    ///
    /// This is the **only** public API for triggering state changes.
    /// It sets the input, marks the action as pending, and returns
    /// a handle for the caller to complete the action.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let action = ReactiveAction::<String, i32>::new();
    ///
    /// // Start the action
    /// let handle = action.dispatch("fetch user 123".to_string());
    ///
    /// // ... perform async work ...
    ///
    /// // Complete with result
    /// handle.complete(42);
    /// ```
    pub fn dispatch(&self, input: I) -> ActionHandle<O> {
        self.set_input(input);
        self.set_pending();
        ActionHandle {
            value: self.value,
            pending: self.pending,
        }
    }

    /// Clear the action state and reset to idle.
    pub fn clear(&self) {
        self.clear_internal();
    }
}

/// Handle returned from `ReactiveAction::dispatch()` to complete the action.
///
/// This is the controlled way to set the action's result value.
#[derive(Clone)]
pub struct ActionHandle<O: Clone + Send + Sync + 'static> {
    value: RwSignal<Option<O>>,
    pending: RwSignal<bool>,
}

impl<O: Clone + Send + Sync + 'static> ActionHandle<O> {
    /// Complete the action with a successful result.
    pub fn complete(self, value: O) {
        self.value.set(Some(value));
        self.pending.set(false);
    }

    /// Complete the action by setting the value (alias for complete).
    pub fn set_value(self, value: O) {
        self.complete(value);
    }

    /// Mark the action as no longer pending without setting a value.
    /// Useful for error cases where you want to clear pending state.
    pub fn cancel(self) {
        self.pending.set(false);
    }
}

/// Extension trait for stores to execute actions.
pub trait StoreActionExt: Store + Sized {
    /// Execute a synchronous action.
    fn dispatch<A>(&self, action: A) -> A::Output
    where
        A: Action<Self>,
    {
        action.execute(self)
    }
}

impl<S: Store> StoreActionExt for S {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_state_default() {
        let state = ActionState::default();
        assert!(state.is_idle());
    }

    #[test]
    fn test_action_state_transitions() {
        assert!(ActionState::Idle.is_idle());
        assert!(!ActionState::Idle.is_pending());
        assert!(!ActionState::Idle.is_finished());

        assert!(ActionState::Pending.is_pending());
        assert!(!ActionState::Pending.is_idle());
        assert!(!ActionState::Pending.is_finished());

        assert!(ActionState::Success.is_success());
        assert!(ActionState::Success.is_finished());

        assert!(ActionState::Error.is_error());
        assert!(ActionState::Error.is_finished());
    }

    #[test]
    fn test_action_error_display() {
        let err = ActionError::Cancelled;
        assert_eq!(err.to_string(), "Action cancelled");

        let err = ActionError::Timeout(5000);
        assert_eq!(err.to_string(), "Action timed out after 5000ms");

        let err = ActionError::failed("Something went wrong");
        assert_eq!(err.to_string(), "Action failed: Something went wrong");

        let err = ActionError::network("Connection refused");
        assert_eq!(err.to_string(), "Network error: Connection refused");

        let err = ActionError::validation("Invalid email");
        assert_eq!(err.to_string(), "Validation error: Invalid email");
    }

    // Note: AsyncActionBuilder requires a Store type, which makes it
    // harder to test in isolation. The builder's functionality is
    // tested through integration tests with real store types.

    #[test]
    fn test_reactive_action_creation() {
        let action: ReactiveAction<String, i32> = ReactiveAction::new();

        assert!(action.input().is_none());
        assert!(action.value().is_none());
        assert!(!action.pending());
        assert_eq!(action.version(), 0);
    }

    #[test]
    fn test_reactive_action_state_changes() {
        let action: ReactiveAction<String, i32> = ReactiveAction::new();

        // Use the public dispatch() API
        let handle = action.dispatch("test".to_string());
        assert_eq!(action.input(), Some("test".to_string()));
        assert!(action.pending());
        assert_eq!(action.version(), 1);

        // Complete the action via the handle
        handle.complete(42);
        assert_eq!(action.value(), Some(42));
        assert!(!action.pending());

        // Clear using the public clear() method
        action.clear();
        assert!(action.input().is_none());
        assert!(action.value().is_none());
    }

    #[test]
    fn test_action_handle_complete() {
        let action: ReactiveAction<String, i32> = ReactiveAction::new();

        let handle = action.dispatch("query".to_string());
        assert!(action.pending());

        handle.complete(100);
        assert!(!action.pending());
        assert_eq!(action.value(), Some(100));
    }

    #[test]
    fn test_action_handle_cancel() {
        let action: ReactiveAction<String, i32> = ReactiveAction::new();

        let handle = action.dispatch("query".to_string());
        assert!(action.pending());

        handle.cancel();
        assert!(!action.pending());
        assert!(action.value().is_none()); // No value set on cancel
    }
}
