// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Cross-store reactive coordination via the middleware EventBus.
//!
//! [`StoreCoordinator`] lets stores declare reactive dependencies on one
//! another: when a source store mutates, registered handlers fire on
//! dependent (target) stores. All routing is performed through the
//! existing [`EventBus`] infrastructure so that coordination rules
//! compose naturally with other subscribers and middleware.
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use leptos_store::prelude::*;
//!
//! let mut coord = StoreCoordinator::new();
//!
//! // Any successful mutation on `cart_store` triggers a recalculation
//! // on `totals_store`:
//! coord.on_change(&cart_store, &totals_store, |totals, _event| {
//!     totals.recalculate();
//! });
//!
//! // Only the "add_item" mutation on `cart_store` triggers an
//! // inventory check on `inventory_store`:
//! coord.on_mutation(&cart_store, "add_item", &inventory_store, |inventory| {
//!     inventory.check_stock();
//! });
//!
//! // Register all rules on the EventBus
//! coord.activate();
//! ```
//!
//! # Sharing an EventBus
//!
//! To share one [`EventBus`] between a [`MiddlewareStore`](crate::middleware::MiddlewareStore) and the
//! coordinator, use [`StoreCoordinator::with_event_bus`]:
//!
//! ```rust,ignore
//! let bus = Arc::new(EventBus::new());
//! let mw_store = MiddlewareStore::with_event_bus(my_store, Arc::clone(&bus));
//! let coord = StoreCoordinator::with_event_bus(Arc::clone(&bus));
//! ```

use crate::middleware::{EventBus, EventSubscriber, StoreEvent};
use crate::store::{Store, StoreId};
use std::sync::Arc;

// ============================================================================
// CoordinationRule (private)
// ============================================================================

/// A single coordination rule linking a source store event to a handler.
struct CoordinationRule {
    /// The source store whose events trigger the handler.
    source_store_id: StoreId,
    /// If `Some`, only the named mutation triggers the handler.
    /// If `None`, any successful mutation or state change triggers it.
    source_mutation: Option<String>,
    /// The handler invoked when the rule matches.
    handler: Arc<dyn Fn(&StoreEvent) + Send + Sync>,
}

// ============================================================================
// CoordinationSubscriber (private)
// ============================================================================

/// An [`EventSubscriber`] created from a single [`CoordinationRule`].
struct CoordinationSubscriber {
    source_store_id: StoreId,
    source_mutation: Option<String>,
    handler: Arc<dyn Fn(&StoreEvent) + Send + Sync>,
}

impl EventSubscriber for CoordinationSubscriber {
    fn on_event(&self, event: &StoreEvent) {
        (self.handler)(event);
    }

    fn name(&self) -> &'static str {
        "StoreCoordinator"
    }

    fn filter(&self, event: &StoreEvent) -> bool {
        match event {
            StoreEvent::MutationCompleted {
                store_id,
                name,
                success,
                ..
            } => {
                if *store_id != self.source_store_id || !success {
                    return false;
                }
                match &self.source_mutation {
                    Some(expected) => *name == expected.as_str(),
                    None => true,
                }
            }
            StoreEvent::StateChanged { store_id, .. } => {
                *store_id == self.source_store_id && self.source_mutation.is_none()
            }
            _ => false,
        }
    }
}

// ============================================================================
// StoreCoordinator
// ============================================================================

/// Declarative cross-store reactive coordination.
///
/// `StoreCoordinator` collects coordination rules that link source store
/// events to handler functions on target stores. Once [`activate`](Self::activate)
/// is called, the rules are registered as [`EventSubscriber`]s on the
/// underlying [`EventBus`].
pub struct StoreCoordinator {
    rules: Vec<CoordinationRule>,
    event_bus: Arc<EventBus>,
}

impl Default for StoreCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl StoreCoordinator {
    /// Create a new coordinator with its own [`EventBus`].
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            event_bus: Arc::new(EventBus::new()),
        }
    }

    /// Create a new coordinator that shares an existing [`EventBus`].
    ///
    /// Use this when you want the coordinator to listen on the same bus
    /// that a [`MiddlewareStore`](crate::middleware::MiddlewareStore) emits to.
    pub fn with_event_bus(event_bus: Arc<EventBus>) -> Self {
        Self {
            rules: Vec::new(),
            event_bus,
        }
    }

    /// Register a handler that fires on **any** successful mutation or
    /// state change of the source store.
    ///
    /// The handler receives a reference to the target store and the
    /// triggering [`StoreEvent`].
    pub fn on_change<Source: Store, Target: Store>(
        &mut self,
        source: &Source,
        target: &Target,
        handler: impl Fn(&Target, &StoreEvent) + Send + Sync + 'static,
    ) -> &mut Self {
        let target = target.clone();
        self.rules.push(CoordinationRule {
            source_store_id: source.id(),
            source_mutation: None,
            handler: Arc::new(move |event| {
                handler(&target, event);
            }),
        });
        self
    }

    /// Register a handler that fires only when the named mutation
    /// completes successfully on the source store.
    ///
    /// The handler receives a reference to the target store.
    pub fn on_mutation<Source: Store, Target: Store>(
        &mut self,
        source: &Source,
        mutation_name: &str,
        target: &Target,
        handler: impl Fn(&Target) + Send + Sync + 'static,
    ) -> &mut Self {
        let target = target.clone();
        self.rules.push(CoordinationRule {
            source_store_id: source.id(),
            source_mutation: Some(mutation_name.to_string()),
            handler: Arc::new(move |_event| {
                handler(&target);
            }),
        });
        self
    }

    /// Register all accumulated rules as [`EventSubscriber`]s on the
    /// [`EventBus`].
    ///
    /// After activation the coordinator retains its rules but they are
    /// now live on the bus. Adding more rules after activation requires
    /// calling `activate` again.
    pub fn activate(&self) {
        for rule in &self.rules {
            let subscriber = CoordinationSubscriber {
                source_store_id: rule.source_store_id,
                source_mutation: rule.source_mutation.clone(),
                handler: Arc::clone(&rule.handler),
            };
            self.event_bus.subscribe(subscriber);
        }
    }

    /// Get a reference to the underlying [`EventBus`].
    ///
    /// Useful for sharing the bus with a
    /// [`MiddlewareStore`](crate::middleware::MiddlewareStore).
    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    /// Return the number of registered coordination rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::*;

    // -- Test stores --------------------------------------------------------

    #[derive(Clone, Debug, Default)]
    struct SourceStore {
        state: RwSignal<i32>,
    }

    impl Store for SourceStore {
        type State = i32;
        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    #[derive(Clone, Debug, Default)]
    struct TargetStore {
        state: RwSignal<i32>,
    }

    impl Store for TargetStore {
        type State = i32;
        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    // -- Helpers ------------------------------------------------------------

    /// Run a closure inside the Leptos reactive Owner.
    fn with_owner<F: FnOnce()>(f: F) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let owner = Owner::new();
            owner.with(f);
        });
    }

    // -- Tests --------------------------------------------------------------

    #[test]
    fn test_coordinator_rule_count() {
        with_owner(|| {
            let source = SourceStore::default();
            let target = TargetStore::default();

            let mut coord = StoreCoordinator::new();
            assert_eq!(coord.rule_count(), 0);

            coord.on_change(&source, &target, |_t, _e| {});
            assert_eq!(coord.rule_count(), 1);

            coord.on_mutation(&source, "increment", &target, |_t| {});
            assert_eq!(coord.rule_count(), 2);
        });
    }

    #[test]
    fn test_coordinator_event_filtering() {
        with_owner(|| {
            let source = SourceStore::default();
            let source_id = source.id();

            // Subscriber that only matches "increment" on the source store
            let subscriber = CoordinationSubscriber {
                source_store_id: source_id,
                source_mutation: Some("increment".to_string()),
                handler: Arc::new(|_| {}),
            };

            // Correct store + correct mutation + success -> true
            assert!(subscriber.filter(&StoreEvent::MutationCompleted {
                store_id: source_id,
                name: "increment",
                duration_ms: 0,
                success: true,
            }));

            // Wrong mutation name -> false
            assert!(!subscriber.filter(&StoreEvent::MutationCompleted {
                store_id: source_id,
                name: "decrement",
                duration_ms: 0,
                success: true,
            }));

            // Correct mutation but failed -> false
            assert!(!subscriber.filter(&StoreEvent::MutationCompleted {
                store_id: source_id,
                name: "increment",
                duration_ms: 0,
                success: false,
            }));

            // Wrong store entirely -> false
            let other_id = StoreId::with_instance::<TargetStore>(999);
            assert!(!subscriber.filter(&StoreEvent::MutationCompleted {
                store_id: other_id,
                name: "increment",
                duration_ms: 0,
                success: true,
            }));

            // StateChanged should not match when source_mutation is Some
            assert!(!subscriber.filter(&StoreEvent::StateChanged {
                store_id: source_id,
                store_name: "SourceStore",
                timestamp: 0,
            }));
        });
    }

    #[test]
    fn test_coordinator_wildcard_filtering() {
        with_owner(|| {
            let source = SourceStore::default();
            let source_id = source.id();

            // Wildcard subscriber (source_mutation: None) matches any mutation
            let subscriber = CoordinationSubscriber {
                source_store_id: source_id,
                source_mutation: None,
                handler: Arc::new(|_| {}),
            };

            // Any successful mutation on the source -> true
            assert!(subscriber.filter(&StoreEvent::MutationCompleted {
                store_id: source_id,
                name: "increment",
                duration_ms: 0,
                success: true,
            }));

            assert!(subscriber.filter(&StoreEvent::MutationCompleted {
                store_id: source_id,
                name: "decrement",
                duration_ms: 0,
                success: true,
            }));

            // Failed mutation -> false
            assert!(!subscriber.filter(&StoreEvent::MutationCompleted {
                store_id: source_id,
                name: "increment",
                duration_ms: 0,
                success: false,
            }));

            // StateChanged on the correct store -> true
            assert!(subscriber.filter(&StoreEvent::StateChanged {
                store_id: source_id,
                store_name: "SourceStore",
                timestamp: 0,
            }));

            // StateChanged on wrong store -> false
            let other_id = StoreId::with_instance::<TargetStore>(999);
            assert!(!subscriber.filter(&StoreEvent::StateChanged {
                store_id: other_id,
                store_name: "TargetStore",
                timestamp: 0,
            }));

            // Unrelated event types -> false
            assert!(!subscriber.filter(&StoreEvent::MutationStarted {
                store_id: source_id,
                name: "increment",
                timestamp: 0,
            }));
        });
    }

    #[test]
    fn test_coordinator_activate_registers_subscribers() {
        with_owner(|| {
            let source = SourceStore::default();
            let target = TargetStore::default();

            let mut coord = StoreCoordinator::new();
            coord.on_change(&source, &target, |_t, _e| {});
            coord.on_mutation(&source, "increment", &target, |_t| {});

            assert_eq!(coord.event_bus().subscriber_count(), 0);

            coord.activate();

            assert_eq!(coord.event_bus().subscriber_count(), 2);
        });
    }
}
