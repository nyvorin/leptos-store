# Composition Example

This example demonstrates **store composition** in leptos-store.

## Features Demonstrated

- **RootStore**: Aggregating multiple domain stores
- **Multi-store access**: Accessing individual stores from RootStore
- **Derived state**: Computing values from multiple stores
- **Context integration**: Using `provide_root_store` and `use_root_store`

## Running the Example

```bash
# SSR mode (recommended)
cd examples/composition-example
cargo leptos watch

# CSR mode
trunk serve
```

## Key Concepts

### Creating a RootStore

```rust
let root = RootStore::builder()
    .with_store(AuthStore::new())
    .with_store(CartStore::new())
    .with_store(UiStore::new())
    .build();

provide_root_store(root);
```

### Accessing Stores

```rust
let root = use_root_store();
let auth = root.expect::<AuthStore>();
let cart = root.expect::<CartStore>();
```

### Derived State

```rust
// Computed from multiple stores
let can_checkout = move || {
    auth.is_authenticated() && cart.item_count() > 0
};
```

## What to Look For

1. Login to enable checkout
2. Add items to cart
3. Watch the "Checkout Ready" button enable
4. Notifications appear across store interactions
