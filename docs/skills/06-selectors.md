# Selectors — Fine-Grained Memo-Based Reactivity

> **When to use:** You need derived/computed values from store state, want to prevent unnecessary re-renders, or need to combine state from multiple stores.

## Prerequisites

No feature gates needed — selectors are always available.

```toml
# Cargo.toml — no extra features required
[dependencies]
leptos-store = "0.5"
```

**Critical import rule:**
```rust
// CORRECT — explicit import from leptos_store
use leptos_store::prelude::create_selector;

// WRONG — glob import causes ambiguity with deprecated Leptos 0.8 create_selector
// use leptos::prelude::*;
```

## Pattern

### Create a Selector

`create_selector` extracts a slice from a store's state and returns a `Memo<T>` that only recomputes when the selected value changes.

```rust
use leptos::prelude::*;
use leptos_store::prelude::{Store, create_selector};

// Given a store with state:
// struct AppState { count: i32, name: String, items: Vec<String> }

let count = create_selector(&store, |state| state.count);
let name = create_selector(&store, |state| state.name.clone());
let item_count = create_selector(&store, |state| state.items.len());
```

**Signature:**
```rust
pub fn create_selector<S, T>(
    store: &S,
    selector_fn: impl Fn(&S::State) -> T + Send + Sync + 'static,
) -> Memo<T>
where
    S: Store,
    T: Clone + PartialEq + Send + Sync + 'static,
```

### Transform with map_selector

Transform a selector's output. Equivalent to `Iterator::map`.

```rust
use leptos_store::prelude::map_selector;

let count = create_selector(&store, |s| s.count);
let doubled = map_selector(count, |c| c * 2);
let is_positive = map_selector(count, |c| *c > 0);
let display = map_selector(count, |c| format!("Count: {c}"));
```

### Filter with filter_selector

Conditionally emit values. Returns `Memo<Option<T>>`.

```rust
use leptos_store::prelude::filter_selector;

let count = create_selector(&store, |s| s.count);
let positive_only = filter_selector(count, |c| *c > 0);

// positive_only.get() returns:
//   Some(5) when count is 5
//   None when count is -1
```

### Combine Multiple Selectors

Combine two selectors (possibly from different stores) into one.

```rust
use leptos_store::prelude::combine_selectors;

let count = create_selector(&store, |s| s.count);
let name = create_selector(&store, |s| s.name.clone());

let summary = combine_selectors(count, name, |c, n| format!("{n}: {c}"));
// summary.get() → "Alice: 42"
```

Cross-store combination:
```rust
let count_a = create_selector(&store_a, |s| s.count);
let count_b = create_selector(&store_b, |s| s.count);
let total = combine_selectors(count_a, count_b, |a, b| a + b);
```

### Chaining Selectors

Selectors compose naturally:

```rust
let count = create_selector(&store, |s| s.count);
let doubled = map_selector(count, |c| c * 2);
let filtered = filter_selector(doubled, |v| *v > 10);
// filtered.get() → Some(16) when count is 8, None when count is 3
```

### selector! Macro

Create multiple selectors from one store in a single declaration:

```rust
use leptos_store::selector;

selector! {
    store: &my_store,
    user_name: |s: &AppState| -> String { s.user.name.clone() },
    is_admin: |s: &AppState| -> bool { s.user.role == Role::Admin },
    item_count: |s: &AppState| -> usize { s.cart.items.len() },
}

// user_name, is_admin, item_count are all Memo<T> values
```

### Use in Components

```rust
#[component]
fn CounterDisplay() -> impl IntoView {
    let store = use_store::<CounterStore>();
    let count = create_selector(&store, |s| s.count);
    let doubled = map_selector(count, |c| c * 2);

    view! {
        <p>"Count: " {move || count.get()}</p>
        <p>"Doubled: " {move || doubled.get()}</p>
    }
}
```

## Key Rules

1. **Import explicitly** — `use leptos_store::prelude::create_selector;` not `use leptos::prelude::*;`. Leptos 0.8 has a deprecated `create_selector` that causes ambiguity with glob imports.
2. **Selectors return `Memo<T>`** — they only recompute when dependencies change. Components using selectors re-render only when the selected slice changes.
3. **`T` must be `Clone + PartialEq + Send + Sync + 'static`** — the memo compares old and new values to determine whether to notify subscribers.
4. **`filter_selector` returns `Memo<Option<T>>`** — `None` means the predicate failed.
5. **Use selectors for derived values** — instead of reading full state in components, extract only what you need via selectors.
6. **Access with `move || selector.get()`** — wrap in a reactive closure inside `view!` macros.

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| `use leptos::prelude::*` causing "ambiguous import" | Use `use leptos_store::prelude::create_selector;` |
| Selector not updating in view | Wrap access in `move \|\|` closure: `{move \|\| count.get()}` |
| Selecting non-Clone types | Ensure the selected type implements `Clone + PartialEq` |
| Creating selectors inside render closures | Create selectors once (outside the closure), read them inside |

## Related Skills

- [01-creating-a-store.md](01-creating-a-store.md) — Store definition (selectors read from stores)
- [07-store-composition.md](07-store-composition.md) — MultiStoreSelector for cross-store derived values
- [troubleshooting.md](troubleshooting.md) — "ambiguous import create_selector" fix
