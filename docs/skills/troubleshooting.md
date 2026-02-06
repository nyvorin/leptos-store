# Troubleshooting — Common Errors Indexed by Symptom

> **When to use:** You hit an error or unexpected behavior while using leptos-store. Find your symptom below for the cause, fix, and prevention.

---

## Compilation Errors

### 1. "ambiguous import `create_selector`"

**Cause:** Leptos 0.8 has a deprecated `create_selector` in `leptos::prelude` that conflicts with leptos-store's version. Glob imports pull in both.

**Fix:**
```rust
// WRONG — glob import causes ambiguity
use leptos::prelude::*;

// CORRECT — explicit import from leptos_store
use leptos_store::prelude::create_selector;
```

**Prevention:** Never use `use leptos::prelude::*;` in files that use leptos-store selectors. Always import explicitly.

---

### 2. "cannot find value `self` in this scope" inside store! macro

**Cause:** Rust 2024 edition macro hygiene prevents `self` from being used in macro-generated code. The `store!` macro binds the first parameter name to `&self`.

**Fix:**
```rust
// WRONG
getters {
    count(self) -> i32 {
        self.read(|s| s.count)  // Error: cannot find value `self`
    }
}

// CORRECT — use `this` (or any identifier)
getters {
    count(this) -> i32 {
        this.read(|s| s.count)
    }
}
```

**Prevention:** Always use `this` as the parameter name in getter, mutator, and action bodies within the `store!` macro.

---

### 3. "feature `X` not found" or "unresolved import"

**Cause:** Missing feature gate in Cargo.toml for the module you're trying to use.

**Fix — Feature Checklist:**

| Module | Required Feature | Cargo.toml |
|--------|-----------------|------------|
| SSR hydration | `hydrate` | `features = ["hydrate"]` |
| Middleware / Audit | `middleware` | `features = ["middleware"]` |
| DevTools | `devtools` | `features = ["devtools"]` (implies `middleware`) |
| localStorage/sessionStorage | `persist-web` | `features = ["persist-web"]` |
| IndexedDB | `persist-idb` | `features = ["persist-idb"]` (implies `persist-web`) |
| Server persistence | `persist-server` | `features = ["persist-server"]` (implies `ssr`) |
| Feature flags template | `templates` | `features = ["templates"]` (implies `hydrate`) |
| Tracing/OpenTelemetry | `tracing` | `features = ["tracing"]` (implies `middleware`) |
| Everything | `full` | `features = ["full"]` |

**Prevention:** Check the Prerequisites section in the relevant skill file before starting.

---

### 4. "trait `Store` not implemented for `MyStore`"

**Cause:** Missing `impl Store` for your store type, or wrong `State` type association.

**Fix:**
```rust
// Option 1: Use store! macro (auto-generates the impl)
store! {
    pub MyStore {
        state MyState { count: i32 = 0 }
    }
}

// Option 2: Manual impl with impl_store! macro
impl_store!(MyStore, MyState, state);

// Option 3: Full manual impl
impl Store for MyStore {
    type State = MyState;
    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}
```

**Prevention:** Use the `store!` macro which auto-generates the `Store` impl.

---

### 5. "Store not found in context" (runtime panic)

**Cause:** `use_store::<S>()` called in a component before `provide_store(s)` in a parent component.

**Fix:**
```rust
// Ensure provide runs in a PARENT component
#[component]
fn App() -> impl IntoView {
    provide_store(MyStore::new());  // Must run first
    view! { <ChildComponent /> }
}

#[component]
fn ChildComponent() -> impl IntoView {
    let store = use_store::<MyStore>();  // Now it works
    // ...
}
```

**Prevention:** Always call `provide_store()` or `provide_root_store()` in a parent component. For non-panicking access, use `try_use_store()` or `use_context::<MyStore>()`.

---

## Runtime Issues

### 6. Store state not updating in UI

**Cause:** Not using reactive access pattern. Store getters must be called inside a `move ||` closure for Leptos to track the dependency.

**Fix:**
```rust
// WRONG — evaluated once, never updates
view! { <p>{store.count()}</p> }

// CORRECT — reactive closure, updates on change
view! { <p>{move || store.count()}</p> }
```

**Prevention:** Always wrap store access in `move ||` closures inside `view!` macros.

---

### 7. Hydration mismatch between SSR and client

**Cause:** Server-rendered state doesn't match what the client expects during hydration.

**Fix checklist:**
1. Same default state on both server and client
2. `store_key()` returns the same string on both sides
3. State type serialization is deterministic (check HashMap ordering)
4. Use `HydrationBuilder` with an explicit `.with_fallback(default)`

**Prevention:** Use `HydrationBuilder::<S>::new().with_fallback(default).build()` for predictable behavior.

---

### 8. Persistence not saving or loading

**Cause:** Multiple possible issues — missing derives, wrong feature gate, key mismatch.

**Fix checklist:**
1. State type has `#[derive(Serialize, Deserialize)]` and `serde = { features = ["derive"] }` in Cargo.toml
2. Correct feature gate enabled (`persist-web` for localStorage, `persist-idb` for IndexedDB)
3. Storage key matches between save and load (`.with_key("same_key")`)
4. Adapter is created correctly (`LocalStorageAdapter`, `SessionStorageAdapter`, etc.)
5. `spawn_local` used for async save/load in WASM context

**Prevention:** Test with `MemoryAdapter` first (no feature gate needed), then switch to real adapter.

---

### 9. WASM bundle unexpectedly large

**Cause:** `hydrate` feature enabled when not needed — it adds serde, serde_json, web-sys, wasm-bindgen (~50KB).

**Fix:**
```toml
# For client-only SPA — use csr, NOT hydrate
[dependencies]
leptos-store = { version = "0.5", features = ["csr"] }

# Only use hydrate for SSR apps that transfer state from server to client
leptos-store = { version = "0.5", features = ["ssr", "hydrate"] }
```

**Prevention:** Only enable `hydrate` for SSR apps that need server-to-client state transfer. Use `csr` for client-only SPAs.

---

## Architecture Confusion

### 10. When to use `hydrate` vs `csr` vs `ssr`

**Decision tree:**

```
Is this an SSR app with client interactivity?
├── YES → features = ["ssr", "hydrate"]
│         Server renders HTML + serialized state
│         Client hydrates from DOM
└── NO
    ├── Client-only SPA? → features = ["csr"]
    │   No server rendering, no hydration overhead
    └── API server only (no client)? → features = ["ssr"]
        Server-side logic only
```

---

### 11. How to share state between components

| Scenario | Solution | Skill |
|----------|----------|-------|
| Single store, parent → children | `provide_store()` + `use_store()` | [01-creating-a-store](01-creating-a-store.md) |
| Multiple domain stores | `RootStore` with `provide_root_store()` | [07-store-composition](07-store-composition.md) |
| Cross-store reactions | `StoreCoordinator` + shared `EventBus` | [08-middleware](08-middleware.md) |
| Derived values from multiple stores | `MultiStoreSelector` or `combine_selectors` | [06-selectors](06-selectors.md), [07-store-composition](07-store-composition.md) |

---

### 12. Store actions vs direct signal updates

**Rule: Use Enterprise Mode.** Actions are the only public write API.

```
Components → call actions (public)
                ↓
Actions → call mutators (private)
                ↓
Mutators → update RwSignal<State>
```

Never update store signals directly from components. Always go through actions. This is the core invariant that enables middleware, audit trails, and debugging.

```rust
// WRONG — bypassing the store's API
store.state.update(|s| s.count += 1);

// CORRECT — using the public action
store.increment();
```

---

## CSR Issues

### 13. `mount_csr_store` vs `provide_store` — which to use?

**Answer:** They are functionally identical. `mount_csr_store()` exists for semantic clarity in CSR apps. Use `mount_csr_store()` in CSR apps, `provide_store()` in SSR apps, and `provide_hydrated_store()` in hydrate apps.

See [09-csr-deployment.md](09-csr-deployment.md) for complete CSR patterns.

---

### 14. CSR app not working — blank page

**Cause:** Usually a build tool issue or wrong feature flag.

**Fix checklist:**
1. Ensure `features = ["csr"]` in Cargo.toml (not `ssr`, not `hydrate`)
2. Install trunk: `cargo install trunk`
3. Create `index.html` with `<link data-trunk rel="rust" />`
4. Run `trunk serve` (not `cargo run`)
5. Add `console_error_panic_hook::set_once()` in `main()` to see WASM errors
6. Check browser console (F12) for errors

---

### 15. CSR state lost on page refresh

**Cause:** CSR state lives in WASM memory — every page refresh starts fresh from defaults.

**Fix:** Add persistence:
```toml
leptos-store = { version = "0.5", features = ["csr", "persist-web"] }
```

Then wrap your store with `PersistentStore` + `LocalStorageAdapter`. See [09-csr-deployment.md](09-csr-deployment.md) § CSR + Persistence and [04-persistence.md](04-persistence.md) for details.

---

## Coordination Issues

### 16. StoreCoordinator not receiving events

**Cause:** EventBus not shared between `MiddlewareStore` and `StoreCoordinator`, or `coord.activate()` not called.

**Fix checklist:**
1. Use `Arc::new(EventBus::new())` and pass `Arc::clone(&bus)` to both `MiddlewareStore::with_event_bus()` and `StoreCoordinator::with_event_bus()`
2. Call `coord.activate()` after registering all rules — rules are inert until activated
3. Use named mutations via `mw_store.mutate("name", || ...)` — direct store method calls bypass middleware

**Prevention:** Always share one `Arc<EventBus>` across all participants. Always call `activate()`.

---

### 17. Circular dependency detected at runtime

**Cause:** Store A depends on Store B, which depends on Store A (directly or transitively).

**Fix:**
```rust
use leptos_store::coordination::StoreDependencyGraph;

let mut graph = StoreDependencyGraph::new();
graph.depends_on(&cart_store, &auth_store);
graph.depends_on(&totals_store, &cart_store);

// Validate at startup — catches cycles before they cause problems
match graph.validate() {
    Ok(()) => { /* acyclic */ }
    Err(e) => panic!("Dependency error: {e}"),
}
```

**Prevention:** Use `StoreDependencyGraph::validate()` at app startup. See [10-cache-invalidation.md](10-cache-invalidation.md) § Dependency Graph.

---

### 18. Cross-store derived data not updating

**Cause:** Using manual coordination when reactive primitives would work better.

**Fix:** For purely derived data (no side effects), use `MultiStoreSelector` or `DerivedView` — they auto-recompute via `Memo<T>`:
```rust
let dashboard = MultiStoreSelector::from_two(
    &cart_store, &pricing_store,
    |cart, pricing| compute_totals(cart, pricing),
);
// Auto-updates when either store changes — no manual invalidation needed
```

Only use `StoreCoordinator` for side effects (API refetches, external cache clearing). See [10-cache-invalidation.md](10-cache-invalidation.md).

---

## Quick Feature Gate Reference

```toml
# Minimal CSR app
leptos-store = { version = "0.5", features = ["csr"] }

# Standard SSR + hydration
leptos-store = { version = "0.5", features = ["ssr", "hydrate"] }

# With persistence
leptos-store = { version = "0.5", features = ["ssr", "hydrate", "persist-web"] }

# With middleware + audit
leptos-store = { version = "0.5", features = ["ssr", "hydrate", "middleware"] }

# Full enterprise
leptos-store = { version = "0.5", features = ["full"] }
```

**Feature dependency chain:**
```
templates → hydrate → serde + web-sys + wasm-bindgen + serde_json
persist-idb → persist-web → web-sys + wasm-bindgen + serde + serde_json + base64 + js-sys
devtools → middleware → js-sys
tracing → middleware → js-sys
```
