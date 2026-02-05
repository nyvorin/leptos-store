// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Middleware system for store actions and mutations.
//!
//! This module provides both an interceptor pattern (middleware chain) and
//! an event bus pattern for observing store operations.
//!
//! # Interceptor Pattern
//!
//! Middleware can intercept mutations and actions before and after execution:
//!
//! ```rust,ignore
//! use leptos_store::middleware::*;
//!
//! struct LoggingMiddleware;
//!
//! impl<S: Store> Middleware<S> for LoggingMiddleware {
//!     fn before_mutate(&self, ctx: &MiddlewareContext<S>) -> MiddlewareResult {
//!         println!("Before mutation: {}", ctx.mutation_name());
//!         MiddlewareResult::Continue
//!     }
//! }
//! ```
//!
//! # Event Bus Pattern
//!
//! Subscribe to store events for observation without affecting control flow:
//!
//! ```rust,ignore
//! use leptos_store::middleware::*;
//!
//! struct MetricsSubscriber;
//!
//! impl EventSubscriber for MetricsSubscriber {
//!     fn on_event(&self, event: &StoreEvent) {
//!         match event {
//!             StoreEvent::MutationCompleted { name, duration_ms } => {
//!                 record_metric(name, *duration_ms);
//!             }
//!             _ => {}
//!         }
//!     }
//! }
//! ```

use crate::store::{Store, StoreError, StoreId};
use leptos::prelude::Get;
use std::any::TypeId;
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use thiserror::Error;

// ============================================================================
// Cross-platform Timing
// ============================================================================

/// A cross-platform instant that works in both native and WASM.
#[derive(Clone, Copy, Debug)]
pub struct CrossInstant {
    #[cfg(target_arch = "wasm32")]
    millis: f64,
    #[cfg(not(target_arch = "wasm32"))]
    instant: std::time::Instant,
}

impl CrossInstant {
    /// Get the current instant.
    pub fn now() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            let millis = js_sys::Date::now();
            Self { millis }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                instant: std::time::Instant::now(),
            }
        }
    }

    /// Get the duration since this instant was created.
    pub fn elapsed(&self) -> Duration {
        #[cfg(target_arch = "wasm32")]
        {
            let now = js_sys::Date::now();
            let elapsed_ms = now - self.millis;
            Duration::from_millis(elapsed_ms.max(0.0) as u64)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.instant.elapsed()
        }
    }
}

// ============================================================================
// Middleware Errors
// ============================================================================

/// Errors that can occur during middleware execution.
#[derive(Debug, Error, Clone)]
pub enum MiddlewareError {
    /// Middleware rejected the operation.
    #[error("Middleware rejected: {0}")]
    Rejected(String),

    /// Middleware validation failed.
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Middleware timed out.
    #[error("Middleware timed out after {0}ms")]
    Timeout(u64),

    /// Internal middleware error.
    #[error("Middleware error: {0}")]
    Internal(String),
}

// ============================================================================
// Middleware Result
// ============================================================================

/// Result of middleware execution that controls the pipeline flow.
#[derive(Debug, Clone, Default)]
pub enum MiddlewareResult {
    /// Continue to the next middleware or operation.
    #[default]
    Continue,
    /// Skip remaining middleware but execute the operation.
    Skip,
    /// Abort the entire operation.
    Abort(MiddlewareError),
    /// Transform and continue (for advanced use cases).
    Transform,
}

impl MiddlewareResult {
    /// Check if the result allows continuation.
    pub fn should_continue(&self) -> bool {
        matches!(self, Self::Continue | Self::Transform)
    }

    /// Check if the result is an abort.
    pub fn is_abort(&self) -> bool {
        matches!(self, Self::Abort(_))
    }

    /// Get the error if this is an abort result.
    pub fn error(&self) -> Option<&MiddlewareError> {
        match self {
            Self::Abort(e) => Some(e),
            _ => None,
        }
    }
}


// ============================================================================
// Mutation Result
// ============================================================================

/// Result of a mutation operation.
#[derive(Debug, Clone)]
pub struct MutationResult {
    /// Whether the mutation succeeded.
    pub success: bool,
    /// Duration of the mutation.
    pub duration: Duration,
    /// Error message if failed.
    pub error: Option<String>,
}

impl MutationResult {
    /// Create a successful mutation result.
    pub fn success(duration: Duration) -> Self {
        Self {
            success: true,
            duration,
            error: None,
        }
    }

    /// Create a failed mutation result.
    pub fn failure(duration: Duration, error: impl Into<String>) -> Self {
        Self {
            success: false,
            duration,
            error: Some(error.into()),
        }
    }
}

/// Result of an action operation.
#[derive(Debug, Clone)]
pub struct ActionResult {
    /// Whether the action succeeded.
    pub success: bool,
    /// Duration of the action.
    pub duration: Duration,
    /// Error message if failed.
    pub error: Option<String>,
    /// Output type name (for debugging).
    pub output_type: Option<&'static str>,
}

impl ActionResult {
    /// Create a successful action result.
    pub fn success(duration: Duration) -> Self {
        Self {
            success: true,
            duration,
            error: None,
            output_type: None,
        }
    }

    /// Create a successful action result with output type.
    pub fn success_with_output(duration: Duration, output_type: &'static str) -> Self {
        Self {
            success: true,
            duration,
            error: None,
            output_type: Some(output_type),
        }
    }

    /// Create a failed action result.
    pub fn failure(duration: Duration, error: impl Into<String>) -> Self {
        Self {
            success: false,
            duration,
            error: Some(error.into()),
            output_type: None,
        }
    }
}

// ============================================================================
// Middleware Context
// ============================================================================

/// Context provided to middleware during mutation interception.
pub struct MiddlewareContext<'a, S: Store> {
    store: &'a S,
    mutation_name: &'static str,
    timestamp: CrossInstant,
    metadata: ContextMetadata,
}

impl<'a, S: Store> MiddlewareContext<'a, S> {
    /// Create a new middleware context.
    pub fn new(store: &'a S, mutation_name: &'static str) -> Self {
        Self {
            store,
            mutation_name,
            timestamp: CrossInstant::now(),
            metadata: ContextMetadata::default(),
        }
    }

    /// Get a reference to the store.
    pub fn store(&self) -> &S {
        self.store
    }

    /// Get the current state (read-only).
    pub fn state(&self) -> S::State {
        self.store.state().get()
    }

    /// Get the mutation name.
    pub fn mutation_name(&self) -> &'static str {
        self.mutation_name
    }

    /// Get the timestamp when this context was created.
    pub fn timestamp(&self) -> CrossInstant {
        self.timestamp
    }

    /// Get the elapsed time since context creation.
    pub fn elapsed(&self) -> Duration {
        self.timestamp.elapsed()
    }

    /// Get the store's unique identifier.
    pub fn store_id(&self) -> StoreId {
        self.store.id()
    }

    /// Get the store's name.
    pub fn store_name(&self) -> &'static str {
        self.store.name()
    }

    /// Get mutable access to metadata.
    pub fn metadata_mut(&mut self) -> &mut ContextMetadata {
        &mut self.metadata
    }

    /// Get read-only access to metadata.
    pub fn metadata(&self) -> &ContextMetadata {
        &self.metadata
    }
}

/// Context provided to middleware during action interception.
pub struct ActionContext<'a, S: Store> {
    store: &'a S,
    action_type: TypeId,
    action_name: &'static str,
    timestamp: CrossInstant,
    metadata: ContextMetadata,
}

impl<'a, S: Store> ActionContext<'a, S> {
    /// Create a new action context.
    pub fn new(store: &'a S, action_type: TypeId, action_name: &'static str) -> Self {
        Self {
            store,
            action_type,
            action_name,
            timestamp: CrossInstant::now(),
            metadata: ContextMetadata::default(),
        }
    }

    /// Get a reference to the store.
    pub fn store(&self) -> &S {
        self.store
    }

    /// Get the current state (read-only).
    pub fn state(&self) -> S::State {
        self.store.state().get()
    }

    /// Get the action type ID.
    pub fn action_type(&self) -> TypeId {
        self.action_type
    }

    /// Get the action name.
    pub fn action_name(&self) -> &'static str {
        self.action_name
    }

    /// Get the timestamp when this context was created.
    pub fn timestamp(&self) -> CrossInstant {
        self.timestamp
    }

    /// Get the elapsed time since context creation.
    pub fn elapsed(&self) -> Duration {
        self.timestamp.elapsed()
    }

    /// Get the store's unique identifier.
    pub fn store_id(&self) -> StoreId {
        self.store.id()
    }

    /// Get the store's name.
    pub fn store_name(&self) -> &'static str {
        self.store.name()
    }

    /// Get mutable access to metadata.
    pub fn metadata_mut(&mut self) -> &mut ContextMetadata {
        &mut self.metadata
    }

    /// Get read-only access to metadata.
    pub fn metadata(&self) -> &ContextMetadata {
        &self.metadata
    }
}

/// Metadata that can be attached to middleware contexts.
#[derive(Debug, Clone, Default)]
pub struct ContextMetadata {
    /// User-defined tags for filtering/routing.
    pub tags: Vec<String>,
    /// Correlation ID for distributed tracing.
    pub correlation_id: Option<String>,
    /// Parent span ID for tracing.
    pub parent_span_id: Option<String>,
    /// Custom key-value pairs.
    pub custom: std::collections::HashMap<String, String>,
}

impl ContextMetadata {
    /// Create new empty metadata.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tag.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set correlation ID.
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }

    /// Add a custom key-value pair.
    pub fn with_custom(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom.insert(key.into(), value.into());
        self
    }
}

// ============================================================================
// Middleware Trait
// ============================================================================

/// Trait for middleware that intercepts store operations.
///
/// Middleware can observe and modify the execution of mutations and actions.
/// Each method returns a `MiddlewareResult` that controls whether the
/// operation should continue, skip, or abort.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::middleware::*;
/// use leptos_store::store::Store;
///
/// struct ValidationMiddleware;
///
/// impl<S: Store> Middleware<S> for ValidationMiddleware {
///     fn before_mutate(&self, ctx: &MiddlewareContext<S>) -> MiddlewareResult {
///         // Validate state before mutation
///         MiddlewareResult::Continue
///     }
///
///     fn after_mutate(&self, ctx: &MiddlewareContext<S>, result: &MutationResult) {
///         if !result.success {
///             eprintln!("Mutation failed: {:?}", result.error);
///         }
///     }
/// }
/// ```
pub trait Middleware<S: Store>: Send + Sync {
    /// Called before a mutation is executed.
    ///
    /// Return `MiddlewareResult::Continue` to proceed, or `Abort` to cancel.
    fn before_mutate(&self, _ctx: &MiddlewareContext<S>) -> MiddlewareResult {
        MiddlewareResult::Continue
    }

    /// Called after a mutation is executed.
    fn after_mutate(&self, _ctx: &MiddlewareContext<S>, _result: &MutationResult) {}

    /// Called before an action is executed.
    ///
    /// Return `MiddlewareResult::Continue` to proceed, or `Abort` to cancel.
    fn before_action(&self, _ctx: &ActionContext<S>) -> MiddlewareResult {
        MiddlewareResult::Continue
    }

    /// Called after an action is executed.
    fn after_action(&self, _ctx: &ActionContext<S>, _result: &ActionResult) {}

    /// Get the middleware name for debugging.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Get the middleware priority (higher = runs first).
    fn priority(&self) -> i32 {
        0
    }
}

// ============================================================================
// Middleware Chain
// ============================================================================

/// A chain of middleware that processes operations in order.
///
/// Middleware is executed in priority order (highest first) for `before_*` hooks
/// and reverse order for `after_*` hooks.
pub struct MiddlewareChain<S: Store> {
    middleware: Vec<Arc<dyn Middleware<S>>>,
    sorted: bool,
}

impl<S: Store> Default for MiddlewareChain<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: Store> MiddlewareChain<S> {
    /// Create a new empty middleware chain.
    pub fn new() -> Self {
        Self {
            middleware: Vec::new(),
            sorted: true,
        }
    }

    /// Add middleware to the chain.
    pub fn add<M: Middleware<S> + 'static>(&mut self, middleware: M) {
        self.middleware.push(Arc::new(middleware));
        self.sorted = false;
    }

    /// Add middleware wrapped in Arc.
    pub fn add_arc(&mut self, middleware: Arc<dyn Middleware<S>>) {
        self.middleware.push(middleware);
        self.sorted = false;
    }

    /// Get the number of middleware in the chain.
    pub fn len(&self) -> usize {
        self.middleware.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.middleware.is_empty()
    }

    /// Sort middleware by priority (called automatically when needed).
    fn ensure_sorted(&mut self) {
        if !self.sorted {
            self.middleware
                .sort_by_key(|b| std::cmp::Reverse(b.priority()));
            self.sorted = true;
        }
    }

    /// Execute before_mutate on all middleware.
    ///
    /// Returns the first `Abort` result, or `Continue` if all pass.
    pub fn before_mutate(&mut self, ctx: &MiddlewareContext<S>) -> MiddlewareResult {
        self.ensure_sorted();

        for m in &self.middleware {
            let result = m.before_mutate(ctx);
            if result.is_abort() {
                return result;
            }
            if matches!(result, MiddlewareResult::Skip) {
                break;
            }
        }

        MiddlewareResult::Continue
    }

    /// Execute after_mutate on all middleware (reverse order).
    pub fn after_mutate(&mut self, ctx: &MiddlewareContext<S>, result: &MutationResult) {
        self.ensure_sorted();

        for m in self.middleware.iter().rev() {
            m.after_mutate(ctx, result);
        }
    }

    /// Execute before_action on all middleware.
    pub fn before_action(&mut self, ctx: &ActionContext<S>) -> MiddlewareResult {
        self.ensure_sorted();

        for m in &self.middleware {
            let result = m.before_action(ctx);
            if result.is_abort() {
                return result;
            }
            if matches!(result, MiddlewareResult::Skip) {
                break;
            }
        }

        MiddlewareResult::Continue
    }

    /// Execute after_action on all middleware (reverse order).
    pub fn after_action(&mut self, ctx: &ActionContext<S>, result: &ActionResult) {
        self.ensure_sorted();

        for m in self.middleware.iter().rev() {
            m.after_action(ctx, result);
        }
    }
}

impl<S: Store> fmt::Debug for MiddlewareChain<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MiddlewareChain")
            .field("count", &self.middleware.len())
            .field(
                "middleware",
                &self.middleware.iter().map(|m| m.name()).collect::<Vec<_>>(),
            )
            .finish()
    }
}

// ============================================================================
// Event Bus
// ============================================================================

/// Events emitted by stores for observation.
#[derive(Debug, Clone)]
pub enum StoreEvent {
    /// State has changed.
    StateChanged {
        /// The store that changed.
        store_id: StoreId,
        /// Store name for debugging.
        store_name: &'static str,
        /// Timestamp of the change (milliseconds since epoch).
        timestamp: u64,
    },

    /// A mutation has started.
    MutationStarted {
        /// The store being mutated.
        store_id: StoreId,
        /// Name of the mutation.
        name: &'static str,
        /// Timestamp when started.
        timestamp: u64,
    },

    /// A mutation has completed.
    MutationCompleted {
        /// The store that was mutated.
        store_id: StoreId,
        /// Name of the mutation.
        name: &'static str,
        /// Duration in milliseconds.
        duration_ms: u64,
        /// Whether it succeeded.
        success: bool,
    },

    /// An action has been dispatched.
    ActionDispatched {
        /// The store handling the action.
        store_id: StoreId,
        /// Type ID of the action.
        action_type: TypeId,
        /// Name of the action.
        action_name: &'static str,
        /// Timestamp when dispatched.
        timestamp: u64,
    },

    /// An action has completed.
    ActionCompleted {
        /// The store that handled the action.
        store_id: StoreId,
        /// Name of the action.
        action_name: &'static str,
        /// Duration in milliseconds.
        duration_ms: u64,
        /// Whether it succeeded.
        success: bool,
    },

    /// An error occurred.
    Error {
        /// The store where the error occurred.
        store_id: StoreId,
        /// Error description.
        message: String,
        /// Source of the error.
        source: ErrorSource,
    },
}

/// Source of an error event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSource {
    /// Error occurred during mutation.
    Mutation,
    /// Error occurred during action.
    Action,
    /// Error occurred in middleware.
    Middleware,
    /// Error occurred during persistence.
    Persistence,
    /// Unknown or other source.
    Unknown,
}

/// Trait for subscribers that receive store events.
pub trait EventSubscriber: Send + Sync {
    /// Called when a store event occurs.
    fn on_event(&self, event: &StoreEvent);

    /// Get the subscriber name for debugging.
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Filter events this subscriber is interested in.
    ///
    /// Return `true` to receive the event, `false` to skip it.
    fn filter(&self, _event: &StoreEvent) -> bool {
        true
    }
}

/// An event bus for distributing store events to subscribers.
pub struct EventBus {
    subscribers: RwLock<Vec<Arc<dyn EventSubscriber>>>,
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus {
    /// Create a new event bus.
    pub fn new() -> Self {
        Self {
            subscribers: RwLock::new(Vec::new()),
        }
    }

    /// Subscribe to events.
    pub fn subscribe<S: EventSubscriber + 'static>(&self, subscriber: S) {
        if let Ok(mut subs) = self.subscribers.write() {
            subs.push(Arc::new(subscriber));
        }
    }

    /// Subscribe with an Arc.
    pub fn subscribe_arc(&self, subscriber: Arc<dyn EventSubscriber>) {
        if let Ok(mut subs) = self.subscribers.write() {
            subs.push(subscriber);
        }
    }

    /// Emit an event to all subscribers.
    pub fn emit(&self, event: StoreEvent) {
        if let Ok(subs) = self.subscribers.read() {
            for sub in subs.iter() {
                if sub.filter(&event) {
                    sub.on_event(&event);
                }
            }
        }
    }

    /// Get the number of subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.read().map(|s| s.len()).unwrap_or(0)
    }

    /// Clear all subscribers.
    pub fn clear(&self) {
        if let Ok(mut subs) = self.subscribers.write() {
            subs.clear();
        }
    }
}

impl fmt::Debug for EventBus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.subscriber_count();
        f.debug_struct("EventBus")
            .field("subscriber_count", &count)
            .finish()
    }
}

// ============================================================================
// Middleware-enabled Store Wrapper
// ============================================================================

/// A store wrapper that enables middleware support.
///
/// This wrapper adds middleware hooks around mutations and actions
/// while maintaining full compatibility with the underlying store.
pub struct MiddlewareStore<S: Store> {
    inner: S,
    middleware: Arc<RwLock<MiddlewareChain<S>>>,
    event_bus: Arc<EventBus>,
}

impl<S: Store> Clone for MiddlewareStore<S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            middleware: Arc::clone(&self.middleware),
            event_bus: Arc::clone(&self.event_bus),
        }
    }
}

impl<S: Store> MiddlewareStore<S> {
    /// Create a new middleware-enabled store.
    pub fn new(store: S) -> Self {
        Self {
            inner: store,
            middleware: Arc::new(RwLock::new(MiddlewareChain::new())),
            event_bus: Arc::new(EventBus::new()),
        }
    }

    /// Create with a shared event bus.
    pub fn with_event_bus(store: S, event_bus: Arc<EventBus>) -> Self {
        Self {
            inner: store,
            middleware: Arc::new(RwLock::new(MiddlewareChain::new())),
            event_bus,
        }
    }

    /// Get the inner store.
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Get mutable access to the inner store.
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Add middleware to this store.
    pub fn add_middleware<M: Middleware<S> + 'static>(&self, middleware: M) {
        if let Ok(mut chain) = self.middleware.write() {
            chain.add(middleware);
        }
    }

    /// Subscribe to events from this store.
    pub fn subscribe<E: EventSubscriber + 'static>(&self, subscriber: E) {
        self.event_bus.subscribe(subscriber);
    }

    /// Get the event bus.
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// Execute a mutation with middleware hooks.
    ///
    /// Returns `Ok(())` if the mutation succeeded, or an error if
    /// middleware aborted or the mutation failed.
    pub fn mutate<F>(&self, mutation_name: &'static str, mutate_fn: F) -> Result<(), StoreError>
    where
        F: FnOnce(),
    {
        let ctx = MiddlewareContext::new(&self.inner, mutation_name);
        let start = CrossInstant::now();

        // Emit mutation started event
        self.event_bus.emit(StoreEvent::MutationStarted {
            store_id: self.inner.id(),
            name: mutation_name,
            timestamp: current_timestamp_ms(),
        });

        // Run before_mutate middleware
        let before_result = if let Ok(mut chain) = self.middleware.write() {
            chain.before_mutate(&ctx)
        } else {
            MiddlewareResult::Continue
        };

        if let MiddlewareResult::Abort(err) = before_result {
            let result = MutationResult::failure(start.elapsed(), err.to_string());
            if let Ok(mut chain) = self.middleware.write() {
                chain.after_mutate(&ctx, &result);
            }
            self.event_bus.emit(StoreEvent::MutationCompleted {
                store_id: self.inner.id(),
                name: mutation_name,
                duration_ms: start.elapsed().as_millis() as u64,
                success: false,
            });
            return Err(StoreError::MutationFailed(err.to_string()));
        }

        // Execute the mutation
        mutate_fn();

        let result = MutationResult::success(start.elapsed());

        // Run after_mutate middleware
        if let Ok(mut chain) = self.middleware.write() {
            chain.after_mutate(&ctx, &result);
        }

        // Emit completion event
        self.event_bus.emit(StoreEvent::MutationCompleted {
            store_id: self.inner.id(),
            name: mutation_name,
            duration_ms: start.elapsed().as_millis() as u64,
            success: true,
        });

        Ok(())
    }

    /// Execute an action with middleware hooks.
    pub fn dispatch<F, R>(
        &self,
        action_name: &'static str,
        action_type: TypeId,
        action_fn: F,
    ) -> Result<R, StoreError>
    where
        F: FnOnce() -> R,
    {
        let ctx = ActionContext::new(&self.inner, action_type, action_name);
        let start = CrossInstant::now();

        // Emit action dispatched event
        self.event_bus.emit(StoreEvent::ActionDispatched {
            store_id: self.inner.id(),
            action_type,
            action_name,
            timestamp: current_timestamp_ms(),
        });

        // Run before_action middleware
        let before_result = if let Ok(mut chain) = self.middleware.write() {
            chain.before_action(&ctx)
        } else {
            MiddlewareResult::Continue
        };

        if let MiddlewareResult::Abort(err) = before_result {
            let result = ActionResult::failure(start.elapsed(), err.to_string());
            if let Ok(mut chain) = self.middleware.write() {
                chain.after_action(&ctx, &result);
            }
            self.event_bus.emit(StoreEvent::ActionCompleted {
                store_id: self.inner.id(),
                action_name,
                duration_ms: start.elapsed().as_millis() as u64,
                success: false,
            });
            return Err(StoreError::MutationFailed(err.to_string()));
        }

        // Execute the action
        let output = action_fn();

        let result = ActionResult::success_with_output(start.elapsed(), std::any::type_name::<R>());

        // Run after_action middleware
        if let Ok(mut chain) = self.middleware.write() {
            chain.after_action(&ctx, &result);
        }

        // Emit completion event
        self.event_bus.emit(StoreEvent::ActionCompleted {
            store_id: self.inner.id(),
            action_name,
            duration_ms: start.elapsed().as_millis() as u64,
            success: true,
        });

        Ok(output)
    }
}

impl<S: Store> Store for MiddlewareStore<S> {
    type State = S::State;

    fn state(&self) -> leptos::prelude::ReadSignal<Self::State> {
        self.inner.state()
    }

    fn id(&self) -> StoreId {
        self.inner.id()
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }
}

// ============================================================================
// Built-in Middleware: Logging
// ============================================================================

/// Log level for the logging middleware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    /// Trace level - most verbose.
    Trace,
    /// Debug level.
    Debug,
    /// Info level - default.
    #[default]
    Info,
    /// Warn level.
    Warn,
    /// Error level - least verbose.
    Error,
    /// No logging.
    Off,
}

/// Configuration for the logging middleware.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Minimum log level to emit.
    pub level: LogLevel,
    /// Whether to log state before mutations.
    pub log_state_before: bool,
    /// Whether to log state after mutations.
    pub log_state_after: bool,
    /// Whether to log timing information.
    pub log_timing: bool,
    /// Prefix for log messages.
    pub prefix: &'static str,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            log_state_before: false,
            log_state_after: false,
            log_timing: true,
            prefix: "[Store]",
        }
    }
}

/// Logging middleware that outputs store operations to the console.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::middleware::*;
///
/// let store = MiddlewareStore::new(my_store);
/// store.add_middleware(LoggingMiddleware::new());
/// ```
pub struct LoggingMiddleware {
    config: LoggingConfig,
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingMiddleware {
    /// Create a new logging middleware with default configuration.
    pub fn new() -> Self {
        Self {
            config: LoggingConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: LoggingConfig) -> Self {
        Self { config }
    }

    /// Set the log level.
    pub fn with_level(mut self, level: LogLevel) -> Self {
        self.config.level = level;
        self
    }

    /// Enable state logging before mutations.
    pub fn log_state_before(mut self) -> Self {
        self.config.log_state_before = true;
        self
    }

    /// Enable state logging after mutations.
    pub fn log_state_after(mut self) -> Self {
        self.config.log_state_after = true;
        self
    }

    /// Set a custom prefix.
    pub fn with_prefix(mut self, prefix: &'static str) -> Self {
        self.config.prefix = prefix;
        self
    }

    fn should_log(&self) -> bool {
        self.config.level != LogLevel::Off
    }

    fn log(&self, level: LogLevel, message: &str) {
        if self.config.level == LogLevel::Off {
            return;
        }

        // Only log if the message level is >= configured level
        let should_emit = match (level, self.config.level) {
            (LogLevel::Off, _) => false,
            (_, LogLevel::Off) => false,
            (LogLevel::Error, _) => true,
            (LogLevel::Warn, LogLevel::Error) => false,
            (LogLevel::Warn, _) => true,
            (LogLevel::Info, LogLevel::Error | LogLevel::Warn) => false,
            (LogLevel::Info, _) => true,
            (LogLevel::Debug, LogLevel::Error | LogLevel::Warn | LogLevel::Info) => false,
            (LogLevel::Debug, _) => true,
            (LogLevel::Trace, LogLevel::Trace) => true,
            (LogLevel::Trace, _) => false,
        };

        if should_emit {
            // Use leptos logging which works in both WASM and native
            match level {
                LogLevel::Error => leptos::logging::error!("{} {}", self.config.prefix, message),
                LogLevel::Warn => leptos::logging::warn!("{} {}", self.config.prefix, message),
                LogLevel::Debug => {
                    leptos::logging::debug_warn!("{} {}", self.config.prefix, message)
                }
                _ => leptos::logging::log!("{} {}", self.config.prefix, message),
            }
        }
    }
}

impl<S: Store> Middleware<S> for LoggingMiddleware {
    fn before_mutate(&self, ctx: &MiddlewareContext<S>) -> MiddlewareResult {
        if self.should_log() {
            self.log(
                LogLevel::Info,
                &format!("Mutation started: {}", ctx.mutation_name()),
            );

            if self.config.log_state_before {
                self.log(
                    LogLevel::Debug,
                    &format!("State before: (store: {})", ctx.store_name()),
                );
            }
        }
        MiddlewareResult::Continue
    }

    fn after_mutate(&self, ctx: &MiddlewareContext<S>, result: &MutationResult) {
        if self.should_log() {
            let status = if result.success {
                "completed"
            } else {
                "failed"
            };

            if self.config.log_timing {
                self.log(
                    if result.success {
                        LogLevel::Info
                    } else {
                        LogLevel::Error
                    },
                    &format!(
                        "Mutation {}: {} ({:?})",
                        status,
                        ctx.mutation_name(),
                        result.duration
                    ),
                );
            } else {
                self.log(
                    if result.success {
                        LogLevel::Info
                    } else {
                        LogLevel::Error
                    },
                    &format!("Mutation {}: {}", status, ctx.mutation_name()),
                );
            }

            if !result.success
                && let Some(ref err) = result.error
            {
                self.log(LogLevel::Error, &format!("Error: {}", err));
            }

            if self.config.log_state_after {
                self.log(
                    LogLevel::Debug,
                    &format!("State after: (store: {})", ctx.store_name()),
                );
            }
        }
    }

    fn before_action(&self, ctx: &ActionContext<S>) -> MiddlewareResult {
        if self.should_log() {
            self.log(
                LogLevel::Info,
                &format!("Action dispatched: {}", ctx.action_name()),
            );
        }
        MiddlewareResult::Continue
    }

    fn after_action(&self, ctx: &ActionContext<S>, result: &ActionResult) {
        if self.should_log() {
            let status = if result.success {
                "completed"
            } else {
                "failed"
            };

            if self.config.log_timing {
                self.log(
                    if result.success {
                        LogLevel::Info
                    } else {
                        LogLevel::Error
                    },
                    &format!(
                        "Action {}: {} ({:?})",
                        status,
                        ctx.action_name(),
                        result.duration
                    ),
                );
            } else {
                self.log(
                    if result.success {
                        LogLevel::Info
                    } else {
                        LogLevel::Error
                    },
                    &format!("Action {}: {}", status, ctx.action_name()),
                );
            }

            if !result.success
                && let Some(ref err) = result.error
            {
                self.log(LogLevel::Error, &format!("Error: {}", err));
            }
        }
    }

    fn name(&self) -> &'static str {
        "LoggingMiddleware"
    }

    fn priority(&self) -> i32 {
        -100 // Run last in before hooks, first in after hooks
    }
}

// ============================================================================
// Built-in Middleware: Timing
// ============================================================================

/// Timing middleware that tracks operation durations.
///
/// This middleware emits timing events that can be used for performance
/// monitoring and debugging.
pub struct TimingMiddleware {
    /// Threshold in milliseconds above which to warn.
    warn_threshold_ms: u64,
    /// Threshold in milliseconds above which to error.
    error_threshold_ms: u64,
}

impl Default for TimingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl TimingMiddleware {
    /// Create a new timing middleware with default thresholds.
    pub fn new() -> Self {
        Self {
            warn_threshold_ms: 100,
            error_threshold_ms: 1000,
        }
    }

    /// Set the warning threshold.
    pub fn with_warn_threshold(mut self, ms: u64) -> Self {
        self.warn_threshold_ms = ms;
        self
    }

    /// Set the error threshold.
    pub fn with_error_threshold(mut self, ms: u64) -> Self {
        self.error_threshold_ms = ms;
        self
    }
}

impl<S: Store> Middleware<S> for TimingMiddleware {
    fn after_mutate(&self, ctx: &MiddlewareContext<S>, result: &MutationResult) {
        let duration_ms = result.duration.as_millis() as u64;

        if duration_ms >= self.error_threshold_ms {
            leptos::logging::error!(
                "[Timing] Slow mutation: {} took {}ms (threshold: {}ms)",
                ctx.mutation_name(),
                duration_ms,
                self.error_threshold_ms
            );
        } else if duration_ms >= self.warn_threshold_ms {
            leptos::logging::warn!(
                "[Timing] Mutation {} took {}ms",
                ctx.mutation_name(),
                duration_ms
            );
        }
    }

    fn after_action(&self, ctx: &ActionContext<S>, result: &ActionResult) {
        let duration_ms = result.duration.as_millis() as u64;

        if duration_ms >= self.error_threshold_ms {
            leptos::logging::error!(
                "[Timing] Slow action: {} took {}ms (threshold: {}ms)",
                ctx.action_name(),
                duration_ms,
                self.error_threshold_ms
            );
        } else if duration_ms >= self.warn_threshold_ms {
            leptos::logging::warn!(
                "[Timing] Action {} took {}ms",
                ctx.action_name(),
                duration_ms
            );
        }
    }

    fn name(&self) -> &'static str {
        "TimingMiddleware"
    }

    fn priority(&self) -> i32 {
        -50 // Run after most middleware but before logging
    }
}

// ============================================================================
// Built-in Middleware: Validation
// ============================================================================

/// Validation function type for state validation.
pub type ValidationFn<State> = Box<dyn Fn(&State) -> Result<(), String> + Send + Sync>;

/// Validation middleware that runs validators before mutations.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::middleware::*;
///
/// let validator = ValidationMiddleware::new()
///     .add_validator(|state: &MyState| {
///         if state.count < 0 {
///             Err("Count cannot be negative".to_string())
///         } else {
///             Ok(())
///         }
///     });
/// ```
pub struct ValidationMiddleware<State> {
    validators: Vec<ValidationFn<State>>,
}

impl<State> Default for ValidationMiddleware<State> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State> ValidationMiddleware<State> {
    /// Create a new validation middleware.
    pub fn new() -> Self {
        Self {
            validators: Vec::new(),
        }
    }

    /// Add a validator function.
    pub fn add_validator<F>(mut self, validator: F) -> Self
    where
        F: Fn(&State) -> Result<(), String> + Send + Sync + 'static,
    {
        self.validators.push(Box::new(validator));
        self
    }
}

impl<S: Store> Middleware<S> for ValidationMiddleware<S::State> {
    fn before_mutate(&self, ctx: &MiddlewareContext<S>) -> MiddlewareResult {
        let state = ctx.state();

        for validator in &self.validators {
            if let Err(err) = validator(&state) {
                return MiddlewareResult::Abort(MiddlewareError::ValidationFailed(err));
            }
        }

        MiddlewareResult::Continue
    }

    fn name(&self) -> &'static str {
        "ValidationMiddleware"
    }

    fn priority(&self) -> i32 {
        100 // Run early to catch invalid states
    }
}

// ============================================================================
// Built-in Middleware: Tracing (feature-gated)
// ============================================================================

/// Tracing middleware for OpenTelemetry integration.
///
/// This middleware creates spans for mutations and actions, enabling
/// distributed tracing across your application.
///
/// # Feature
///
/// This middleware requires the `tracing` feature to be enabled.
#[cfg(feature = "tracing")]
pub struct TracingMiddleware {
    /// Service name for spans.
    service_name: &'static str,
}

#[cfg(feature = "tracing")]
impl Default for TracingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "tracing")]
impl TracingMiddleware {
    /// Create a new tracing middleware.
    pub fn new() -> Self {
        Self {
            service_name: "leptos-store",
        }
    }

    /// Set the service name for spans.
    pub fn with_service_name(mut self, name: &'static str) -> Self {
        self.service_name = name;
        self
    }
}

#[cfg(feature = "tracing")]
impl<S: Store> Middleware<S> for TracingMiddleware {
    fn before_mutate(&self, ctx: &MiddlewareContext<S>) -> MiddlewareResult {
        tracing::info_span!(
            "store.mutation",
            store = ctx.store_name(),
            mutation = ctx.mutation_name(),
            service = self.service_name,
        );
        MiddlewareResult::Continue
    }

    fn before_action(&self, ctx: &ActionContext<S>) -> MiddlewareResult {
        tracing::info_span!(
            "store.action",
            store = ctx.store_name(),
            action = ctx.action_name(),
            service = self.service_name,
        );
        MiddlewareResult::Continue
    }

    fn name(&self) -> &'static str {
        "TracingMiddleware"
    }

    fn priority(&self) -> i32 {
        200 // Run very early to capture the full span
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get current timestamp in milliseconds.
fn current_timestamp_ms() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Create a middleware context for testing.
#[cfg(test)]
pub fn test_middleware_context<S: Store>(store: &S) -> MiddlewareContext<'_, S> {
    MiddlewareContext::new(store, "test_mutation")
}

/// Create an action context for testing.
#[cfg(test)]
pub fn test_action_context<S: Store>(store: &S) -> ActionContext<'_, S> {
    ActionContext::new(store, TypeId::of::<()>(), "test_action")
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Clone, Debug, Default)]
    struct TestState {
        count: i32,
    }

    #[derive(Clone)]
    struct TestStore {
        state: RwSignal<TestState>,
    }

    impl TestStore {
        fn new() -> Self {
            Self {
                state: RwSignal::new(TestState::default()),
            }
        }
    }

    impl Store for TestStore {
        type State = TestState;

        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    // Test middleware that counts calls
    struct CountingMiddleware {
        before_mutate_count: AtomicU32,
        after_mutate_count: AtomicU32,
    }

    impl CountingMiddleware {
        fn new() -> Self {
            Self {
                before_mutate_count: AtomicU32::new(0),
                after_mutate_count: AtomicU32::new(0),
            }
        }

        fn before_count(&self) -> u32 {
            self.before_mutate_count.load(Ordering::SeqCst)
        }

        fn after_count(&self) -> u32 {
            self.after_mutate_count.load(Ordering::SeqCst)
        }
    }

    impl<S: Store> Middleware<S> for CountingMiddleware {
        fn before_mutate(&self, _ctx: &MiddlewareContext<S>) -> MiddlewareResult {
            self.before_mutate_count.fetch_add(1, Ordering::SeqCst);
            MiddlewareResult::Continue
        }

        fn after_mutate(&self, _ctx: &MiddlewareContext<S>, _result: &MutationResult) {
            self.after_mutate_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    // Test middleware that aborts
    struct AbortingMiddleware;

    impl<S: Store> Middleware<S> for AbortingMiddleware {
        fn before_mutate(&self, _ctx: &MiddlewareContext<S>) -> MiddlewareResult {
            MiddlewareResult::Abort(MiddlewareError::Rejected("Test abort".to_string()))
        }
    }

    #[test]
    fn test_middleware_result_methods() {
        assert!(MiddlewareResult::Continue.should_continue());
        assert!(MiddlewareResult::Transform.should_continue());
        assert!(!MiddlewareResult::Skip.should_continue());
        assert!(
            !MiddlewareResult::Abort(MiddlewareError::Rejected("".to_string())).should_continue()
        );

        assert!(!MiddlewareResult::Continue.is_abort());
        assert!(MiddlewareResult::Abort(MiddlewareError::Rejected("".to_string())).is_abort());

        let abort = MiddlewareResult::Abort(MiddlewareError::Rejected("test".to_string()));
        assert!(abort.error().is_some());
        assert!(MiddlewareResult::Continue.error().is_none());
    }

    #[test]
    fn test_mutation_result() {
        let success = MutationResult::success(Duration::from_millis(10));
        assert!(success.success);
        assert!(success.error.is_none());

        let failure = MutationResult::failure(Duration::from_millis(5), "test error");
        assert!(!failure.success);
        assert_eq!(failure.error, Some("test error".to_string()));
    }

    #[test]
    fn test_action_result() {
        let success = ActionResult::success(Duration::from_millis(10));
        assert!(success.success);
        assert!(success.error.is_none());

        let success_with_output =
            ActionResult::success_with_output(Duration::from_millis(10), "String");
        assert!(success_with_output.success);
        assert_eq!(success_with_output.output_type, Some("String"));

        let failure = ActionResult::failure(Duration::from_millis(5), "action error");
        assert!(!failure.success);
        assert_eq!(failure.error, Some("action error".to_string()));
    }

    #[test]
    fn test_context_metadata() {
        let meta = ContextMetadata::new()
            .with_tag("test")
            .with_correlation_id("abc-123")
            .with_custom("key", "value");

        assert_eq!(meta.tags, vec!["test"]);
        assert_eq!(meta.correlation_id, Some("abc-123".to_string()));
        assert_eq!(meta.custom.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_middleware_chain_add_and_len() {
        let mut chain: MiddlewareChain<TestStore> = MiddlewareChain::new();
        assert!(chain.is_empty());
        assert_eq!(chain.len(), 0);

        chain.add(CountingMiddleware::new());
        assert!(!chain.is_empty());
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn test_middleware_chain_execution() {
        let store = TestStore::new();
        let counting = Arc::new(CountingMiddleware::new());

        let mut chain: MiddlewareChain<TestStore> = MiddlewareChain::new();
        chain.add_arc(counting.clone());

        let ctx = MiddlewareContext::new(&store, "test");

        let result = chain.before_mutate(&ctx);
        assert!(result.should_continue());
        assert_eq!(counting.before_count(), 1);

        chain.after_mutate(&ctx, &MutationResult::success(Duration::from_millis(1)));
        assert_eq!(counting.after_count(), 1);
    }

    #[test]
    fn test_middleware_chain_abort() {
        let store = TestStore::new();

        let mut chain: MiddlewareChain<TestStore> = MiddlewareChain::new();
        chain.add(AbortingMiddleware);

        let ctx = MiddlewareContext::new(&store, "test");
        let result = chain.before_mutate(&ctx);

        assert!(result.is_abort());
    }

    #[test]
    fn test_event_bus() {
        struct TestSubscriber {
            count: AtomicU32,
        }

        impl EventSubscriber for TestSubscriber {
            fn on_event(&self, _event: &StoreEvent) {
                self.count.fetch_add(1, Ordering::SeqCst);
            }
        }

        let bus = EventBus::new();
        assert_eq!(bus.subscriber_count(), 0);

        let subscriber = Arc::new(TestSubscriber {
            count: AtomicU32::new(0),
        });

        bus.subscribe_arc(subscriber.clone());
        assert_eq!(bus.subscriber_count(), 1);

        bus.emit(StoreEvent::StateChanged {
            store_id: StoreId::new::<TestStore>(),
            store_name: "TestStore",
            timestamp: 12345,
        });

        assert_eq!(subscriber.count.load(Ordering::SeqCst), 1);

        bus.clear();
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_event_subscriber_filter() {
        struct FilteredSubscriber {
            mutation_count: AtomicU32,
        }

        impl EventSubscriber for FilteredSubscriber {
            fn on_event(&self, _event: &StoreEvent) {
                self.mutation_count.fetch_add(1, Ordering::SeqCst);
            }

            fn filter(&self, event: &StoreEvent) -> bool {
                matches!(event, StoreEvent::MutationCompleted { .. })
            }
        }

        let bus = EventBus::new();
        let subscriber = Arc::new(FilteredSubscriber {
            mutation_count: AtomicU32::new(0),
        });

        bus.subscribe_arc(subscriber.clone());

        // This should be filtered out
        bus.emit(StoreEvent::StateChanged {
            store_id: StoreId::new::<TestStore>(),
            store_name: "TestStore",
            timestamp: 12345,
        });
        assert_eq!(subscriber.mutation_count.load(Ordering::SeqCst), 0);

        // This should pass the filter
        bus.emit(StoreEvent::MutationCompleted {
            store_id: StoreId::new::<TestStore>(),
            name: "test",
            duration_ms: 10,
            success: true,
        });
        assert_eq!(subscriber.mutation_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_middleware_store() {
        let store = TestStore::new();
        let mw_store = MiddlewareStore::new(store);

        // Should implement Store trait
        let _state = mw_store.state();
        let _id = mw_store.id();
        let _name = mw_store.name();
    }

    #[test]
    fn test_middleware_error_display() {
        assert_eq!(
            MiddlewareError::Rejected("test".to_string()).to_string(),
            "Middleware rejected: test"
        );
        assert_eq!(
            MiddlewareError::ValidationFailed("invalid".to_string()).to_string(),
            "Validation failed: invalid"
        );
        assert_eq!(
            MiddlewareError::Timeout(1000).to_string(),
            "Middleware timed out after 1000ms"
        );
        assert_eq!(
            MiddlewareError::Internal("oops".to_string()).to_string(),
            "Middleware error: oops"
        );
    }
}
