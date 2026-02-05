# Middleware Example

This example demonstrates the **middleware system** in leptos-store.

## Features Demonstrated

- **Interceptor Pattern**: Middleware with `before_mutate` and `after_mutate` hooks
- **Event Bus Pattern**: Subscribers that observe store events
- **Custom Middleware**: Logging and timing middleware implementations

## Running the Example

```bash
# SSR mode (recommended)
cd examples/middleware-example
cargo leptos watch

# CSR mode
trunk serve
```

## Key Concepts

### Middleware Trait

```rust
impl<S: Store> Middleware<S> for ConsoleLoggingMiddleware {
    fn before_mutate(&self, ctx: &MiddlewareContext<S>) -> MiddlewareResult {
        console::log!("Before: {}", ctx.mutation_name());
        MiddlewareResult::Continue
    }

    fn after_mutate(&self, ctx: &MiddlewareContext<S>) -> MiddlewareResult {
        console::log!("After: {} ({:?})", ctx.mutation_name(), ctx.elapsed());
        MiddlewareResult::Continue
    }
}
```

### Event Subscriber

```rust
impl EventSubscriber for MetricsSubscriber {
    fn on_event(&self, event: &StoreEvent) {
        match event {
            StoreEvent::MutationCompleted { name, duration_ms, .. } => {
                record_metric(name, *duration_ms);
            }
            _ => {}
        }
    }
}
```

## What to Look For

1. Open the browser console
2. Add, toggle, and remove tasks
3. Watch the middleware log each operation with timing information
