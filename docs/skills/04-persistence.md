# Persistence

## When to Use
You need store state to survive page reloads, browser sessions, or sync with a server backend.

## Prerequisites

Choose the feature gate for your storage backend:

```toml
# Cargo.toml — pick one or more:
[dependencies]
leptos-store = { version = "0.5", features = ["persist-web"] }     # localStorage/sessionStorage
leptos-store = { version = "0.5", features = ["persist-idb"] }     # IndexedDB (implies persist-web)
leptos-store = { version = "0.5", features = ["persist-server"] }  # Server-side persistence
serde = { version = "1", features = ["derive"] }                   # Required for state serialization
```

State types **must** derive `Serialize` and `Deserialize`.

## Pattern: PersistentStore Wrapper

Wrap any store with `PersistentStore` to add persistence:

```rust
use leptos_store::prelude::*;

let store = MyStore::new();
let persistent = PersistentStore::new(store, LocalStorageAdapter::new())
    .with_key("my_store")
    .with_debounce(500)
    .with_version(1)
    .with_key_prefix("app_");
```

### Builder Methods

| Method | Default | Description |
|--------|---------|-------------|
| `.with_key(key)` | `""` | Storage key identifier |
| `.with_debounce(ms)` | `100` | Debounce save interval in ms |
| `.with_version(v)` | `1` | Version for migration tracking |
| `.with_auto_save(bool)` | `true` | Auto-save on state changes |
| `.with_auto_load(bool)` | `true` | Auto-load on store creation |
| `.with_key_prefix(prefix)` | `"leptos_store_"` | Prefix for storage keys |

The full storage key is `{prefix}{key}` — e.g., `"leptos_store_my_store"`.

## Available Adapters

| Adapter | Feature | Storage | Capacity | Async |
|---------|---------|---------|----------|-------|
| `MemoryAdapter` | (always) | In-memory | Unlimited | Yes (trivial) |
| `LocalStorageAdapter` | `persist-web` | localStorage | ~5MB | Yes (wrapped sync) |
| `SessionStorageAdapter` | `persist-web` | sessionStorage | ~5MB | Yes (wrapped sync) |
| `IndexedDbAdapter` | `persist-idb` | IndexedDB | ~50MB+ | Yes (native) |
| `ServerSyncAdapter` | `persist-server` | Remote server | Unlimited | Yes (network) |

### MemoryAdapter (Testing)

```rust
use leptos_store::prelude::MemoryAdapter;

let adapter = MemoryAdapter::new();

// Or with initial data
use std::collections::HashMap;
let mut data = HashMap::new();
data.insert("key".to_string(), b"value".to_vec());
let adapter = MemoryAdapter::with_data(data);

// Inspect for testing
assert_eq!(adapter.len(), 1);
let snapshot = adapter.snapshot();
```

### LocalStorageAdapter

```rust
use leptos_store::prelude::LocalStorageAdapter;

let adapter = LocalStorageAdapter::new();
assert!(adapter.is_available()); // false outside browser
```

### SessionStorageAdapter

```rust
use leptos_store::prelude::SessionStorageAdapter;

let adapter = SessionStorageAdapter::new();
// Data is cleared when browser tab closes
```

### IndexedDbAdapter

```rust
use leptos_store::prelude::IndexedDbAdapter;

let adapter = IndexedDbAdapter::new("my_app_db")
    .with_store_name("stores");
```

## Pattern: Save and Load

All persistence operations are async:

```rust
// Save current state
persistent.save().await?;  // -> PersistResult<()>

// Load saved state
match persistent.load().await? {
    Some(state) => { /* Restore state */ }
    None => { /* No saved data, use defaults */ }
}

// Check if saved data exists
let exists = persistent.exists().await?;

// Remove saved data
persistent.remove().await?;
```

## Pattern: Auto-Save with Leptos Effects

Set up reactive auto-save that triggers when state changes:

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;

#[component]
fn App() -> impl IntoView {
    let store = MyStore::new();
    let persistent = PersistentStore::new(store, LocalStorageAdapter::new())
        .with_key("my_store")
        .with_debounce(500);

    // Auto-save: Effect tracks state changes, spawns save
    let persistent_save = persistent.clone();
    Effect::new(move |_| {
        // This runs reactively whenever store state changes
        let _state = persistent_save.inner().state().get();
        let p = persistent_save.clone();
        leptos::task::spawn_local(async move {
            if let Err(e) = p.save().await {
                leptos::logging::error!("Save failed: {}", e);
            }
        });
    });

    // Auto-load on mount
    let persistent_load = persistent.clone();
    Effect::new(move |_| {
        let p = persistent_load.clone();
        leptos::task::spawn_local(async move {
            match p.load().await {
                Ok(Some(_state)) => {
                    // Apply loaded state to store
                    leptos::logging::log!("State loaded from storage");
                }
                Ok(None) => {
                    leptos::logging::log!("No saved state found");
                }
                Err(e) => {
                    leptos::logging::error!("Load failed: {}", e);
                }
            }
        });
    });

    provide_store(persistent);
    view! { <MainContent /> }
}
```

## Pattern: PersistConfig

For advanced configuration:

```rust
use leptos_store::prelude::PersistConfig;

let config = PersistConfig::new("my_store");
assert_eq!(config.full_key(), "leptos_store_my_store");
assert_eq!(config.debounce_ms, 100);
assert_eq!(config.version, 1);
assert!(config.auto_save);
assert!(config.auto_load);
```

## Pattern: Custom Adapter

Implement `PersistenceAdapter` for custom backends:

```rust
use leptos_store::persistence::{PersistenceAdapter, PersistFuture, StorageType};

struct RedisAdapter { /* ... */ }

impl PersistenceAdapter for RedisAdapter {
    fn save<'a>(&'a self, key: &'a str, data: &'a [u8]) -> PersistFuture<'a, ()> {
        Box::pin(async move {
            // Save to Redis
            Ok(())
        })
    }

    fn load<'a>(&'a self, key: &'a str) -> PersistFuture<'a, Option<Vec<u8>>> {
        Box::pin(async move {
            // Load from Redis
            Ok(None)
        })
    }

    fn remove<'a>(&'a self, key: &'a str) -> PersistFuture<'a, ()> {
        Box::pin(async move {
            // Remove from Redis
            Ok(())
        })
    }

    fn storage_type(&self) -> StorageType {
        StorageType::Custom
    }
}
```

## Error Handling

```rust
use leptos_store::prelude::PersistError;

// Common errors:
// PersistError::Serialization(_)    — State failed to serialize
// PersistError::Deserialization(_)  — Stored data couldn't be deserialized
// PersistError::NotAvailable(_)     — Storage backend unavailable
// PersistError::QuotaExceeded       — localStorage/sessionStorage full
// PersistError::VersionMismatch { expected, found } — Migration needed
// PersistError::NotFound(_)         — Key doesn't exist
```

## Key Rules

1. **State types MUST derive `Serialize` + `Deserialize`** — the adapter serializes state to bytes via serde_json.

2. **`persist-idb` implies `persist-web`** — IndexedDB support includes localStorage/sessionStorage.

3. **Use `MemoryAdapter` in tests** — it provides the same async interface without browser APIs.

4. **Debounce saves to avoid thrashing** — rapid state updates (e.g., typing) should be debounced. Default is 100ms, raise to 500ms+ for high-frequency updates.

5. **Version your persisted state** — use `.with_version(n)` so you can detect and migrate stale data. `PersistError::VersionMismatch` tells you when stored data is from an older version.

6. **`PersistentStore` implements `Store`** — you can provide it directly and use all store features through it.

## Common Mistakes

- Forgetting `serde = { features = ["derive"] }` in Cargo.toml
- Not handling `PersistError::VersionMismatch` when state shape changes
- Setting debounce to 0 in production (causes excessive writes)
- Using `LocalStorageAdapter` in tests (unavailable outside browser) — use `MemoryAdapter`
- Not awaiting `.save()` / `.load()` — they return futures

## Related Skills
- [01-creating-a-store.md](01-creating-a-store.md) — Creating the store to persist
- [05-ssr-hydration.md](05-ssr-hydration.md) — Server-side state transfer (different from persistence)
- [architecture-guide.md](architecture-guide.md) — Choosing persistence features
