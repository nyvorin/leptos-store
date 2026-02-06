# Architecture Guide — Design Decisions Before Writing Code

> **When to use:** Starting a new leptos-store project, planning which features to enable, or deciding how to structure your stores.

## Decision 1: How Many Stores?

```
What's the app scope?
├── Single page / small app
│   → One store via store! macro
│   → See: 01-creating-a-store.md
│
├── Multi-page with distinct domains (auth, cart, settings)
│   → Multiple stores via RootStore composition
│   → See: 07-store-composition.md
│
└── Micro-frontend or team boundaries
    → RootStore + StoreGroup + namespace! macro
    → See: 07-store-composition.md
```

**Rule of thumb:** Start with one store. Split when you have 3+ distinct domains or different teams owning different state.

## Decision 2: Which Features?

```
What does your app need?
├── Client-only SPA (no server rendering)
│   → features = ["csr"]
│   → See: 09-csr-deployment.md
│
├── Server-rendered + interactive
│   → features = ["ssr", "hydrate"]
│   → hydrate adds serde + web-sys (~50KB WASM)
│
├── Persist state across sessions?
│   → Add "persist-web" (localStorage/sessionStorage)
│   → Or "persist-idb" (IndexedDB, implies persist-web)
│   → See: 04-persistence.md
│
├── Audit trail / debugging / middleware?
│   → Add "middleware" (adds js-sys)
│   → Add "devtools" for browser integration (implies middleware)
│   → See: 08-middleware.md
│
└── Feature flags / A/B testing?
    → Add "templates" (implies hydrate)
    → See: 02-feature-flags.md
```

### Feature Dependency Chart

```
templates ──→ hydrate ──→ serde, serde_json, web-sys, wasm-bindgen
persist-idb → persist-web → web-sys, wasm-bindgen, serde, serde_json, base64, js-sys
devtools ───→ middleware ──→ js-sys
tracing ────→ middleware ──→ js-sys
full = middleware + devtools + persist-web + persist-idb + server-actions + templates
```

### Cargo.toml Templates

**Minimal (CSR-only):**
```toml
[dependencies]
leptos-store = { version = "0.5", features = ["csr"] }
```

**Standard (SSR + Hydrate):**
```toml
[dependencies]
leptos-store = { version = "0.5", features = ["ssr", "hydrate"] }
```

**With persistence:**
```toml
[dependencies]
leptos-store = { version = "0.5", features = ["ssr", "hydrate", "persist-web"] }
```

**Enterprise (everything):**
```toml
[dependencies]
leptos-store = { version = "0.5", features = ["ssr", "hydrate", "middleware", "devtools", "persist-web", "templates"] }
```

**Full (all features):**
```toml
[dependencies]
leptos-store = { version = "0.5", features = ["full"] }
```

## Decision 3: Store Complexity

```
What does the store need to do?
├── Simple state (CRUD, form data)
│   → Basic store! with state + getters + mutators + actions
│   → See: 01-creating-a-store.md
│
├── Derived / computed values
│   → Add selectors for fine-grained reactivity
│   → See: 06-selectors.md
│
├── Server communication (API calls, login)
│   → Add async actions with spawn_local
│   → See: 03-async-actions.md
│
├── Cross-store reactions (cart changes → recalculate totals)
│   → Add middleware coordination via EventBus
│   → See: 08-middleware.md
│
├── Cross-store cache invalidation / dependency ordering
│   → MultiStoreSelector for reactive invalidation
│   → StoreDependencyGraph for explicit dependencies
│   → See: 10-cache-invalidation.md
│
└── State history / undo / audit
    → Add AuditTrail with StateDiff
    → See: 08-middleware.md
```

## Decision 4: Performance Considerations

| Symptom | Solution | Skill |
|---------|----------|-------|
| Many components subscribe to one field | Use selectors for fine-grained reactivity | [06-selectors](06-selectors.md) |
| Large state objects causing slow updates | Split into composed stores via RootStore | [07-store-composition](07-store-composition.md) |
| Frequent rapid updates (typing, dragging) | Selector memoization + persistence debounce | [06-selectors](06-selectors.md), [04-persistence](04-persistence.md) |
| Large WASM bundle | Audit feature gates — remove `hydrate` if CSR-only | [troubleshooting](troubleshooting.md) #9 |

## Key Rules (All Skills)

1. **Use `this` not `self`** in `store!` macro bodies — Rust 2024 edition macro hygiene
2. **Use explicit imports** — never glob import from `leptos::prelude` (avoids `create_selector` ambiguity)
3. **Always specify required feature gates** in Cargo.toml
4. **Enterprise Mode invariant** — actions (public) → mutators (private) → state. Never bypass.
5. **leptos-store requires Leptos 0.8+ and Rust 1.92+** (2024 edition)

## Version Compatibility

| Component | Version |
|-----------|---------|
| leptos-store | 0.5.0 |
| Leptos | 0.8 |
| Rust edition | 2024 |
| MSRV | 1.92 |

## Skills Index

### Start Here
- [architecture-guide.md](architecture-guide.md) — You are here

### Patterns (by priority)
1. [01-creating-a-store.md](01-creating-a-store.md) — The store! macro, state, getters, mutators, actions
2. [02-feature-flags.md](02-feature-flags.md) — Template system for feature management
3. [03-async-actions.md](03-async-actions.md) — Server calls, loading states, error handling
4. [04-persistence.md](04-persistence.md) — Web storage, IndexedDB, server persistence
5. [05-ssr-hydration.md](05-ssr-hydration.md) — HydratableStore, actix-web integration
6. [06-selectors.md](06-selectors.md) — Fine-grained Memo-based reactivity
7. [07-store-composition.md](07-store-composition.md) — RootStore, CompositeStore patterns
8. [08-middleware.md](08-middleware.md) — Audit trails, event bus, coordination
9. [09-csr-deployment.md](09-csr-deployment.md) — Client-only SPA deployment with trunk
10. [10-cache-invalidation.md](10-cache-invalidation.md) — Cross-store reactive cache invalidation

### Diagnostics
- [troubleshooting.md](troubleshooting.md) — All common errors indexed by symptom
