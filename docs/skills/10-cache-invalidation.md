# Cache Invalidation — Cross-Store Reactive Updates

> **When to use:** Multiple stores where mutations in one affect derived data in another, cross-domain coordination, cache freshness requirements.

## Prerequisites

Cache invalidation uses two layers:

1. **Reactive invalidation** (built-in, no feature flag) — `MultiStoreSelector` and `DerivedView` auto-recompute via `Memo<T>`
2. **Explicit invalidation** (requires `middleware` feature) — `StoreCoordinator`, `CacheInvalidated` events, `StoreDependencyGraph`

```toml
# For reactive invalidation only (no extra features needed)
[dependencies]
leptos-store = "0.5"

# For explicit invalidation + coordination
[dependencies]
leptos-store = { version = "0.5", features = ["middleware"] }
```

## Reactive Invalidation (Built-In)

**This is the primary cache invalidation mechanism.** Leptos's reactive system handles it automatically.

`MultiStoreSelector` and `DerivedView` are both `Memo<T>` under the hood. When any source signal changes, the memo recomputes. No manual invalidation needed.

### MultiStoreSelector — Cross-Store Derived Values

```rust
use leptos_store::composition::MultiStoreSelector;

// When cart_store state changes, this selector auto-recomputes
let dashboard = MultiStoreSelector::from_two(
    &cart_store,
    &pricing_store,
    |cart_state, pricing_state| {
        let subtotal: f64 = cart_state.items.iter()
            .map(|item| pricing_state.price_for(&item.sku))
            .sum();
        DashboardData { item_count: cart_state.items.len(), subtotal }
    },
);

// Use in a component — auto-updates when either store changes
view! { <p>"Total: $" {move || dashboard.get().subtotal}</p> }
```

**Source:** `src/composition.rs` — `MultiStoreSelector` wraps `Memo::new()`.

### DerivedView — Read-Only Cross-Store Computation

```rust
use leptos_store::composition::DerivedView;

let summary = DerivedView::new(move || {
    let user = auth_store.display_name();
    let count = cart_store.item_count();
    let total = pricing_store.cart_total();
    format!("{user}: {count} items, ${total:.2}")
});

view! { <p>{move || summary.get()}</p> }
```

**Source:** `src/composition.rs` — `DerivedView` wraps `Memo::new()`.

### When Reactive Invalidation Is Enough

Use reactive invalidation alone when:
- Derived data is purely computed from store signals
- No external side effects needed (no API calls, no cache clearing)
- The computation is cheap enough to run on every change

## Explicit Invalidation via StoreCoordinator

For side effects beyond reactive memos — e.g., refetch from server, clear an external cache, or notify other services — use the `StoreCoordinator` and `CacheInvalidated` events.

### on_change — Any Mutation Triggers Handler

```rust
use leptos_store::coordination::StoreCoordinator;
use std::sync::Arc;

let bus = Arc::new(EventBus::new());
let mut coord = StoreCoordinator::with_event_bus(Arc::clone(&bus));

// When cart mutates, recalculate totals
coord.on_change(&cart_store, &totals_store, |totals, _event| {
    totals.recalculate();
});

coord.activate();  // REQUIRED — registers rules on the EventBus
```

### on_mutation — Specific Mutation Triggers Handler

```rust
// Only "add_item" triggers an inventory check
coord.on_mutation(&cart_store, "add_item", &inventory_store, |inventory| {
    inventory.check_stock();
});
```

### invalidate_on_change — Emit CacheInvalidated Events

A convenience method that emits `StoreEvent::CacheInvalidated` when a source store mutates. Other subscribers (including other coordinators) can listen for these events.

```rust
let mut coord = StoreCoordinator::with_event_bus(Arc::clone(&bus));

// When cart mutates, emit a CacheInvalidated event with scope "pricing"
coord.invalidate_on_change(&cart_store, Some("pricing"));

coord.activate();
```

### Listening for CacheInvalidated Events

```rust
use leptos_store::middleware::{EventSubscriber, StoreEvent};

struct CacheRefresher;

impl EventSubscriber for CacheRefresher {
    fn on_event(&self, event: &StoreEvent) {
        if let StoreEvent::CacheInvalidated { scope, .. } = event {
            match *scope {
                Some("pricing") => refresh_pricing_cache(),
                Some("inventory") => refresh_inventory_cache(),
                None => refresh_all_caches(),
                _ => {}
            }
        }
    }

    fn filter(&self, event: &StoreEvent) -> bool {
        matches!(event, StoreEvent::CacheInvalidated { .. })
    }
}

bus.subscribe(CacheRefresher);
```

## Dependency Graph

Declare store dependencies, validate for cycles, and compute initialization ordering.

```rust
use leptos_store::coordination::StoreDependencyGraph;

let mut graph = StoreDependencyGraph::new();
graph.depends_on(&cart_store, &auth_store);      // cart depends on auth
graph.depends_on(&totals_store, &cart_store);     // totals depends on cart
graph.depends_on(&totals_store, &pricing_store);  // totals also depends on pricing

// Validate — detect circular dependencies
graph.validate().expect("No circular dependencies");

// Get initialization order (dependencies first)
let order = graph.topological_order().unwrap();
// order: [auth, pricing, cart, totals] (or similar valid topological sort)

// Query what depends on a store
let cart_dependents = graph.dependents_of(cart_store.id());
// cart_dependents: [totals_store.id()]
```

### Integration with StoreCoordinator

```rust
let coord = StoreCoordinator::with_event_bus(Arc::clone(&bus))
    .with_dependency_graph(graph);

// Query the graph later
if let Some(graph) = coord.dependency_graph() {
    let order = graph.topological_order().unwrap();
    // Initialize stores in this order
}
```

### CoordinationError

```rust
use leptos_store::coordination::CoordinationError;

match graph.validate() {
    Ok(()) => { /* graph is acyclic */ }
    Err(CoordinationError::CircularDependency(msg)) => {
        // msg contains the cycle path, e.g. "CartStore → AuthStore → CartStore"
        panic!("Circular dependency: {msg}");
    }
    Err(CoordinationError::StoreNotFound(msg)) => {
        panic!("Store not registered: {msg}");
    }
}
```

## Cascading Updates Pattern

When Store A mutation triggers Store B update, which triggers Store C:

```rust
let mut coord = StoreCoordinator::with_event_bus(Arc::clone(&bus));

// Cart → Totals
coord.on_change(&cart_store, &totals_store, |totals, _| {
    totals.recalculate();
});

// Totals → Dashboard
coord.on_change(&totals_store, &dashboard_store, |dashboard, _| {
    dashboard.refresh_summary();
});

coord.activate();
```

**Important:** Updates are reactive but not transactional. There is no guarantee that Store C sees a consistent snapshot of both Store A and Store B during the cascade. Each handler runs independently. This is a known limitation — for transactional consistency, compute derived values in a single `MultiStoreSelector` instead.

## Hydration Consistency

Each store hydrates independently in Leptos. For dependent stores, the initialization order matters: a store should not read from a dependency that hasn't been hydrated yet.

### RootStoreBuilder Ordering Hints

```rust
use leptos_store::composition::RootStore;

let root = RootStore::builder()
    .with_store(AuthStore::new())
    .with_store_after::<_, AuthStore>(CartStore::new())      // cart after auth
    .with_store_after::<_, CartStore>(TotalsStore::new())    // totals after cart
    .build();
```

The builder records the constraint. Query it with `builder.initialization_order()`. Actual sequencing during hydration is determined by component render order in Leptos — structure your component tree so that parent stores render before child stores.

### Recommended Pattern

1. Hydrate stores in dependency order (use `StoreDependencyGraph::topological_order()` as a guide)
2. Activate the `StoreCoordinator` after all stores are hydrated
3. Only then do cross-store coordination rules fire

## Cache TTL Pattern

For time-based cache invalidation, use Leptos reactive primitives — no library code needed:

```rust
use leptos::prelude::*;

// Invalidate pricing cache every 5 minutes
Effect::new(move || {
    let store = pricing_store.clone();
    set_interval(
        move || {
            store.refresh_from_api();
        },
        std::time::Duration::from_secs(300),
    );
});
```

This is a user-land pattern using existing reactive primitives (`Effect::new` + `set_interval`).

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Not sharing EventBus between MiddlewareStore and StoreCoordinator | Use `Arc::new(EventBus::new())` and pass `Arc::clone(&bus)` to both |
| Forgetting `coord.activate()` | Rules aren't registered until `activate()` is called — nothing happens without it |
| Circular coordination rules (A → B → A) | Use `StoreDependencyGraph::validate()` to detect cycles before activation |
| Assuming synchronous cross-store updates | Coordination is reactive, not transactional — use `MultiStoreSelector` for consistent cross-store reads |
| Not hydrating in dependency order | Structure component tree so parent stores render first, or use `with_store_after()` ordering hints |
| Using coordination for purely derived data | Use `MultiStoreSelector` or `DerivedView` instead — they're simpler and auto-memoized |

## Key Rules

1. **`MultiStoreSelector` and `DerivedView` ARE the primary cache invalidation mechanism** — they use `Memo<T>` which auto-recomputes when source signals change.
2. **Use `StoreCoordinator` only for side effects** — API refetches, external cache clearing, cross-system notifications.
3. **Always call `coord.activate()`** — rules are inert until activated on the EventBus.
4. **Share one `Arc<EventBus>`** across all participating middleware stores and coordinators.
5. **Use `StoreDependencyGraph::validate()`** to catch circular dependencies at startup, not at runtime.
6. **Hydrate in dependency order** — use `topological_order()` as a guide for structuring your component tree.
7. **Coordination is reactive, not transactional** — for consistent cross-store reads, use a `MultiStoreSelector`.

## Related Skills

- [07-store-composition.md](07-store-composition.md) — MultiStoreSelector, DerivedView, RootStore
- [08-middleware.md](08-middleware.md) — EventBus, MiddlewareStore, StoreCoordinator basics
- [06-selectors.md](06-selectors.md) — Single-store Memo-based selectors
- [05-ssr-hydration.md](05-ssr-hydration.md) — Hydration ordering considerations
- [troubleshooting.md](troubleshooting.md) — Common coordination issues
