# Feature Flags Example

This example demonstrates the **feature flag template** in leptos-store.

## Features Demonstrated

- **FeatureFlagStore**: Managing feature flags
- **Feature component**: Conditional rendering based on flags
- **FeatureVariant**: A/B testing with flag variants
- **Local overrides**: Toggle flags at runtime

## Running the Example

```bash
# SSR mode (recommended)
cd examples/feature-flags-example
cargo leptos watch

# CSR mode
trunk serve
```

## Key Concepts

### Setting Up Flags

```rust
let flags = FeatureFlagStore::new();
flags.set_flags(vec![
    FeatureFlag::new("dark_mode", true),
    FeatureFlag::new("beta_features", false),
    FeatureFlag::with_variant("hero_style", true, "modern"),
]);
provide_context(flags);
```

### Feature Component

```rust
<Feature flag="beta_features">
    <BetaContent />
</Feature>

// With invert (show when disabled)
<Feature flag="beta_features" invert=true>
    <UpgradePrompt />
</Feature>
```

### Feature Variants

```rust
<FeatureVariant flag="hero_style" variant="modern">
    <ModernHero />
</FeatureVariant>
<FeatureVariant flag="hero_style" variant="classic">
    <ClassicHero />
</FeatureVariant>
```

## What to Look For

1. Toggle flags using the controls
2. Watch content appear/disappear
3. Try the "invert" behavior
4. See variant-based rendering
