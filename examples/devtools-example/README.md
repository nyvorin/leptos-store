# Devtools Example

This example demonstrates the **devtools integration** in leptos-store.

## Features Demonstrated

- **Console API**: `window.__LEPTOS_STORE__` for browser debugging
- **StoreInspector**: Floating debug panel component
- **Event tracking**: Monitor store operations

## Running the Example

```bash
# SSR mode (recommended)
cd examples/devtools-example
cargo leptos watch

# CSR mode
trunk serve
```

## Key Concepts

### Initializing Devtools

```rust
#[cfg(feature = "devtools")]
init_devtools();
```

### Adding the Inspector

```rust
#[cfg(feature = "devtools")]
view! { <StoreInspector /> }
```

### Console API

Open browser DevTools console and try:

```javascript
// Show help
__LEPTOS_STORE__.help()

// List registered stores
__LEPTOS_STORE__.getStores()

// Get state of a store
__LEPTOS_STORE__.getState("counter")

// View event history
__LEPTOS_STORE__.getEvents()
```

## What to Look For

1. Look for the floating inspector panel (bottom-right)
2. Click to expand and view stores
3. Open browser console
4. Try the `__LEPTOS_STORE__` commands
5. Interact with the counter and watch events
