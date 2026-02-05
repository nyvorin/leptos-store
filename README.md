# leptos-store

> Enterprise-grade, type-enforced state management for Leptos

[![Crates.io](https://img.shields.io/crates/v/leptos-store.svg)](https://crates.io/crates/leptos-store)
[![Documentation](https://img.shields.io/docsrs/leptos-store/latest)](https://docs.rs/leptos-store)
[![License](https://img.shields.io/crates/l/leptos-store.svg)](LICENSE)

## Overview

`leptos-store` provides a structured, SSR-safe state management architecture for [Leptos](https://leptos.dev), inspired by **Vuex** and **Pinia**, translated into idiomatic Rust.

Leptos provides excellent low-level primitives (signals, context, resources), but intentionally does not define a canonical, scalable state architecture. At scale, this absence can create challenges for large teams, enterprise governance, long-lived applications, SSR safety, and state auditing unless additional architectural patterns are introduced.

**leptos-store exists to solve structure, not reactivity.**

## Features

- 🏗️ **Global, namespaced stores** - Clear domain boundaries
- 🔒 **Predictable mutation flow** - Only mutators can write state
- 🌐 **First-class SSR support** - Works seamlessly with server-side rendering
- 💧 **SSR Hydration** - Automatic state serialization and hydration between server and client
- ⚡ **Async-safe actions** - Built-in support for async operations
- 🔧 **Compile-time enforcement** - Catch errors at compile time, not runtime
- 📦 **Zero magic** - No hidden executors or runtime reflection

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
leptos-store = "0.2"
leptos = "0.8"
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `ssr` | ✅ Yes | Server-side rendering support |
| `hydrate` | ❌ No | SSR hydration with automatic state serialization and transfer |
| `csr` | ❌ No | Client-side rendering only (no SSR) |
| `middleware` | ❌ No | Middleware system, audit trail, store coordination |
| `devtools` | ❌ No | DevTools integration with time-travel debugging |
| `persist-web` | ❌ No | Browser-based state persistence (localStorage/sessionStorage) |

#### Basic Usage (SSR without Hydration)

The `ssr` feature is enabled by default. For basic SSR without state hydration:

```toml
[dependencies]
leptos-store = "0.2"
```

#### Full SSR with Hydration (Recommended for Production)

For full SSR applications where state needs to transfer from server to client, enable the `hydrate` feature. This requires different features for server and client builds:

```toml
[dependencies]
leptos-store = { version = "0.2", default-features = false }

[features]
ssr = ["leptos-store/ssr", "leptos/ssr"]
hydrate = ["leptos-store/hydrate", "leptos/hydrate"]
```

The `hydrate` feature enables:
- `HydratableStore` trait for state serialization
- `provide_hydrated_store()` - Server-side state embedding
- `use_hydrated_store()` - Client-side state recovery
- Automatic JSON serialization via `serde`

#### Client-Side Only

For SPAs without server rendering:

```toml
[dependencies]
leptos-store = { version = "0.2", default-features = false, features = ["csr"] }
```

### Deployment Models

| Model | Feature Flag | Description | Best For |
|-------|-------------|-------------|----------|
| **SSR** | `ssr` (default) | Store created per-request on server, HTML rendered with initial state | SEO, fast initial paint |
| **Hydrate** | `hydrate` | Server renders AND serializes state; client picks up seamlessly | Full-stack apps with dynamic data |
| **CSR** | `csr` | Store created once in the browser; no server, no hydration | SPAs, static sites, prototypes |

**CSR Quick Start:**

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;

// In CSR mode, just create and provide — no server needed
let store = MyStore::new();
provide_store(store);
// Or use the CSR helper:
// mount_csr_store(store);
```

> **Feature flag conflicts:** `csr` and `ssr` should not both be enabled. `hydrate` implies `ssr` behavior on the server side.

## Quick Start

### Define Your Store (Enterprise Mode)

The library enforces the **Enterprise Mode** pattern:
- **Getters**: Public, read-only derived values
- **Mutators**: Private, internal state modification only
- **Actions**: Public, the only external API for writes

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;

// Define your state
#[derive(Clone, Debug, Default)]
pub struct CounterState {
    pub count: i32,
}

// Define your store
#[derive(Clone)]
pub struct CounterStore {
    state: RwSignal<CounterState>,  // Private field
}

impl CounterStore {
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(CounterState::default()),
        }
    }

    // Getters - PUBLIC, derived read-only values
    pub fn doubled(&self) -> i32 {
        self.state.with(|s| s.count * 2)
    }

    // Mutators - PRIVATE, internal state changes
    fn set_count(&self, value: i32) {
        self.state.update(|s| s.count = value);
    }

    fn add_to_count(&self, delta: i32) {
        self.state.update(|s| s.count += delta);
    }

    // Actions - PUBLIC, the external API for writes
    pub fn increment(&self) {
        self.add_to_count(1);
    }

    pub fn decrement(&self) {
        self.add_to_count(-1);
    }

    pub fn reset(&self) {
        self.set_count(0);
    }
}

impl Store for CounterStore {
    type State = CounterState;

    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}
```

### Use in Components

```rust
#[component]
pub fn App() -> impl IntoView {
    // Provide store to component tree
    let store = CounterStore::new();
    provide_store(store);

    view! {
        <Counter />
    }
}

#[component]
fn Counter() -> impl IntoView {
    let store = use_store::<CounterStore>();

    view! {
        <div>
            <p>"Count: " {move || store.state().get().count}</p>
            <p>"Doubled: " {move || store.doubled()}</p>
            <button on:click=move |_| store.increment()>"+"</button>
            <button on:click=move |_| store.decrement()>"-"</button>
        </div>
    }
}
```

### Using the `store!` Macro

For less boilerplate, use the declarative macro:

```rust
use leptos_store::store;

store! {
    pub CounterStore {
        state CounterState {
            count: i32 = 0,
        }

        getters {
            doubled(this) -> i32 {
                this.read(|s| s.count * 2)
            }
        }

        // PRIVATE - internal state changes only
        mutators {
            set_count(this, value: i32) {
                this.mutate(|s| s.count = value);
            }
            add_to_count(this, delta: i32) {
                this.mutate(|s| s.count += delta);
            }
        }

        // PUBLIC - external API for writes
        actions {
            increment(this) {
                this.add_to_count(1);
            }
            decrement(this) {
                this.add_to_count(-1);
            }
            reset(this) {
                this.set_count(0);
            }
        }
    }
}
```

> **Note**: Use `this` (or any identifier) instead of `self` in getter/mutator/action bodies due to Rust 2024 macro hygiene rules. The macro provides `this.read()` for getters and `this.mutate()` for mutators. Mutators are **private** - external code must use public **actions**.

## Available Macros

| Macro | Purpose | Feature |
|-------|---------|---------|
| `define_state!` | Define state structs with default values | - |
| `define_hydratable_state!` | Define state with serde derives for hydration | `hydrate` |
| `define_action!` | Define synchronous action structs | - |
| `define_async_action!` | Define async action structs with result types | - |
| `impl_store!` | Implement Store trait for an existing type | - |
| `impl_hydratable_store!` | Implement HydratableStore trait | `hydrate` |
| `store!` | Complete store definition in one macro | - |
| `selector!` | Batch-create multiple selectors from a store | - |
| `namespace!` | Define typed namespace combining multiple stores | - |
| `derive_state_diff!` | Generate `StateDiff` impl for field-level diffing | `middleware` |

### `define_state!` - State with Defaults

```rust
use leptos_store::define_state;

define_state! {
    #[derive(Clone, Debug, PartialEq)]
    pub struct UserState {
        name: String,                    // Uses String::default()
        email: Option<String>,           // Uses None
        age: u32 = 0,                    // Explicit default
        active: bool = true,             // Explicit default
    }
}

let user = UserState::default();
assert_eq!(user.name, "");
assert!(user.active);
```

### `define_action!` - Synchronous Actions

```rust
use leptos_store::define_action;

define_action! {
    /// Updates user profile information
    #[derive(Debug, Clone)]
    pub UpdateProfileAction {
        user_id: String,
        name: Option<String>,
        email: Option<String>,
    }
}

let action = UpdateProfileAction::new(
    "user_123".to_string(),
    Some("John Doe".to_string()),
    None,
);
```

### `define_async_action!` - Async Actions with Error Types

```rust
use leptos_store::define_async_action;

// Define your error type
#[derive(Debug, Clone)]
enum ApiError {
    Network(String),
    NotFound,
    Unauthorized,
}

// Define the async action
define_async_action! {
    /// Fetches user data from the API
    #[derive(Debug, Clone)]
    pub FetchUserAction {
        user_id: String,
        include_profile: bool,
    } -> Result<UserData, ApiError>
}

let action = FetchUserAction::new("user_123".to_string(), true);

// Helper methods for documentation
assert!(FetchUserAction::result_type_description().contains("Result"));
assert_eq!(FetchUserAction::output_type_name(), "UserData");
assert_eq!(FetchUserAction::error_type_name(), "ApiError");
```

### `impl_store!` - Quick Store Trait Implementation

```rust
use leptos::prelude::*;
use leptos_store::{impl_store, store::Store};

#[derive(Clone, Default)]
struct CartState {
    items: Vec<String>,
    total: f64,
}

#[derive(Clone)]
struct CartStore {
    state: RwSignal<CartState>,
}

// One-liner to implement Store trait
impl_store!(CartStore, CartState, state);
```

## Conceptual Model

Each store is a **domain module** composed of:

| Layer | Description | Can Write State | Async | Side Effects |
|-------|-------------|-----------------|-------|--------------|
| **State** | Read-only externally | N/A | ❌ | ❌ |
| **Getters** | Derived, read-only | ❌ | ❌ | ❌ |
| **Mutators** | Pure, synchronous writes | ✅ | ❌ | ❌ |
| **Actions** | Sync orchestration | ❌ | ❌ | ✅ |
| **Async Actions** | Async orchestration | ❌ | ✅ | ✅ |

**Only mutators may write state.** This is the core principle that ensures predictability.

## Advanced Usage

### Async Actions

```rust
use leptos_store::prelude::*;

pub struct LoginAction {
    pub email: String,
    pub password: String,
}

impl AsyncAction<AuthStore> for LoginAction {
    type Output = AuthToken;
    type Error = AuthError;

    async fn execute(&self, store: &AuthStore) -> ActionResult<Self::Output, Self::Error> {
        // Perform async operation
        let token = auth_api::login(&self.email, &self.password).await?;
        
        // Dispatch mutation
        store.set_authenticated(true, token.clone());
        
        Ok(token)
    }
}
```

**Lifecycle:** Every async action follows: `Idle → Pending → Success | Error`

- **Idle**: Initial state, no operation in progress
- **Pending**: Operation dispatched, awaiting result; use for loading indicators
- **Success**: Operation completed; result available via `ActionHandle`
- **Error**: Operation failed; error available for display or retry

**Loading state in components:**

```rust
let handle = store.dispatch_async(FetchUsersAction { page: 1 });

view! {
    {move || match handle.state().get() {
        ActionState::Idle => view! { <p>"Ready"</p> }.into_any(),
        ActionState::Pending => view! { <p>"Loading..."</p> }.into_any(),
        ActionState::Success => view! { <p>"Loaded!"</p> }.into_any(),
        ActionState::Error => view! { <p>"Failed. Try again."</p> }.into_any(),
    }}
}
```

**Cancellation:** Call `handle.cancel()` to abort an in-flight async action. The state transitions to `Idle` and any pending future is dropped.

### Selectors — Fine-Grained Reactivity

Selectors create memoized views into specific slices of store state. Unlike reading the full state signal, selectors only trigger re-renders when their particular slice changes.

```rust
use leptos_store::prelude::*;

let store = use_store::<DashboardStore>();

// Extract a single slice — only re-computes when user_name changes
let user_name = create_selector(&store, |s| s.user_name.clone());

// Combine two selectors into a derived value
let item_count = create_selector(&store, |s| s.cart_items.len());
let discount = create_selector(&store, |s| s.cart_discount);
let total = combine_selectors(item_count, discount, |count, disc| {
    let subtotal = *count as f64 * 9.99;
    subtotal * (1.0 - disc / 100.0)
});

// Transform a selector's output
let badge = map_selector(item_count, |n| format!("{n} items"));

// Only propagate when a condition is met
let active_items = filter_selector(item_count, |n| *n > 0);
// Returns Memo<Option<usize>> — None when cart is empty
```

**Batch declaration with `selector!` macro:**

```rust
selector! {
    store: &my_store,
    user_name: |s: &AppState| -> String { s.user.name.clone() },
    is_admin: |s: &AppState| -> bool { s.user.role == Role::Admin },
    cart_total: |s: &AppState| -> f64 { s.cart.items.iter().map(|i| i.price).sum() },
}
// Generates: let user_name: Memo<String>, let is_admin: Memo<bool>, etc.
```

### Scoped Stores

For multiple instances of the same store type:

```rust
// Provide scoped stores with unique IDs
provide_scoped_store::<CounterStore, 1>(counter1);
provide_scoped_store::<CounterStore, 2>(counter2);

// Access scoped stores
let counter1 = use_scoped_store::<CounterStore, 1>();
let counter2 = use_scoped_store::<CounterStore, 2>();
```

### Multiple Namespaced Stores

For large applications with many domain stores, use the `namespace!` macro to create a typed container with generated context helpers:

```rust
use leptos_store::namespace;

namespace! {
    pub AppStores {
        user: UserStore,
        products: ProductStore,
        cart: CartStore,
        orders: OrderStore,
        ui: UiStore,
    }
}

// Generated API:
// AppStores::new(user, products, cart, orders, ui)
// app_stores.user() -> &UserStore
// provide_app_stores(stores) — wraps provide_context
// use_app_stores() -> AppStores — wraps use_context
```

**Domain boundary guidelines:**
- **One domain = one store** — `UserStore` owns auth + profile + preferences
- **Stores communicate via `StoreCoordinator`**, not direct references
- **Shared state** (theme, locale) goes in a `UiStore`; domain-specific state stays in its own store
- **Module organization:** `stores/user/mod.rs`, `stores/cart/mod.rs`, etc.

### Audit Trail (requires `middleware` feature)

Track every state mutation with field-level diffs, user context, and state replay:

```rust
use leptos_store::prelude::*;

// Create audit trail
let audit = AuditTrail::<MyState>::new()
    .with_max_entries(1000)
    .with_user_context(|| AuditUserContext {
        user_id: Some("admin".into()),
        session_id: None,
        ip_address: None,
        metadata: Default::default(),
    });

// Record a mutation with automatic diff
let before = old_state.clone();
let after = new_state.clone();
audit.record_with_diff("update_profile", None, before, after);

// Query audit entries
let recent = audit.entries_since(timestamp);
let profile_changes = audit.entries_for_mutation("update_profile");

// Replay: get state at any point in history
let historical_state = audit.state_at(entry_id);
```

**Field-level diffs with `derive_state_diff!`:**

```rust
use leptos_store::derive_state_diff;

derive_state_diff! {
    pub struct UserState {
        pub name: String,
        pub email: String,
        pub role: String,
    }
}
// Generates StateDiff impl — diff() returns Vec<FieldChange>
// Each FieldChange has: field_path, old_value, new_value, change_type
```

### Cross-Store Coordination (requires `middleware` feature)

Coordinate reactive dependencies between stores with `StoreCoordinator`:

```rust
use leptos_store::prelude::*;

let mut coordinator = StoreCoordinator::new();

// When auth store changes, clear the inventory cache
coordinator.on_change(&auth_store, &inventory_store, |target, event| {
    target.clear_cache();
});

// When a specific mutation fires, react in another store
coordinator.on_mutation(&inventory_store, "update_stock", &notification_store, |target| {
    target.add_notification("Stock updated".into());
});

// Start listening after all rules are registered
coordinator.activate();
```

The coordinator is stateless (just rules) — on hydration, re-register the same rules client-side.

### Store Registry

For debugging and hot-reloading:

```rust
let mut registry = StoreRegistry::new();
registry.register(my_store)?;

// Later...
let store = registry.get::<MyStore>();
```

### SSR Hydration

For full SSR applications, implement `HydratableStore` to enable automatic state transfer from server to client:

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;
use serde::{Serialize, Deserialize};

// State must derive Serialize and Deserialize
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TokenState {
    pub tokens: Vec<Token>,
    pub loading: bool,
}

#[derive(Clone)]
pub struct TokenStore {
    state: RwSignal<TokenState>,
}

impl Store for TokenStore {
    type State = TokenState;
    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}

// Implement HydratableStore for SSR hydration
#[cfg(feature = "hydrate")]
impl HydratableStore for TokenStore {
    fn serialize_state(&self) -> Result<String, StoreHydrationError> {
        serde_json::to_string(&self.state.get())
            .map_err(|e| StoreHydrationError::Serialization(e.to_string()))
    }

    fn from_hydrated_state(data: &str) -> Result<Self, StoreHydrationError> {
        let state: TokenState = serde_json::from_str(data)
            .map_err(|e| StoreHydrationError::Deserialization(e.to_string()))?;
        Ok(Self { state: RwSignal::new(state) })
    }

    fn store_key() -> &'static str {
        "token_store"
    }
}
```

Or use the `impl_hydratable_store!` macro for less boilerplate:

```rust
use leptos_store::impl_hydratable_store;

impl_hydratable_store!(TokenStore, TokenState, state, "token_store");
```

**Server-side (SSR):**
```rust
// Provide store and render hydration script
let store = TokenStore::new_with_data(tokens);
let hydration_script = provide_hydrated_store(store);

view! {
    {hydration_script}
    <App/>
}
```

**Client-side (Hydration):**
```rust
// Automatically hydrate from server-rendered state
let store = use_hydrated_store::<TokenStore>();
```

## Design Philosophy

### Convention over Primitives

Instead of giving you raw signals and hoping for the best, leptos-store provides a structured architecture that scales.

### Compile-time Enforcement

The type system prevents invalid state transitions. If it compiles, it follows the rules.

### SSR-First Design

Every feature is designed with server-side rendering in mind. No hydration mismatches.

## Examples

See the `examples/` directory for complete examples:

- `counter-example` - **Simple counter** using the `store!` macro with increment/decrement
- `auth-store-example` - User authentication flow with login/logout
- `token-explorer-example` - **Full SSR with hydration** - Real-time Solana token explorer using Jupiter API
- `csr-example` - **CSR-only** todo list demonstrating client-side store initialization
- `selectors-example` - **Fine-grained reactivity** with selectors, combinators, and the `selector!` macro
- `middleware-example` - Middleware pipeline with logging, validation, and event bus
- `composition-example` - Multi-store composition with `RootStore` builder
- `persistence-example` - State persistence across page reloads
- `feature-flags-example` - Feature flag management store
- `devtools-example` - DevTools integration with time-travel debugging

### Running Examples

```bash
# List all available examples
make examples-list

# Run a specific example (SSR mode with cargo-leptos)
make run NAME=auth-store-example
make run NAME=token-explorer-example

# Build an example
make build-example NAME=token-explorer-example
```

### Token Explorer Example

The `token-explorer-example` demonstrates full SSR hydration:

- 🌐 Server-side data fetching from Jupiter API
- 💧 Automatic state hydration to client
- 🔄 Client-side polling for real-time updates
- 🔍 Reactive filtering and sorting
- 🎨 Beautiful token card UI

```bash
# Run the token explorer
make run NAME=token-explorer-example

# Opens at http://127.0.0.1:3005
```

## Contributing

We welcome contributions! See [`AUTHORING.md`](./AUTHORING.md) for:

- Development setup and workflow
- Project structure and architecture
- Testing and code quality guidelines
- Publishing releases

```bash
# Quick start for contributors
git clone https://github.com/your-org/leptos-store.git
cd leptos-store
make check   # Verify setup
make test    # Run tests
make help    # See all commands
```

## License and Attribution

leptos-store is licensed under the [Apache License, Version 2.0](./LICENSE).

You are free to use, modify, and distribute this software, including for
commercial purposes, provided that you retain the license text and the
NOTICE file as required by the Apache 2.0 License.

This software is provided "AS IS", without warranty of any kind. The author
is not liable for any damages arising from its use.
