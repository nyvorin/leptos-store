# CSR Deployment — Client-Only SPA with leptos-store

> **When to use:** Building a client-only SPA with `trunk` or `wasm-pack` — no server rendering, no hydration, no actix-web.

## Prerequisites

```toml
# Cargo.toml
[dependencies]
leptos = { version = "0.8", features = ["csr"] }
leptos-store = { version = "0.5", features = ["csr"] }

# Optional: persist state across page reloads
# leptos-store = { version = "0.5", features = ["csr", "persist-web"] }
```

Build tool — install and use [trunk](https://trunkrs.dev/):
```bash
cargo install trunk
trunk serve  # dev server with hot reload
trunk build --release  # production build
```

## CSR vs SSR vs Hydrate

| | CSR | SSR | Hydrate |
|---|---|---|---|
| **Feature flag** | `csr` | `ssr` | `ssr` + `hydrate` |
| **Server needed** | No | Yes | Yes |
| **State creation** | `Store::new()` | `Store::new()` per request | `from_hydrated_state()` |
| **State transfer** | None | Fresh per request | JSON embedded in HTML |
| **Serde required** | No | No | Yes (`Serialize + Deserialize`) |
| **Bundle impact** | Minimal | N/A (server binary) | +~50KB (serde + web-sys) |
| **Context function** | `mount_csr_store()` | `provide_store()` | `provide_hydrated_store()` / `use_hydrated_store()` |
| **When to use** | SPAs, static hosting, GitHub Pages | API-driven server apps | SEO, fast initial paint, server-rendered interactivity |

## CSR State Initialization

CSR uses `Store::new()` — no serde, no hydration script, no server. State starts from defaults on every page load.

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub count: i32,
    pub name: String,
}

#[derive(Clone)]
pub struct AppStore {
    state: RwSignal<AppState>,
}

impl AppStore {
    pub fn new() -> Self {
        Self { state: RwSignal::new(AppState::default()) }
    }

    pub fn count(&self) -> i32 {
        self.state.with(|s| s.count)
    }

    pub fn increment(&self) {
        self.state.update(|s| s.count += 1);
    }
}

impl Store for AppStore {
    type State = AppState;
    fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
}

#[component]
fn App() -> impl IntoView {
    let store = AppStore::new();
    mount_csr_store(store);

    view! { <Counter /> }
}

#[component]
fn Counter() -> impl IntoView {
    let store = use_store::<AppStore>();
    view! {
        <p>"Count: " {move || store.count()}</p>
        <button on:click=move |_| store.increment()>"+"</button>
    }
}
```

**Key difference from SSR:** No `Serialize`/`Deserialize` derives needed. No `HydratableStore` impl. No hydration script in the HTML.

## CSR + Persistence

CSR state is lost on every page refresh. Use `persist-web` to save and restore automatically:

```toml
# Cargo.toml
[dependencies]
leptos-store = { version = "0.5", features = ["csr", "persist-web"] }
serde = { version = "1", features = ["derive"] }
```

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;
use serde::{Serialize, Deserialize};

// State needs Serialize + Deserialize for persistence
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AppState {
    pub count: i32,
    pub preferences: UserPrefs,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserPrefs {
    pub theme: String,
    pub language: String,
}

#[derive(Clone)]
pub struct AppStore {
    state: RwSignal<AppState>,
}

impl AppStore {
    pub fn new() -> Self {
        Self { state: RwSignal::new(AppState::default()) }
    }
}

impl Store for AppStore {
    type State = AppState;
    fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
}

#[component]
fn App() -> impl IntoView {
    // Create store with default state
    let store = AppStore::new();

    // Wrap with localStorage persistence
    let persistent = PersistentStore::new(store)
        .with_adapter(LocalStorageAdapter::new())
        .with_key("my-app-state");

    // Load previously saved state (if any)
    persistent.load();

    // Provide the persistent store to the tree
    mount_csr_store(persistent);

    view! { <MainApp /> }
}
```

State is now automatically saved to `localStorage` on changes and restored on page load.

## CSR + Async Actions

Fetching initial data from an API on mount:

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct TodoState {
    pub items: Vec<String>,
    pub loading: bool,
}

#[derive(Clone)]
pub struct TodoStore {
    state: RwSignal<TodoState>,
}

impl TodoStore {
    pub fn new() -> Self {
        Self { state: RwSignal::new(TodoState::default()) }
    }

    pub fn set_loading(&self, loading: bool) {
        self.state.update(|s| s.loading = loading);
    }

    pub fn set_items(&self, items: Vec<String>) {
        self.state.update(|s| s.items = items);
    }

    pub fn is_loading(&self) -> bool {
        self.state.with(|s| s.loading)
    }

    pub fn items(&self) -> Vec<String> {
        self.state.with(|s| s.items.clone())
    }
}

impl Store for TodoStore {
    type State = TodoState;
    fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
}

#[component]
fn App() -> impl IntoView {
    let store = TodoStore::new();
    mount_csr_store(store);

    // Fetch data on mount
    let store_fetch = use_store::<TodoStore>();
    Effect::new(move || {
        let store = store_fetch.clone();
        leptos::task::spawn_local(async move {
            store.set_loading(true);
            // Fetch from your API
            let items = fetch_todos_from_api().await;
            store.set_items(items);
            store.set_loading(false);
        });
    });

    view! { <TodoList /> }
}
```

## Complete CSR App

Full project structure for a CSR app:

### `Cargo.toml`

```toml
[package]
name = "my-csr-app"
version = "0.1.0"
edition = "2024"

[dependencies]
leptos = { version = "0.8", features = ["csr"] }
leptos-store = { version = "0.5", features = ["csr"] }
console_error_panic_hook = "0.1"
wasm-bindgen = "0.2"
```

### `index.html` (for trunk)

```html
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8" />
    <title>My CSR App</title>
    <link data-trunk rel="rust" data-wasm-opt="z" />
</head>
<body></body>
</html>
```

### `src/main.rs`

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;

#[derive(Clone, Debug, Default)]
pub struct CounterState {
    pub count: i32,
}

#[derive(Clone)]
pub struct CounterStore {
    state: RwSignal<CounterState>,
}

impl CounterStore {
    pub fn new() -> Self {
        Self { state: RwSignal::new(CounterState::default()) }
    }

    pub fn count(&self) -> i32 {
        self.state.with(|s| s.count)
    }

    pub fn increment(&self) {
        self.state.update(|s| s.count += 1);
    }

    pub fn decrement(&self) {
        self.state.update(|s| s.count -= 1);
    }
}

impl Store for CounterStore {
    type State = CounterState;
    fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
}

#[component]
fn App() -> impl IntoView {
    let store = CounterStore::new();
    mount_csr_store(store);

    view! { <Counter /> }
}

#[component]
fn Counter() -> impl IntoView {
    let store = use_store::<CounterStore>();

    view! {
        <div>
            <h1>"Counter: " {move || store.count()}</h1>
            <button on:click=move |_| store.increment()>"+"</button>
            <button on:click=move |_| store.decrement()>"-"</button>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
```

### Build and run

```bash
trunk serve        # Development with hot reload
trunk build --release  # Production build → dist/
```

## Common Mistakes

### 1. Wrong feature flag

```toml
# WRONG — ssr is for server rendering
leptos-store = { version = "0.5", features = ["ssr"] }

# WRONG — hydrate adds ~50KB of unnecessary WASM
leptos-store = { version = "0.5", features = ["hydrate"] }

# CORRECT — csr for client-only apps
leptos-store = { version = "0.5", features = ["csr"] }
```

### 2. Accidentally enabling hydrate

Enabling `hydrate` in a CSR app adds serde, serde_json, web-sys, and wasm-bindgen — roughly **~50KB** of unnecessary WASM. Use `csr` instead.

### 3. Not using console_error_panic_hook

Without this, WASM panics show as unhelpful `unreachable` errors in the browser console:

```rust
fn main() {
    console_error_panic_hook::set_once();  // Always add this
    leptos::mount::mount_to_body(App);
}
```

### 4. Trying to use server functions in CSR

Server functions (`#[server]`) don't exist in CSR builds. Use `fetch` or `reqwest` (with WASM target) to call external APIs:

```rust
// WRONG in CSR — server functions require SSR
#[server]
async fn get_data() -> Result<Data, ServerFnError> { ... }

// CORRECT in CSR — use client-side HTTP
async fn get_data() -> Result<Data, String> {
    // Use gloo_net, reqwest, or raw web_sys::fetch
    todo!()
}
```

### 5. State lost on page refresh

CSR state lives entirely in WASM memory. Every page refresh starts fresh. **Solution:** Add `persist-web` feature and wrap your store with `PersistentStore` + `LocalStorageAdapter` (see CSR + Persistence section above).

### 6. `mount_csr_store` vs `provide_store` confusion

They are functionally identical. `mount_csr_store` exists purely for semantic clarity — use it in CSR apps so your code communicates intent. In SSR/Hydrate apps, use `provide_store` or `provide_hydrated_store` instead.

## Key Rules

1. **Use `features = ["csr"]` for CSR apps** — never `ssr`, never `hydrate`.
2. **No serde needed** for CSR-only state — only add `Serialize`/`Deserialize` if you also use `persist-web`.
3. **Never enable `hydrate` for CSR apps** — it adds ~50KB of unnecessary WASM dependencies.
4. **Always add `console_error_panic_hook`** — essential for debugging WASM panics.
5. **State is ephemeral without persistence** — add `persist-web` if state should survive page reloads.
6. **Use `mount_csr_store()`** not `provide_store()` — semantically communicates CSR intent (though both work).
7. **No server functions in CSR** — use client-side HTTP for API calls.

## Related Skills

- [01-creating-a-store.md](01-creating-a-store.md) — Store definition basics
- [04-persistence.md](04-persistence.md) — Detailed persistence patterns
- [05-ssr-hydration.md](05-ssr-hydration.md) — SSR/Hydrate patterns (contrast with CSR)
- [architecture-guide.md](architecture-guide.md) — Choosing between CSR, SSR, and hydrate
- [troubleshooting.md](troubleshooting.md) — Common errors
