# SSR Hydration

## When to Use
You're building an SSR app with Leptos and need store state to transfer from server to client without hydration mismatches.

## Prerequisites
```toml
# Cargo.toml
[dependencies]
leptos-store = { version = "0.5", features = ["ssr", "hydrate"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Feature setup for SSR + client hydration:
[features]
ssr = ["leptos/ssr", "leptos-store/ssr"]
hydrate = ["leptos/hydrate", "leptos-store/hydrate"]
```

The `hydrate` feature adds `serde`, `serde_json`, `web-sys`, and `wasm-bindgen` — approximately **~50KB to WASM bundle**. Only enable for SSR apps that need server-to-client state transfer.

## Pattern: HydratableStore Trait

Implement `HydratableStore` to enable server → client state transfer:

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;
use serde::{Serialize, Deserialize};

// State must derive these 5 traits:
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CounterState {
    pub count: i32,
    pub last_updated: Option<String>,
}

#[derive(Clone)]
pub struct CounterStore {
    state: RwSignal<CounterState>,
}

impl CounterStore {
    pub fn new() -> Self {
        Self { state: RwSignal::new(CounterState::default()) }
    }
}

impl Store for CounterStore {
    type State = CounterState;
    fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
}

impl HydratableStore for CounterStore {
    fn serialize_state(&self) -> Result<String, StoreHydrationError> {
        let state = self.state.get();
        serde_json::to_string(&state)
            .map_err(|e| StoreHydrationError::Serialization(e.to_string()))
    }

    fn from_hydrated_state(data: &str) -> Result<Self, StoreHydrationError> {
        let state: CounterState = serde_json::from_str(data)
            .map_err(|e| StoreHydrationError::Deserialization(e.to_string()))?;
        Ok(Self { state: RwSignal::new(state) })
    }

    fn store_key() -> &'static str {
        "counter"  // Must be unique across all stores
    }
}
```

### Using the impl_hydratable_store! Macro

For stores with a single `state: RwSignal<S>` field:

```rust
use leptos_store::{impl_store, impl_hydratable_store};

impl_store!(CounterStore, CounterState, state);
impl_hydratable_store!(CounterStore, "counter");
```

## Pattern: Server-Side Rendering

On the server, use `provide_hydrated_store()` which:
1. Provides the store to the component tree
2. Serializes the state to JSON
3. Returns a `<script>` tag to embed in the HTML

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let store = CounterStore::new();
    // This returns a <script> tag with serialized state
    let hydration_script = provide_hydrated_store(store);

    view! {
        {hydration_script}
        <MainContent />
    }
}
```

The rendered HTML includes:
```html
<script id="__LEPTOS_STORE_STATE__counter" type="application/json">
  {"count":0,"last_updated":null}
</script>
```

## Pattern: Client-Side Hydration

On the client, use `use_hydrated_store()` which:
1. Checks the DOM for a hydration script tag
2. If found, deserializes and creates the store from that data
3. If not found, falls back to regular context lookup

```rust
#[component]
fn Counter() -> impl IntoView {
    // Automatically hydrates from DOM data if available
    let store = use_hydrated_store::<CounterStore>();

    view! {
        <span>{move || store.state().get().count}</span>
    }
}
```

### Non-Panicking Alternative

```rust
match try_use_hydrated_store::<CounterStore>() {
    Ok(store) => { /* Use hydrated store */ }
    Err(e) => { /* Handle error, use fallback */ }
}
```

## Pattern: HydrationBuilder

For controlled hydration with fallback:

```rust
let store = HydrationBuilder::<CounterStore>::new()
    .with_fallback(CounterStore::new())  // Use if hydration fails
    .build();  // Panics without fallback if hydration fails

// Or non-panicking:
let store = HydrationBuilder::<CounterStore>::new()
    .with_fallback(CounterStore::new())
    .try_build()?;
```

## Pattern: Manual Hydration Script

Generate hydration HTML manually (e.g., in actix-web handlers):

```rust
use leptos_store::prelude::*;

let store = CounterStore::new();
let json = store.serialize_state().unwrap();
let html = hydration_script_html("counter", &json);
// Returns: <script id="__LEPTOS_STORE_STATE__counter" type="application/json">...</script>
```

## Pattern: Complete SSR App Setup

### Server (actix-web)

```rust
// In your server setup (lib.rs or main.rs):
#[component]
pub fn App() -> impl IntoView {
    let counter = CounterStore::new();
    let hydration_script = provide_hydrated_store(counter);

    view! {
        {hydration_script}
        <Router>
            <Routes fallback=|| "Not found">
                <Route path=path!("/") view=HomePage />
            </Routes>
        </Router>
    }
}
```

### Client (WASM)

```rust
// In your client hydration entry point:
#[component]
fn HomePage() -> impl IntoView {
    let store = use_hydrated_store::<CounterStore>();
    let store_display = store.clone();
    let store_inc = store.clone();

    view! {
        <p>"Count: " {move || store_display.state().get().count}</p>
        <button on:click=move |_| {
            // Call store actions to modify state
        }>"Increment"</button>
    }
}
```

## How It Works

```
Server                              Client
──────                              ──────
1. Create store                     1. Page loads with HTML
2. provide_hydrated_store(store)    2. WASM starts
3. Renders <script> with JSON       3. use_hydrated_store::<S>()
4. Sends full HTML to client        4. Reads <script> from DOM
                                    5. Deserializes JSON → State
                                    6. Creates store with state
                                    7. Reactive UI takes over
```

## Key Rules

1. **`hydrate` adds ~50KB to WASM** — only enable for SSR apps. Use `csr` feature for client-only apps.

2. **Server: `provide_hydrated_store()`; Client: `use_hydrated_store()`** — these are the paired functions. Don't mix them up.

3. **State types need 5 derives**: `Clone, Debug, Default, Serialize, Deserialize`.

4. **`store_key()` must be unique** across all stores — it's used as the DOM element ID.

5. **Don't enable `hydrate` for CSR-only apps** — use `features = ["csr"]` instead. The hydrate feature adds unnecessary serde/web-sys dependencies.

6. **Hydration mismatch** occurs when server-rendered state doesn't match client expectations. Ensure both sides use the same default state and serialization format.

## Common Mistakes

- Enabling `hydrate` in a CSR-only app (wastes ~50KB in WASM)
- Forgetting `Serialize, Deserialize` derives on the state type
- Using `provide_store()` instead of `provide_hydrated_store()` on the server (no script tag generated)
- Using `use_store()` instead of `use_hydrated_store()` on the client (skips DOM hydration)
- Duplicate `store_key()` values across different stores
- State fields that aren't serializable (e.g., `RwSignal` inside state)

## Related Skills
- [01-creating-a-store.md](01-creating-a-store.md) — Base store creation
- [04-persistence.md](04-persistence.md) — Client-side persistence (different from hydration)
- [02-feature-flags.md](02-feature-flags.md) — FeatureFlagStore has built-in hydration support
- [architecture-guide.md](architecture-guide.md) — Choosing between CSR, SSR, and hydrate features
