# Store Composition — RootStore, Dependencies, Multi-Store Patterns

> **When to use:** Your app has multiple domain stores (auth, cart, UI), you need cross-store data access, or you're building a micro-frontend architecture.

## Prerequisites

No feature gates needed — composition is always available.

```toml
# Cargo.toml — no extra features required
[dependencies]
leptos-store = "0.5"
```

## Pattern

### RootStore — Aggregating Multiple Stores

Use `RootStore` when you have 3+ domain stores. It provides type-safe access to all stores from a single context entry.

```rust
use leptos_store::composition::{RootStore, CompositeStore};

// Build a root store from individual domain stores
let root = RootStore::builder()
    .with_store(AuthStore::new())
    .with_store(CartStore::new())
    .with_store(UiStore::new())
    .build();

// Type-safe access — returns Option<&S>
let auth: &AuthStore = root.get::<AuthStore>().unwrap();

// Or panic with helpful message if missing
let cart: &CartStore = root.expect::<CartStore>();

// Check presence
assert!(root.contains::<AuthStore>());
assert_eq!(root.len(), 3);
```

### Providing and Consuming RootStore

```rust
use leptos_store::composition::{provide_root_store, use_root_store, use_store_from_root};

// In your App component — provide once at the root
#[component]
fn App() -> impl IntoView {
    let root = RootStore::builder()
        .with_store(AuthStore::new())
        .with_store(CartStore::new())
        .build();
    provide_root_store(root);

    view! { <Dashboard /> }
}

// In child components — extract individual stores
#[component]
fn Dashboard() -> impl IntoView {
    // Option 1: Get root, then extract
    let root = use_root_store();
    let auth = root.expect::<AuthStore>();

    // Option 2: Direct extraction helper
    let cart = use_store_from_root::<CartStore>();

    view! {
        <p>"User: " {move || auth.display_name()}</p>
        <p>"Cart items: " {move || cart.item_count()}</p>
    }
}
```

### Helper Pattern — Typed Accessor Functions

Create convenience functions for frequently accessed stores:

```rust
pub fn use_auth() -> AuthStore {
    use_store_from_root::<AuthStore>()
}

pub fn use_cart() -> CartStore {
    use_store_from_root::<CartStore>()
}

// Usage in components
let auth = use_auth();
let cart = use_cart();
```

### StoreDependency — Cross-Store References

When one store needs to read another store's state:

```rust
use leptos_store::composition::StoreDependency;

struct CartStore {
    state: RwSignal<CartState>,
    auth: StoreDependency<AuthStore>,
}

impl CartStore {
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(CartState::default()),
            auth: StoreDependency::new(), // unresolved
        }
    }

    pub fn with_auth(auth: AuthStore) -> Self {
        Self {
            state: RwSignal::new(CartState::default()),
            auth: StoreDependency::resolved(auth),
        }
    }

    pub fn checkout(&self) {
        if let Some(auth) = self.auth.get() {
            if auth.is_authenticated() {
                // Process checkout
            }
        }
    }
}
```

Resolving dependencies after construction:

```rust
let auth = AuthStore::new();
let mut cart = CartStore::new();
cart.auth.resolve(auth.clone());  // now resolved
assert!(cart.auth.is_resolved());
```

### MultiStoreSelector — Cross-Store Derived Values

> **Cache invalidation:** `MultiStoreSelector` and `DerivedView` are the primary cross-store cache invalidation mechanism — they use `Memo<T>` which auto-recomputes when any source signal changes. No manual invalidation needed. For side-effect-based invalidation (API refetches, external cache clearing), see [10-cache-invalidation.md](10-cache-invalidation.md).

Combine state from 2 or 3 stores into a single reactive `Memo<T>`:

```rust
use leptos_store::composition::MultiStoreSelector;

// From two stores
let selector = MultiStoreSelector::from_two(
    &auth_store,
    &cart_store,
    |auth_state, cart_state| {
        (auth_state.user.is_some(), cart_state.items.len())
    },
);

let (is_auth, item_count) = selector.get();

// From three stores
let dashboard = MultiStoreSelector::from_three(
    &auth_store,
    &cart_store,
    &ui_store,
    |auth, cart, ui| DashboardData {
        user: auth.user.clone(),
        cart_items: cart.items.len(),
        theme: ui.theme.clone(),
    },
);
```

### StoreGroup — Domain Organization

Group related stores by domain:

```rust
use leptos_store::composition::StoreGroup;

let mut commerce = StoreGroup::new("commerce");
commerce.add(CartStore::new());
commerce.add(InventoryStore::new());

let cart = commerce.get::<CartStore>().unwrap();
assert_eq!(commerce.name(), "commerce");
assert_eq!(commerce.len(), 2);
```

### namespace! Macro — Typed Store Aggregation

The `namespace!` macro creates a typed namespace with auto-generated `provide_` and `use_` context helpers:

```rust
use leptos_store::namespace;

namespace! {
    pub AppStores {
        user: UserStore,
        products: ProductStore,
        cart: CartStore,
    }
}

// Creates AppStores struct + provide_app_stores() + use_app_stores()
let stores = AppStores::new(
    UserStore::new(),
    ProductStore::new(),
    CartStore::new(),
);
provide_app_stores(stores);

// In child components
let stores = use_app_stores();
let user = stores.user();      // returns &UserStore
let cart = stores.cart();       // returns &CartStore
assert_eq!(stores.store_count(), 3);
```

### DerivedView — Read-Only Cross-Store Computation

```rust
use leptos_store::composition::DerivedView;

let view = DerivedView::new(move || {
    let user = auth_store.display_name();
    let items = cart_store.item_count();
    format!("{user} has {items} items")
});

// Use in component
view! { <p>{move || view.get()}</p> }
```

## Key Rules

1. **Use `RootStore` when you have 3+ domain stores** — it provides type-safe, centralized store access.
2. **Provide `RootStore` at app root** — extract individual stores in child components via `use_store_from_root::<S>()`.
3. **`StoreDependency` prevents circular imports** — stores reference each other through dependencies, not direct ownership.
4. **`MultiStoreSelector` combines state reactively** — returns a `Memo<T>` that updates when any source store changes.
5. **Use `namespace!` for typed convenience** — auto-generates `provide_` and `use_` context helpers.

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| Providing individual stores instead of RootStore | Use `provide_root_store()` once, extract with `use_store_from_root::<S>()` |
| Circular store dependencies | Use `StoreDependency<S>` for lazy/optional references |
| Forgetting to resolve StoreDependency | Call `.resolve(store)` or use `StoreDependency::resolved(store)` at construction |
| Creating RootStore in child components | Create and provide at the app root only |

## Related Skills

- [01-creating-a-store.md](01-creating-a-store.md) — Individual store definition
- [06-selectors.md](06-selectors.md) — Single-store selectors and combine_selectors
- [08-middleware.md](08-middleware.md) — Cross-store coordination via EventBus
- [10-cache-invalidation.md](10-cache-invalidation.md) — Cross-store reactive updates, dependency graphs, cascading invalidation
