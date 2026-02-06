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
//! ```rust
//! # use leptos::prelude::{RwSignal, ReadSignal};
//! # use leptos_store::store::Store;
//! # use leptos_store::coordination::StoreCoordinator;
//! # #[derive(Clone, Debug, Default)]
//! # struct CartState;
//! # #[derive(Clone)]
//! # struct CartStore { state: RwSignal<CartState> }
//! # impl Store for CartStore {
//! #     type State = CartState;
//! #     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
//! # }
//! # #[derive(Clone, Debug, Default)]
//! # struct TotalsState;
//! # #[derive(Clone)]
//! # struct TotalsStore { state: RwSignal<TotalsState> }
//! # impl TotalsStore { fn recalculate(&self) {} }
//! # impl Store for TotalsStore {
//! #     type State = TotalsState;
//! #     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
//! # }
//! # #[derive(Clone, Debug, Default)]
//! # struct InventoryState;
//! # #[derive(Clone)]
//! # struct InventoryStore { state: RwSignal<InventoryState> }
//! # impl InventoryStore { fn check_stock(&self) {} }
//! # impl Store for InventoryStore {
//! #     type State = InventoryState;
//! #     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
//! # }
//! # let cart_store = CartStore { state: RwSignal::new(CartState) };
//! # let totals_store = TotalsStore { state: RwSignal::new(TotalsState) };
//! # let inventory_store = InventoryStore { state: RwSignal::new(InventoryState) };
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
//! ```rust
//! # use leptos::prelude::{RwSignal, ReadSignal};
//! # use leptos_store::store::Store;
//! # use leptos_store::middleware::{EventBus, MiddlewareStore};
//! # use leptos_store::coordination::StoreCoordinator;
//! # use std::sync::Arc;
//! # #[derive(Clone, Debug, Default)]
//! # struct MyState;
//! # #[derive(Clone)]
//! # struct MyStore { state: RwSignal<MyState> }
//! # impl Store for MyStore {
//! #     type State = MyState;
//! #     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
//! # }
//! # let my_store = MyStore { state: RwSignal::new(MyState) };
//! let bus = Arc::new(EventBus::new());
//! let mw_store = MiddlewareStore::with_event_bus(my_store, Arc::clone(&bus));
//! let coord = StoreCoordinator::with_event_bus(Arc::clone(&bus));
//! ```

use crate::middleware::{EventBus, EventSubscriber, StoreEvent};
use crate::store::{Store, StoreId};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use thiserror::Error;

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
            // CacheInvalidated is an output event — coordination rules
            // never re-trigger on it to avoid infinite loops.
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
    dependency_graph: Option<StoreDependencyGraph>,
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
            dependency_graph: None,
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
            dependency_graph: None,
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

    /// Register an invalidation rule: when the source store mutates
    /// successfully, emit a [`StoreEvent::CacheInvalidated`] event on the
    /// [`EventBus`].
    ///
    /// This is a convenience over manually writing `on_change` + `emit`
    /// logic. Subscribers (including other coordinators) can listen for
    /// `CacheInvalidated` events to refresh derived data, clear external
    /// caches, or trigger re-fetches.
    ///
    /// # Arguments
    ///
    /// * `source` — the store whose mutations trigger invalidation
    /// * `scope` — optional label to narrow which caches to invalidate
    ///   (e.g. `Some("pricing")`). Pass `None` for blanket invalidation.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use leptos::prelude::{RwSignal, ReadSignal};
    /// # use leptos_store::store::Store;
    /// # use leptos_store::coordination::StoreCoordinator;
    /// # #[derive(Clone, Debug, Default)]
    /// # struct CartState;
    /// # #[derive(Clone)]
    /// # struct CartStore { state: RwSignal<CartState> }
    /// # impl Store for CartStore {
    /// #     type State = CartState;
    /// #     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
    /// # }
    /// # let cart_store = CartStore { state: RwSignal::new(CartState) };
    /// let mut coord = StoreCoordinator::new();
    /// coord.invalidate_on_change(&cart_store, Some("pricing"));
    /// coord.activate();
    /// ```
    pub fn invalidate_on_change<Source: Store>(
        &mut self,
        source: &Source,
        scope: Option<&'static str>,
    ) -> &mut Self {
        let source_id = source.id();
        let bus = Arc::clone(&self.event_bus);
        self.rules.push(CoordinationRule {
            source_store_id: source_id,
            source_mutation: None,
            handler: Arc::new(move |_event| {
                bus.emit(StoreEvent::CacheInvalidated {
                    source_store_id: source_id,
                    scope,
                    timestamp: coordination_timestamp_ms(),
                });
            }),
        });
        self
    }

    /// Return the number of registered coordination rules.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Cross-platform timestamp in milliseconds (mirrors middleware::current_timestamp_ms).
fn coordination_timestamp_ms() -> u64 {
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

// ============================================================================
// CoordinationError
// ============================================================================

/// Errors that can occur during store coordination.
#[derive(Debug, Error, Clone)]
pub enum CoordinationError {
    /// A circular dependency was detected in the store dependency graph.
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    /// A referenced store was not found in the dependency graph.
    #[error("Store not found in graph: {0}")]
    StoreNotFound(String),
}

// ============================================================================
// StoreDependencyGraph
// ============================================================================

/// A directed acyclic graph of store dependencies for initialization ordering.
///
/// Use this to declare that one store depends on another, validate that no
/// circular dependencies exist, and compute the correct initialization
/// (topological) order.
///
/// # Example
///
/// ```rust
/// # use leptos::prelude::{RwSignal, ReadSignal};
/// # use leptos_store::store::Store;
/// # use leptos_store::coordination::StoreDependencyGraph;
/// # #[derive(Clone, Debug, Default)]
/// # struct AuthState;
/// # #[derive(Clone)]
/// # struct AuthStore { state: RwSignal<AuthState> }
/// # impl Store for AuthStore {
/// #     type State = AuthState;
/// #     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
/// # }
/// # #[derive(Clone, Debug, Default)]
/// # struct CartState;
/// # #[derive(Clone)]
/// # struct CartStore { state: RwSignal<CartState> }
/// # impl Store for CartStore {
/// #     type State = CartState;
/// #     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
/// # }
/// # #[derive(Clone, Debug, Default)]
/// # struct TotalsState;
/// # #[derive(Clone)]
/// # struct TotalsStore { state: RwSignal<TotalsState> }
/// # impl Store for TotalsStore {
/// #     type State = TotalsState;
/// #     fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
/// # }
/// # let auth = AuthStore { state: RwSignal::new(AuthState) };
/// # let cart = CartStore { state: RwSignal::new(CartState) };
/// # let totals = TotalsStore { state: RwSignal::new(TotalsState) };
/// let mut graph = StoreDependencyGraph::new();
/// graph.depends_on(&cart, &auth);    // cart depends on auth
/// graph.depends_on(&totals, &cart);  // totals depends on cart
///
/// // Validate — no cycles
/// assert!(graph.validate().is_ok());
///
/// // Topological order: auth → cart → totals
/// let order = graph.topological_order().unwrap();
/// assert_eq!(order.len(), 3);
/// ```
pub struct StoreDependencyGraph {
    /// Edges: key depends on each value (key → [dependencies]).
    edges: HashMap<StoreId, Vec<StoreId>>,
    /// Human-readable names for debugging.
    names: HashMap<StoreId, &'static str>,
}

impl Default for StoreDependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl StoreDependencyGraph {
    /// Create an empty dependency graph.
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            names: HashMap::new(),
        }
    }

    /// Declare that `source` depends on `dependency`.
    ///
    /// This means `dependency` should be initialized before `source`.
    pub fn depends_on<Source: Store, Dep: Store>(
        &mut self,
        source: &Source,
        dependency: &Dep,
    ) -> &mut Self {
        let source_id = source.id();
        let dep_id = dependency.id();

        self.names.entry(source_id).or_insert_with(|| source.name());
        self.names
            .entry(dep_id)
            .or_insert_with(|| dependency.name());

        // Ensure both nodes exist in the graph
        self.edges.entry(dep_id).or_default();
        self.edges.entry(source_id).or_default().push(dep_id);

        self
    }

    /// Validate the graph — detect circular dependencies via DFS.
    pub fn validate(&self) -> Result<(), CoordinationError> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for &node in self.edges.keys() {
            if !visited.contains(&node) {
                self.dfs_cycle_check(node, &mut visited, &mut rec_stack, &mut path)?;
            }
        }
        Ok(())
    }

    /// Compute topological initialization order using Kahn's algorithm.
    ///
    /// Returns store IDs in the order they should be initialized
    /// (dependencies first).
    pub fn topological_order(&self) -> Result<Vec<StoreId>, CoordinationError> {
        // Validate first
        self.validate()?;

        // Build reverse adjacency: for each edge (source → dep), we need
        // dep → source (dep must come before source).
        let mut reverse_adj: HashMap<StoreId, Vec<StoreId>> = HashMap::new();
        for &node in self.edges.keys() {
            reverse_adj.entry(node).or_default();
        }
        for (&source, deps) in &self.edges {
            for &dep in deps {
                reverse_adj.entry(dep).or_default().push(source);
            }
        }

        // In-degree[node] = number of dependencies node has (edges[node].len())
        let mut in_deg: HashMap<StoreId, usize> = self
            .edges
            .iter()
            .map(|(&id, deps)| (id, deps.len()))
            .collect();

        // Kahn's algorithm
        let mut queue: VecDeque<StoreId> = in_deg
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(id, _)| *id)
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop_front() {
            result.push(node);

            // For each store that depends on `node`, decrement in-degree
            if let Some(dependents) = reverse_adj.get(&node) {
                for &dependent in dependents {
                    if let Some(deg) = in_deg.get_mut(&dependent) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push_back(dependent);
                        }
                    }
                }
            }
        }

        if result.len() != self.edges.len() {
            // Should not happen since we validated first, but just in case
            return Err(CoordinationError::CircularDependency(
                "cycle detected during topological sort".to_string(),
            ));
        }

        Ok(result)
    }

    /// Get all stores that directly depend on the given store.
    pub fn dependents_of(&self, store_id: StoreId) -> Vec<StoreId> {
        self.edges
            .iter()
            .filter_map(|(&source, deps)| {
                if deps.contains(&store_id) {
                    Some(source)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the human-readable name for a store ID (if registered).
    pub fn store_name(&self, store_id: StoreId) -> Option<&'static str> {
        self.names.get(&store_id).copied()
    }

    /// Get the number of stores in the graph.
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Check if the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    // -- private helpers --

    fn dfs_cycle_check(
        &self,
        node: StoreId,
        visited: &mut HashSet<StoreId>,
        rec_stack: &mut HashSet<StoreId>,
        path: &mut Vec<StoreId>,
    ) -> Result<(), CoordinationError> {
        visited.insert(node);
        rec_stack.insert(node);
        path.push(node);

        if let Some(deps) = self.edges.get(&node) {
            for &dep in deps {
                if !visited.contains(&dep) {
                    self.dfs_cycle_check(dep, visited, rec_stack, path)?;
                } else if rec_stack.contains(&dep) {
                    // Found a cycle — build descriptive message
                    let cycle_names: Vec<&str> = path
                        .iter()
                        .skip_while(|&&id| id != dep)
                        .filter_map(|id| self.names.get(id).copied())
                        .collect();
                    let dep_name = self.names.get(&dep).copied().unwrap_or("unknown");
                    return Err(CoordinationError::CircularDependency(format!(
                        "{} → {}",
                        cycle_names.join(" → "),
                        dep_name
                    )));
                }
            }
        }

        path.pop();
        rec_stack.remove(&node);
        Ok(())
    }
}

impl std::fmt::Debug for StoreDependencyGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StoreDependencyGraph")
            .field("store_count", &self.edges.len())
            .field("names", &self.names.values().collect::<Vec<_>>())
            .finish()
    }
}

// ============================================================================
// Wire StoreDependencyGraph into StoreCoordinator
// ============================================================================

impl StoreCoordinator {
    /// Attach a dependency graph to this coordinator.
    ///
    /// The graph is informational — use it to validate dependencies and
    /// compute initialization ordering. The coordinator does not enforce
    /// ordering at runtime.
    pub fn with_dependency_graph(mut self, graph: StoreDependencyGraph) -> Self {
        self.dependency_graph = Some(graph);
        self
    }

    /// Get a reference to the dependency graph, if one was attached.
    pub fn dependency_graph(&self) -> Option<&StoreDependencyGraph> {
        self.dependency_graph.as_ref()
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

    // -- CacheInvalidated filtering tests ------------------------------------

    #[test]
    fn test_cache_invalidated_not_matched_by_coordination_subscriber() {
        with_owner(|| {
            let source = SourceStore::default();
            let source_id = source.id();

            // CacheInvalidated is an output event — coordination subscribers
            // never re-trigger on it (prevents infinite loops).
            let wildcard_sub = CoordinationSubscriber {
                source_store_id: source_id,
                source_mutation: None,
                handler: Arc::new(|_| {}),
            };

            assert!(!wildcard_sub.filter(&StoreEvent::CacheInvalidated {
                source_store_id: source_id,
                scope: None,
                timestamp: 0,
            }));

            let specific_sub = CoordinationSubscriber {
                source_store_id: source_id,
                source_mutation: Some("increment".to_string()),
                handler: Arc::new(|_| {}),
            };

            assert!(!specific_sub.filter(&StoreEvent::CacheInvalidated {
                source_store_id: source_id,
                scope: Some("pricing"),
                timestamp: 0,
            }));
        });
    }

    // -- invalidate_on_change tests ------------------------------------------

    #[test]
    fn test_invalidate_on_change_rule_count() {
        with_owner(|| {
            let source = SourceStore::default();
            let mut coord = StoreCoordinator::new();

            coord.invalidate_on_change(&source, Some("pricing"));
            assert_eq!(coord.rule_count(), 1);

            coord.invalidate_on_change(&source, None);
            assert_eq!(coord.rule_count(), 2);
        });
    }

    #[test]
    fn test_invalidate_on_change_emits_event() {
        use std::sync::atomic::{AtomicU32, Ordering};

        with_owner(|| {
            let source = SourceStore::default();
            let source_id = source.id();

            let mut coord = StoreCoordinator::new();
            coord.invalidate_on_change(&source, Some("test-scope"));
            coord.activate();

            // Count CacheInvalidated events
            let count = Arc::new(AtomicU32::new(0));
            let count_clone = count.clone();
            struct CountInvalidations {
                count: Arc<AtomicU32>,
            }
            impl EventSubscriber for CountInvalidations {
                fn on_event(&self, _event: &StoreEvent) {
                    self.count.fetch_add(1, Ordering::SeqCst);
                }
                fn filter(&self, event: &StoreEvent) -> bool {
                    matches!(event, StoreEvent::CacheInvalidated { .. })
                }
            }
            coord
                .event_bus()
                .subscribe(CountInvalidations { count: count_clone });

            // Emit a MutationCompleted — should trigger the invalidation rule
            // which emits CacheInvalidated
            coord.event_bus().emit(StoreEvent::MutationCompleted {
                store_id: source_id,
                name: "update",
                duration_ms: 1,
                success: true,
            });

            assert_eq!(count.load(Ordering::SeqCst), 1);
        });
    }

    // -- StoreDependencyGraph tests ------------------------------------------

    #[derive(Clone, Debug, Default)]
    struct StoreA {
        state: RwSignal<i32>,
    }
    impl Store for StoreA {
        type State = i32;
        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    #[derive(Clone, Debug, Default)]
    struct StoreB {
        state: RwSignal<i32>,
    }
    impl Store for StoreB {
        type State = i32;
        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    #[derive(Clone, Debug, Default)]
    struct StoreC {
        state: RwSignal<i32>,
    }
    impl Store for StoreC {
        type State = i32;
        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    #[test]
    fn test_dependency_graph_empty() {
        let graph = StoreDependencyGraph::new();
        assert!(graph.is_empty());
        assert_eq!(graph.len(), 0);
        assert!(graph.validate().is_ok());
        assert!(graph.topological_order().unwrap().is_empty());
    }

    #[test]
    fn test_dependency_graph_linear() {
        with_owner(|| {
            let a = StoreA::default();
            let b = StoreB::default();
            let c = StoreC::default();

            let mut graph = StoreDependencyGraph::new();
            graph.depends_on(&b, &a); // B depends on A
            graph.depends_on(&c, &b); // C depends on B

            assert_eq!(graph.len(), 3);
            assert!(graph.validate().is_ok());

            let order = graph.topological_order().unwrap();
            assert_eq!(order.len(), 3);

            // A must come before B, B must come before C
            let pos_a = order.iter().position(|id| *id == a.id()).unwrap();
            let pos_b = order.iter().position(|id| *id == b.id()).unwrap();
            let pos_c = order.iter().position(|id| *id == c.id()).unwrap();
            assert!(pos_a < pos_b);
            assert!(pos_b < pos_c);
        });
    }

    #[test]
    fn test_dependency_graph_cycle_detection() {
        with_owner(|| {
            let a = StoreA::default();
            let b = StoreB::default();

            let mut graph = StoreDependencyGraph::new();
            graph.depends_on(&b, &a); // B depends on A
            graph.depends_on(&a, &b); // A depends on B — cycle!

            let result = graph.validate();
            assert!(result.is_err());
            match result {
                Err(CoordinationError::CircularDependency(msg)) => {
                    assert!(msg.contains("→"), "Error should contain cycle path: {msg}");
                }
                _ => panic!("Expected CircularDependency error"),
            }
        });
    }

    #[test]
    fn test_dependency_graph_dependents_of() {
        with_owner(|| {
            let a = StoreA::default();
            let b = StoreB::default();
            let c = StoreC::default();

            let mut graph = StoreDependencyGraph::new();
            graph.depends_on(&b, &a); // B depends on A
            graph.depends_on(&c, &a); // C depends on A

            let dependents = graph.dependents_of(a.id());
            assert_eq!(dependents.len(), 2);
            assert!(dependents.contains(&b.id()));
            assert!(dependents.contains(&c.id()));

            // B has no dependents
            assert!(graph.dependents_of(b.id()).is_empty());
        });
    }

    #[test]
    fn test_dependency_graph_store_name() {
        with_owner(|| {
            let a = StoreA::default();
            let b = StoreB::default();

            let mut graph = StoreDependencyGraph::new();
            graph.depends_on(&b, &a);

            assert!(graph.store_name(a.id()).is_some());
            assert!(graph.store_name(b.id()).is_some());
        });
    }

    #[test]
    fn test_coordinator_with_dependency_graph() {
        with_owner(|| {
            let a = StoreA::default();
            let b = StoreB::default();

            let mut graph = StoreDependencyGraph::new();
            graph.depends_on(&b, &a);

            let coord = StoreCoordinator::new().with_dependency_graph(graph);
            assert!(coord.dependency_graph().is_some());
            assert_eq!(coord.dependency_graph().unwrap().len(), 2);
        });
    }
}
