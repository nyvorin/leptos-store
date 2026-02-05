# Counter Example

A simple counter demonstrating the leptos-store `store!` macro with the **Enterprise Mode** pattern.

## Features

- **Single Macro Definition**: Complete store with `store!` macro
- **State**: Auto-generated `CounterState` with defaults
- **Getters**: Public, read-only derived values
- **Mutators**: Private, internal state changes only
- **Actions**: Public, the only external API for writes
- **SSR Support**: Server-side rendering with Actix Web

## Running

```bash
# From the project root
make run NAME=counter-example

# Opens at http://127.0.0.1:3001
```

## The `store!` Macro (Enterprise Mode)

The `store!` macro generates a complete store with enforced access control:

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
            is_positive(this) -> bool {
                this.read(|s| s.count > 0)
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

        // PUBLIC - the external API for writes
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

### Key Points

- Use `this` (or any identifier) instead of `self` due to Rust 2024 macro hygiene
- **Getters**: Public, use `this.read(|s| ...)` for read-only access
- **Mutators**: **Private**, use `this.mutate(|s| ...)` for state changes
- **Actions**: **Public**, call mutators internally
- The macro auto-implements the `Store` trait

## Generated API

The macro generates:

```rust
// State struct with Default
pub struct CounterState { pub count: i32 }

// Store struct
pub struct CounterStore { ... }

impl CounterStore {
    pub fn new() -> Self
    pub fn with_state(state: CounterState) -> Self

    // Getters - PUBLIC
    pub fn doubled(&self) -> i32
    pub fn is_positive(&self) -> bool

    // Mutators - PRIVATE (cannot be called from outside)
    fn set_count(&self, value: i32)
    fn add_to_count(&self, delta: i32)

    // Actions - PUBLIC
    pub fn increment(&self)
    pub fn decrement(&self)
    pub fn reset(&self)
}

impl Store for CounterStore { ... }
```

## Usage in Components

```rust
use leptos_store::prelude::*;
use crate::counter_store::CounterStore;

#[component]
fn Counter() -> impl IntoView {
    let store = use_store::<CounterStore>();

    view! {
        <div>
            <p>"Count: " {move || store.state().get().count}</p>
            <p>"Doubled: " {move || store.doubled()}</p>
            // Components can only call PUBLIC actions
            <button on:click=move |_| store.increment()>"+"</button>
            <button on:click=move |_| store.decrement()>"-"</button>
            <button on:click=move |_| store.reset()>"Reset"</button>
        </div>
    }
}
```

## Why Enterprise Mode?

External code **cannot**:
- Call `store.set_count(5)` directly (mutator is private)
- Bypass business logic

External code **can only**:
- Read state via `store.state().get()`
- Read derived values via public getters
- Write via public actions that enforce business rules
