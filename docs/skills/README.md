# leptos-store Skills for AI Agents

Agent-optimized implementation guides for leptos-store v0.5.
Structured for Context7 consumption — not user documentation.

## How to Use

When a developer asks about leptos-store, query these skills by topic.
Start with the architecture guide for new projects, or jump to a
specific pattern skill for targeted questions.

## Skills Index

### Start Here
- [architecture-guide.md](architecture-guide.md) — Design decisions before writing code

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

## Key Rules (All Skills)
- Use `this` not `self` in store! macro bodies (Rust 2024 edition hygiene)
- Use explicit imports, never glob imports from `leptos::prelude` (avoids `create_selector` ambiguity)
- Always specify required feature gates in Cargo.toml
- Code examples are complete and compilable
- leptos-store requires Leptos 0.8+ and Rust 1.92+ (2024 edition)

## Version
- leptos-store: 0.5.0
- Leptos: 0.8
- Rust edition: 2024
- MSRV: 1.92
