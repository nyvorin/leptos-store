# Persistence Example

This example demonstrates **localStorage persistence** in leptos-store.

## Features Demonstrated

- **Auto-save**: State is automatically saved to localStorage on changes
- **Auto-load**: State is restored from localStorage on page load
- **Serialization**: Uses serde for JSON serialization

## Running the Example

```bash
# SSR mode (recommended)
cd examples/persistence-example
cargo leptos watch

# CSR mode
trunk serve
```

## Key Concepts

### Saving State

```rust
Effect::new(move |_| {
    let state = store.get_state();
    if let Ok(json) = serde_json::to_string(&state) {
        storage.set_item("notes_store", &json);
    }
});
```

### Loading State

```rust
Effect::new(move |_| {
    if let Ok(Some(data)) = storage.get_item("notes_store") {
        if let Ok(state) = serde_json::from_str(&data) {
            store.load_state(state);
        }
    }
});
```

## What to Look For

1. Add some notes
2. Refresh the page
3. Notes persist across reloads
4. Check DevTools > Application > Local Storage
