# Middleware — Audit Trails, Event Bus, Cross-Store Coordination

> **When to use:** You need to intercept/observe store mutations, track state history for debugging or compliance, or coordinate reactions between stores.

## Prerequisites

```toml
# Cargo.toml
[dependencies]
leptos-store = { version = "0.5", features = ["middleware"] }
```

The `middleware` feature adds the `js-sys` dependency (for cross-platform timing in WASM).

## Pattern

This skill covers three subsystems that work together:

1. **Middleware** — Intercept mutations before/after execution
2. **Audit Trail** — Record state mutation history with diffs
3. **Store Coordination** — Cross-store reactive reactions via EventBus

---

## 1. Middleware

### MiddlewareStore — Wrapping a Store

```rust
use leptos_store::middleware::{MiddlewareStore, LoggingMiddleware};

let store = MiddlewareStore::new(MyStore::new());

// Add built-in middleware
store.add_middleware(LoggingMiddleware::new());

// Execute mutations through the middleware pipeline
store.mutate("increment", || {
    store.inner().increment();
}).unwrap();
```

### The Middleware Trait

```rust
use leptos_store::middleware::{Middleware, MiddlewareContext, MiddlewareResult, MutationResult};
use leptos_store::store::Store;

struct BalanceValidator;

impl<S: Store> Middleware<S> for BalanceValidator {
    fn before_mutate(&self, ctx: &MiddlewareContext<S>) -> MiddlewareResult {
        // Validate before mutation executes
        MiddlewareResult::Continue
    }

    fn after_mutate(&self, ctx: &MiddlewareContext<S>, result: &MutationResult) {
        if !result.success {
            eprintln!("Mutation {} failed: {:?}", ctx.mutation_name(), result.error);
        }
    }

    fn name(&self) -> &'static str {
        "BalanceValidator"
    }

    fn priority(&self) -> i32 {
        100  // higher = runs first in before hooks
    }
}
```

**MiddlewareResult options:**
- `Continue` — proceed to next middleware/operation
- `Skip` — skip remaining middleware, execute operation
- `Abort(MiddlewareError)` — cancel the entire operation
- `Transform` — continue (for advanced use cases)

### Built-in Middleware

**LoggingMiddleware** — logs mutations and actions to console:
```rust
use leptos_store::middleware::{LoggingMiddleware, LogLevel};

let logger = LoggingMiddleware::new()
    .with_level(LogLevel::Debug)
    .log_state_before()
    .log_state_after()
    .with_prefix("[MyApp]");

store.add_middleware(logger);
```

**TimingMiddleware** — warns on slow operations:
```rust
use leptos_store::middleware::TimingMiddleware;

let timer = TimingMiddleware::new()
    .with_warn_threshold(100)   // warn above 100ms
    .with_error_threshold(1000); // error above 1000ms

store.add_middleware(timer);
```

**ValidationMiddleware** — validates state before mutations:
```rust
use leptos_store::middleware::ValidationMiddleware;

let validator = ValidationMiddleware::new()
    .add_validator(|state: &AccountState| {
        if state.balance < 0 {
            Err("Balance cannot be negative".to_string())
        } else {
            Ok(())
        }
    });

store.add_middleware(validator);
```

### Named Mutations

Always use named mutations with `mw_store.mutate("name", ...)` for tracking:

```rust
// Named mutation — tracked by middleware, audit trail, coordination
mw_store.mutate("add_item", || {
    mw_store.inner().add_item(item);
}).unwrap();

// Named action dispatch
use std::any::TypeId;
mw_store.dispatch("checkout", TypeId::of::<CheckoutAction>(), || {
    mw_store.inner().checkout()
}).unwrap();
```

### Middleware Priority

Lower priority number = runs later in `before_*` hooks, earlier in `after_*` hooks.

| Middleware | Priority | Purpose |
|-----------|----------|---------|
| ValidationMiddleware | 100 | Runs early to catch invalid states |
| TimingMiddleware | -50 | Runs after most, before logging |
| LoggingMiddleware | -100 | Runs last (captures full timing) |

---

## 2. Audit Trail

### Basic Recording

```rust
use leptos_store::audit::AuditTrail;

let trail: AuditTrail<AppState> = AuditTrail::new()
    .with_max_entries(500);  // default is 1000

let before = get_current_state();
perform_mutation();
let after = get_current_state();

trail.record("increment", &before, &after);
```

### Recording with Field-Level Diffs

Implement the `StateDiff` trait for detailed change tracking:

```rust
use leptos_store::audit::{StateDiff, FieldChange, ChangeType};

impl StateDiff for AppState {
    fn diff(&self, other: &Self) -> Vec<FieldChange> {
        let mut changes = Vec::new();
        if self.count != other.count {
            changes.push(FieldChange {
                field_path: "count".into(),
                old_value: format!("{}", self.count),
                new_value: format!("{}", other.count),
                change_type: ChangeType::Modified,
            });
        }
        changes
    }
}

// Now use record_with_diff for field-level tracking
trail.record_with_diff("increment", &before, &after);
```

Or use the `derive_state_diff!` macro:

```rust
use leptos_store::derive_state_diff;

derive_state_diff! {
    pub struct UserState {
        pub name: String,
        pub email: String,
        pub login_count: u32,
    }
}
// StateDiff is now auto-implemented
```

### Querying the Audit Trail

```rust
// All entries (oldest to newest)
let entries = trail.entries();

// Filter by mutation name
let increments = trail.entries_for_mutation("increment");

// Filter by timestamp
let recent = trail.entries_since(timestamp_ms);

// Look up by entry ID
let entry = trail.entry_by_id(42);

// State replay — get the state at a specific point
let snapshot: Option<AppState> = trail.state_at(entry_id);
```

### User Context

Attach user identity to audit entries:

```rust
use leptos_store::audit::AuditUserContext;

let trail = AuditTrail::new()
    .with_user_context(|| {
        AuditUserContext::new()
            .with_user_id("user-42")
            .with_session_id("sess-abc")
            .with_ip_address("192.168.1.1")
            .with_metadata("role", "admin")
    });

// All recorded entries now automatically include user context
trail.record("action", &before, &after);
let ctx = trail.entries()[0].user_context.as_ref().unwrap();
assert_eq!(ctx.user_id, Some("user-42".to_string()));
```

---

## 3. Store Coordination

### EventBus — Shared Event Infrastructure

```rust
use leptos_store::middleware::{EventBus, MiddlewareStore};
use std::sync::Arc;

// Create a shared event bus
let bus = Arc::new(EventBus::new());

// Middleware stores share the same bus
let cart_mw = MiddlewareStore::with_event_bus(CartStore::new(), Arc::clone(&bus));
let totals_mw = MiddlewareStore::with_event_bus(TotalsStore::new(), Arc::clone(&bus));
```

### StoreCoordinator — Declarative Cross-Store Reactions

```rust
use leptos_store::coordination::StoreCoordinator;

let mut coord = StoreCoordinator::with_event_bus(Arc::clone(&bus));

// Any mutation on cart → recalculate totals
coord.on_change(&cart_store, &totals_store, |totals, _event| {
    totals.recalculate();
});

// Only "add_item" mutation on cart → check inventory
coord.on_mutation(&cart_store, "add_item", &inventory_store, |inventory| {
    inventory.check_stock();
});

// Register rules on the EventBus
coord.activate();
```

### Event Subscribers

Subscribe to store events for observation:

```rust
use leptos_store::middleware::{EventSubscriber, StoreEvent};

struct MetricsSubscriber;

impl EventSubscriber for MetricsSubscriber {
    fn on_event(&self, event: &StoreEvent) {
        match event {
            StoreEvent::MutationCompleted { name, duration_ms, success, .. } => {
                record_metric(name, *duration_ms, *success);
            }
            _ => {}
        }
    }

    fn filter(&self, event: &StoreEvent) -> bool {
        matches!(event, StoreEvent::MutationCompleted { .. })
    }
}

bus.subscribe(MetricsSubscriber);
```

### StoreEvent Variants

```rust
pub enum StoreEvent {
    StateChanged { store_id, store_name, timestamp },
    MutationStarted { store_id, name, timestamp },
    MutationCompleted { store_id, name, duration_ms, success },
    ActionDispatched { store_id, action_type, action_name, timestamp },
    ActionCompleted { store_id, action_name, duration_ms, success },
    Error { store_id, message, source },
}
```

## Complete Example

```rust
use leptos_store::middleware::*;
use leptos_store::audit::*;
use leptos_store::coordination::*;
use std::sync::Arc;

// 1. Create shared event bus
let bus = Arc::new(EventBus::new());

// 2. Wrap stores with middleware
let cart = MiddlewareStore::with_event_bus(CartStore::new(), Arc::clone(&bus));
cart.add_middleware(LoggingMiddleware::new());
cart.add_middleware(ValidationMiddleware::new()
    .add_validator(|s: &CartState| {
        if s.items.len() > 100 { Err("Cart too large".into()) } else { Ok(()) }
    })
);

// 3. Set up audit trail
let trail: AuditTrail<CartState> = AuditTrail::new()
    .with_max_entries(500)
    .with_user_context(|| AuditUserContext::new().with_user_id("current-user"));

// 4. Set up cross-store coordination
let mut coord = StoreCoordinator::with_event_bus(Arc::clone(&bus));
coord.on_change(&cart, &totals_store, |totals, _| totals.recalculate());
coord.activate();

// 5. Execute a tracked mutation
let before = cart.inner().get_state();
cart.mutate("add_item", || {
    cart.inner().add_item(item);
}).unwrap();
let after = cart.inner().get_state();
trail.record_with_diff("add_item", &before, &after);
```

## Key Rules

1. **`middleware` feature adds `js-sys` dependency** — needed for cross-platform timing.
2. **Always use named mutations** — `mw_store.mutate("name", ...)` enables tracking by audit trail and coordination.
3. **Middleware priority: higher number = runs first** in `before_*` hooks, reverse in `after_*` hooks.
4. **`StateDiff` must be manually implemented** (or use `derive_state_diff!` macro) for field-level audit diffs.
5. **Coordination requires a shared `Arc<EventBus>`** across all participating middleware stores and coordinators.
6. **Call `coordinator.activate()`** to register rules — without this, no events are routed.

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Mutations not tracked by middleware | Use `mw_store.mutate("name", \|\| ...)` not direct store method calls |
| Coordinator not receiving events | Ensure shared `Arc<EventBus>` between MiddlewareStore and StoreCoordinator |
| Forgot to call `coord.activate()` | Rules aren't registered until `activate()` is called |
| Audit trail growing unbounded | Set `.with_max_entries(n)` — default is 1000 |
| `derive_state_diff!` not available | Requires `middleware` feature gate |

## Related Skills

- [01-creating-a-store.md](01-creating-a-store.md) — Store definition (middleware wraps stores)
- [07-store-composition.md](07-store-composition.md) — RootStore for multi-store architecture
- [troubleshooting.md](troubleshooting.md) — Common middleware issues
