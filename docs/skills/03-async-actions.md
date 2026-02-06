# Async Actions

## When to Use
You need to perform asynchronous operations (API calls, timers, database queries) and update store state with the results.

## Prerequisites
```toml
# Cargo.toml
[dependencies]
leptos-store = "0.5"
leptos = "0.8"
```

No extra feature gates needed — async actions are always available.

## Pattern: AsyncAction Trait

The `AsyncAction<S>` trait defines an async operation tied to a store. It has two associated types: `Output` (success value) and `Error` (failure value).

### Defining an Async Action

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;
use std::error::Error;
use std::fmt;

// 1. Define your store
#[derive(Clone, Default)]
struct AuthState {
    token: Option<String>,
    username: Option<String>,
}

#[derive(Clone)]
struct AuthStore {
    state: RwSignal<AuthState>,
}

impl AuthStore {
    fn new() -> Self {
        Self { state: RwSignal::new(AuthState::default()) }
    }

    fn set_token(&self, token: String) {
        self.state.update(|s| s.token = Some(token));
    }

    fn set_username(&self, name: String) {
        self.state.update(|s| s.username = Some(name));
    }
}

impl Store for AuthStore {
    type State = AuthState;
    fn state(&self) -> ReadSignal<Self::State> { self.state.read_only() }
}

// 2. Define your error type
#[derive(Debug)]
struct AuthError(String);

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl Error for AuthError {}

// 3. Define the async action
pub struct LoginAction {
    pub email: String,
    pub password: String,
}

impl AsyncAction<AuthStore> for LoginAction {
    type Output = String;  // Returns a token
    type Error = AuthError;

    async fn execute(
        &self,
        _store: &AuthStore,
    ) -> ActionResult<Self::Output, Self::Error> {
        // Simulate API call
        if self.email.is_empty() {
            return Err(ActionError::validation("Email required"));
        }
        Ok("token_abc123".to_string())
    }
}
```

### Using the define_async_action! Macro

For simpler action definitions, use the macro:

```rust
use leptos_store::define_async_action;

define_async_action! {
    /// Fetches user data from the API
    #[derive(Debug, Clone)]
    pub FetchUserAction {
        user_id: String,
    } -> Result<String, AuthError>
}

let action = FetchUserAction::new("user_123".to_string());
```

## Pattern: ActionState Tracking

`ActionState` tracks the lifecycle of an async operation:

```rust
use leptos_store::prelude::ActionState;

let state = ActionState::Idle;     // Not started
assert!(state.is_idle());

let state = ActionState::Pending;  // Running
assert!(state.is_pending());
assert!(!state.is_finished());

let state = ActionState::Success;  // Completed OK
assert!(state.is_success());
assert!(state.is_finished());

let state = ActionState::Error;    // Failed
assert!(state.is_error());
assert!(state.is_finished());
```

## Pattern: ReactiveAction in Components

`ReactiveAction` provides reactive signals for tracking action state in the UI:

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;

#[component]
fn LoginForm() -> impl IntoView {
    let action: ReactiveAction<String, String> = ReactiveAction::new();

    let action_submit = action.clone();
    let action_pending = action.clone();
    let action_value = action.clone();

    let on_submit = move |_| {
        let handle = action_submit.dispatch("user@example.com".to_string());

        // Simulate async completion
        // In real code: spawn_local(async move { ... handle.complete(result) })
        handle.complete("token123".to_string());
    };

    view! {
        <button
            on:click=on_submit
            disabled=move || action_pending.pending()
        >
            {move || if action_pending.pending() { "Loading..." } else { "Login" }}
        </button>
        <p>{move || action_value.value().unwrap_or_default()}</p>
    }
}
```

### ActionHandle

When you call `dispatch()`, you get back an `ActionHandle` to complete the action:

```rust
let action: ReactiveAction<String, i32> = ReactiveAction::new();

// Start the action
let handle = action.dispatch("fetch data".to_string());
assert!(action.pending());

// Complete with a value
handle.complete(42);
assert!(!action.pending());
assert_eq!(action.value(), Some(42));
```

Or cancel it:

```rust
let handle = action.dispatch("fetch data".to_string());
handle.cancel();  // Clears pending without setting a value
```

## Pattern: spawn_local for WASM Async

In WASM components, use `spawn_local` to run async code:

```rust
use leptos::prelude::*;
use leptos_store::prelude::*;

fn trigger_login(store: AuthStore, email: String, password: String) {
    leptos::task::spawn_local(async move {
        let action = LoginAction { email, password };
        match action.execute(&store).await {
            Ok(token) => {
                store.set_token(token);
            }
            Err(e) => {
                leptos::logging::error!("Login failed: {}", e);
            }
        }
    });
}
```

## Pattern: AsyncActionBuilder

Configure retry and timeout behavior:

```rust
use leptos_store::prelude::*;

let builder: AsyncActionBuilder<AuthStore, (), ActionError> =
    AsyncActionBuilder::new()
        .with_timeout(5000)   // 5 second timeout
        .with_retry(3);       // Retry up to 3 times

assert_eq!(builder.timeout_ms(), Some(5000));
assert_eq!(builder.retry_count(), 3);
```

## Error Types

```rust
use leptos_store::prelude::ActionError;

// Built-in error variants
let e = ActionError::Cancelled;
let e = ActionError::Timeout(5000);
let e = ActionError::failed("Something broke");
let e = ActionError::network("Connection refused");
let e = ActionError::validation("Invalid email");
```

## Key Rules

1. **Async actions cannot directly modify state** — they must go through the store's public actions or mutators. This preserves the Enterprise Mode invariant.

2. **Always handle both Ok and Err cases** — async operations can fail. Use `ActionState` or pattern matching to show appropriate UI.

3. **Use `spawn_local` for WASM, `tokio::spawn` for SSR** — async runtime differs by platform.

4. **Track loading state in the component** — use `ReactiveAction` signals for UI state (pending, value). Only put shared loading state in the store.

5. **Use `ActionResult<T, E>`** — the standard Result alias: `Result<T, E>` where E defaults to `ActionError`.

## Common Mistakes

- Trying to mutate store state directly inside `execute()` — must call store actions instead
- Forgetting to `spawn_local` the async work in a component event handler
- Not handling the `Err` case, causing silent failures
- Storing `ActionState` in the store when it's only needed locally in one component

## Related Skills
- [01-creating-a-store.md](01-creating-a-store.md) — Store actions that async results flow into
- [04-persistence.md](04-persistence.md) — Persisting state after async updates
- [05-ssr-hydration.md](05-ssr-hydration.md) — Async data fetching in SSR context
