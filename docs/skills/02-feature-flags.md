# Feature Flags

## When to Use
You need feature toggles, A/B testing, gradual rollouts, or conditional rendering based on feature availability.

## Prerequisites
```toml
# Cargo.toml
[dependencies]
leptos-store = { version = "0.5", features = ["templates"] }
```

The `templates` feature implies `hydrate`, which adds `serde` + `web-sys` to the WASM bundle.

## Pattern: Feature Flag Store

### Creating and Providing Flags

```rust
use leptos::prelude::*;
use leptos_store::templates::feature_flags::*;

#[component]
fn App() -> impl IntoView {
    // Create store with initial flags
    let store = FeatureFlagStore::with_flags(vec![
        FeatureFlag::new("dark_mode", true),
        FeatureFlag::new("beta_features", false),
        FeatureFlag::with_variant("homepage_hero", true, "variant_a"),
    ]);

    // Provide via Leptos context
    provide_context(store);

    view! { <MainContent /> }
}
```

### FeatureFlag Constructors

```rust
// Simple on/off flag
let flag = FeatureFlag::new("dark_mode", true);

// Flag with variant (for A/B testing)
let flag = FeatureFlag::with_variant("hero_style", true, "modern");

// Flag with metadata and description
let flag = FeatureFlag::new("premium_feature", false)
    .with_description("Premium-only checkout flow")
    .with_metadata("owner", "team-payments");
```

### FeatureFlagStore API

```rust
let store = FeatureFlagStore::new();

// Set all flags (replaces existing)
store.set_flags(vec![
    FeatureFlag::new("feature_a", true),
    FeatureFlag::new("feature_b", false),
]);

// Getters
store.is_enabled("feature_a");           // -> bool
store.get_variant("hero_style");         // -> Option<String>
store.get_flag("feature_a");             // -> Option<FeatureFlag>
store.all_flags();                       // -> Vec<FeatureFlag>
store.flag_keys();                       // -> Vec<String>
store.is_loaded();                       // -> bool
store.is_loading();                      // -> bool

// Actions
store.enable("feature_b");              // Set flag to true
store.disable("feature_a");             // Set flag to false
store.toggle("feature_a");              // Flip flag value

// Individual flag management
store.set_flag(FeatureFlag::new("new_flag", true));
store.remove_flag("old_flag");

// Development overrides (checked before actual flag value)
store.set_override("experimental", true);
store.remove_override("experimental");
store.clear_overrides();
```

### Override Precedence

When checking `is_enabled()`, the store checks overrides first:

1. If an override exists for the key, return the override value
2. Otherwise, return the actual flag value
3. If the flag doesn't exist, return `false`

Overrides are transient — they are not serialized and don't affect persisted state.

## Pattern: Conditional Rendering Components

### `<Feature>` — Show/Hide Based on Flag

```rust
use leptos_store::templates::feature_flags::*;

#[component]
fn App() -> impl IntoView {
    view! {
        // Render children only if flag is enabled
        <Feature flag="dark_mode">
            <DarkModeToggle />
        </Feature>

        // Invert: render if flag is DISABLED
        <Feature flag="legacy_checkout" invert=true>
            <NewCheckoutFlow />
        </Feature>
    }
}
```

### `<FeatureVariant>` — A/B Testing

```rust
#[component]
fn HeroSection() -> impl IntoView {
    view! {
        <FeatureVariant flag="hero_style" variant="modern">
            <ModernHero />
        </FeatureVariant>
        <FeatureVariant flag="hero_style" variant="classic">
            <ClassicHero />
        </FeatureVariant>
    }
}
```

### Convenience Functions

```rust
// Check flag programmatically in a component
let is_dark = use_feature("dark_mode");
// Use in view: {move || if is_dark() { "dark" } else { "light" }}

// Access the full store
let flags = use_feature_flags();
```

## Pattern: Context Helpers

```rust
// Provide (alternative to provide_context)
provide_feature_flags(store);

// Access (alternative to use_context)
let store = use_feature_flags();

// Reactive check (returns closure)
let is_enabled = use_feature("my_flag");
```

## Key Rules

1. **`templates` feature implies `hydrate`** — this adds serde + web-sys to the WASM bundle. Only enable if you need feature flags.

2. **Flag keys are `&'static str` in components** — the `<Feature>` and `<FeatureVariant>` components require `&'static str` for flag keys.

3. **Overrides are development-only** — they are `#[serde(skip)]` so they are not serialized. Use them for local testing, not production logic.

4. **Unknown flags return `false`** — `is_enabled()` returns `false` for flags that don't exist, providing safe defaults.

5. **Provide via `provide_context(store)`** — the `<Feature>` component uses `use_context::<FeatureFlagStore>()` internally.

## Common Mistakes

- Forgetting to add `features = ["templates"]` in Cargo.toml
- Using `provide_store()` instead of `provide_context()` — the Feature component expects raw context
- Checking flags before `set_flags()` — `is_loaded()` will be false and all flags return false

## Related Skills
- [05-ssr-hydration.md](05-ssr-hydration.md) — FeatureFlagStore supports hydration automatically
- [01-creating-a-store.md](01-creating-a-store.md) — Understanding the Store trait
- [architecture-guide.md](architecture-guide.md) — When to enable the templates feature
