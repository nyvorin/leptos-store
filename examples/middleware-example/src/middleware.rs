// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Custom Middleware Implementations
//!
//! This module demonstrates how to create custom middleware for leptos-store.
//! We implement logging, timing, and validation middleware.

#[cfg(any(feature = "hydrate", feature = "ssr"))]
use leptos_store::middleware::{
    EventSubscriber, Middleware, MiddlewareContext, MiddlewareResult, MutationResult, StoreEvent,
};
#[cfg(any(feature = "hydrate", feature = "ssr"))]
use leptos_store::store::Store;

// ============================================================================
// Logging Middleware
// ============================================================================

/// A middleware that logs all mutations to the console.
#[cfg(any(feature = "hydrate", feature = "ssr"))]
pub struct ConsoleLoggingMiddleware {
    pub prefix: &'static str,
}

#[cfg(any(feature = "hydrate", feature = "ssr"))]
impl ConsoleLoggingMiddleware {
    pub fn new(prefix: &'static str) -> Self {
        Self { prefix }
    }
}

#[cfg(any(feature = "hydrate", feature = "ssr"))]
impl<S: Store> Middleware<S> for ConsoleLoggingMiddleware {
    fn before_mutate(&self, ctx: &MiddlewareContext<S>) -> MiddlewareResult {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(
            &format!(
                "[{}] Before mutation: {} on {}",
                self.prefix,
                ctx.mutation_name(),
                ctx.store_name()
            )
            .into(),
        );

        #[cfg(not(target_arch = "wasm32"))]
        println!(
            "[{}] Before mutation: {} on {}",
            self.prefix,
            ctx.mutation_name(),
            ctx.store_name()
        );

        MiddlewareResult::Continue
    }

    fn after_mutate(&self, ctx: &MiddlewareContext<S>, result: &MutationResult) {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(
            &format!(
                "[{}] After mutation: {} completed in {:?} (success: {})",
                self.prefix,
                ctx.mutation_name(),
                result.duration,
                result.success
            )
            .into(),
        );

        #[cfg(not(target_arch = "wasm32"))]
        println!(
            "[{}] After mutation: {} completed in {:?} (success: {})",
            self.prefix,
            ctx.mutation_name(),
            result.duration,
            result.success
        );
    }
}

// ============================================================================
// Metrics Subscriber (Event Bus Pattern)
// ============================================================================

/// An event subscriber that tracks metrics.
#[cfg(any(feature = "hydrate", feature = "ssr"))]
pub struct MetricsSubscriber {
    pub name: &'static str,
}

#[cfg(any(feature = "hydrate", feature = "ssr"))]
impl MetricsSubscriber {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[cfg(any(feature = "hydrate", feature = "ssr"))]
impl EventSubscriber for MetricsSubscriber {
    fn on_event(&self, event: &StoreEvent) {
        match event {
            StoreEvent::MutationCompleted {
                name, duration_ms, ..
            } => {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(
                    &format!(
                        "[{}] Metric: mutation '{}' took {}ms",
                        self.name, name, duration_ms
                    )
                    .into(),
                );

                #[cfg(not(target_arch = "wasm32"))]
                println!(
                    "[{}] Metric: mutation '{}' took {}ms",
                    self.name, name, duration_ms
                );
            }
            StoreEvent::ActionCompleted {
                action_name,
                duration_ms,
                success,
                ..
            } => {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(
                    &format!(
                        "[{}] Metric: action '{}' took {}ms (success: {})",
                        self.name, action_name, duration_ms, success
                    )
                    .into(),
                );

                #[cfg(not(target_arch = "wasm32"))]
                println!(
                    "[{}] Metric: action '{}' took {}ms (success: {})",
                    self.name, action_name, duration_ms, success
                );
            }
            StoreEvent::Error { message, .. } => {
                #[cfg(target_arch = "wasm32")]
                web_sys::console::error_1(
                    &format!("[{}] Error occurred: {}", self.name, message).into(),
                );

                #[cfg(not(target_arch = "wasm32"))]
                eprintln!("[{}] Error occurred: {}", self.name, message);
            }
            _ => {}
        }
    }

    fn name(&self) -> &'static str {
        self.name
    }
}
