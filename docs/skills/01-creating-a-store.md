# Creating a Store

## When to Use
You need reactive state in a Leptos component — counter, form, user session, shopping cart, any domain state.

## Prerequisites
```toml
# Cargo.toml
[dependencies]
leptos-store = "0.5"
leptos = "0.8"
```

No feature gates needed for basic stores. The `store!` macro is always available.

## Pattern: The `store!` Macro (Enterprise Mode)

The `store!` macro generates a complete store with enforced access patterns:
- **Getters** — public, read-only derived values using `this.read(|s| ...)`
- **Mutators** — **private**, internal state changes using `this.mutate(|s| ...)`
- **Actions** — **public**, the only external write API (calls private mutators)

This is the **Enterprise Mode** invariant: external code cannot bypass business logic by calling mutators directly.

### Complete Example

```rust
use leptos_store::store;

store! {
    pub CounterStore {
        // 1. State: fields with optional defaults
        state CounterState {
            count: i32 = 0,
        }

        // 2. Getters: PUBLIC read-only derived values
        getters {
            doubled(this) -> i32 {
                this.read(|s| s.count * 2)
            }

            is_positive(this) -> bool {
                this.read(|s| s.count > 0)
            }
        }

        // 3. Mutators: PRIVATE internal state changes
        mutators {
            set_count(this, value: i32) {
                this.mutate(|s| s.count = value);
            }

            add_to_count(this, delta: i32) {
                this.mutate(|s| s.count += delta);
            }
        }

        // 4. Actions: PUBLIC external write API
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

            set(this, value: i32) {
                this.set_count(value);
            }
        }
    }
}
```

### What the Macro Generates

The `store!` macro expands to:

```rust
// State struct with Clone, Debug, Default
#[derive(Clone, Debug)]
pub struct CounterState {
    pub count: i32,
}

impl Default for CounterState {
    fn default() -> Self { Self { count: 0 } }
}

// Store struct wrapping RwSignal
#[derive(Clone)]
pub struct CounterStore {
    state: RwSignal<CounterState>,
}

impl CounterStore {
    pub fn new() -> Self { /* ... */ }
    pub fn with_state(state: CounterState) -> Self { /* ... */ }

    // Getters — PUBLIC
    pub fn doubled(&self) -> i32 { /* this.read(|s| s.count * 2) */ }
    pub fn is_positive(&self) -> bool { /* ... */ }

    // Mutators — PRIVATE (fn, not pub fn)
    fn set_count(&self, value: i32) { /* this.mutate(|s| s.count = value) */ }
    fn add_to_count(&self, delta: i32) { /* ... */ }

    // Actions — PUBLIC
    pub fn increment(&self) { self.add_to_count(1); }
    pub fn decrement(&self) { self.add_to_count(-1); }
    pub fn reset(&self) { self.set_count(0); }
    pub fn set(&self, value: i32) { self.set_count(value); }

    // Internal helpers (generated)
    fn read<R>(&self, f: impl FnOnce(&CounterState) -> R) -> R { /* ... */ }
    fn mutate<R>(&self, f: impl FnOnce(&mut CounterState) -> R) -> R { /* ... */ }
}

// Store trait impl
impl Store for CounterStore {
    type State = CounterState;
    fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
}
```

### Constructors

```rust
// Default state
let store = CounterStore::new();

// Custom initial state
let store = CounterStore::with_state(CounterState { count: 42 });
```

## Pattern: Providing and Consuming Stores

Use `provide_store()` in a parent component and `use_store()` in children.

```rust
use leptos::prelude::*;
use leptos_store::prelude::{Store, provide_store, use_store};

// In a parent component:
#[component]
pub fn App() -> impl IntoView {
    let store = CounterStore::new();
    provide_store(store);

    view! { <Counter /> }
}

// In a child component:
#[component]
fn Counter() -> impl IntoView {
    let store = use_store::<CounterStore>();

    // Clone store for each closure that needs it
    let store_inc = store.clone();
    let store_dec = store.clone();
    let store_display = store.clone();

    view! {
        <span>{move || store_display.state().get().count}</span>
        <button on:click=move |_| store_inc.increment()>"+"</button>
        <button on:click=move |_| store_dec.decrement()>"-"</button>
    }
}
```

## Key Rules

1. **Use `this` not `self`** in store! macro bodies. Rust 2024 edition macro hygiene prevents `self` from working inside macro-generated code. The first parameter name in getters/mutators/actions is bound to `&self`.

2. **Getters are public, mutators are private, actions are public.** This is enforced by the macro — mutators generate `fn` (private), getters and actions generate `pub fn`.

3. **Only actions can call mutators.** This is the core invariant. External code must go through actions to modify state.

4. **Use explicit imports:**
   ```rust
   use leptos_store::prelude::{Store, provide_store, use_store};
   ```
   Never use `use leptos::prelude::*;` — it can introduce ambiguous imports (e.g., `create_selector`).

5. **Clone the store for closures.** Leptos closures in `view!` need `move`, so clone the store for each closure that captures it.

6. **Access state reactively** with `move || store.getter()` or `move || store.state().get().field` inside view macros.

## Related Skills
- [06-selectors.md](06-selectors.md) — For fine-grained derived state
- [07-store-composition.md](07-store-composition.md) — For multiple domain stores
- [architecture-guide.md](architecture-guide.md) — For deciding store structure
