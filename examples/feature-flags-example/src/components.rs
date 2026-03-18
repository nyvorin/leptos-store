// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! UI Components for the Feature Flags Example

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

#[cfg(any(feature = "hydrate", feature = "ssr"))]
use leptos_store::templates::feature_flags::{
    Feature, FeatureFlag, FeatureFlagStore, FeatureVariant,
};

/// Embeddable demo component for the showcase.
///
/// Creates a FeatureFlagStore with default flags and renders the landing page.
#[component]
pub fn Demo() -> impl IntoView {
    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    {
        let flags = FeatureFlagStore::new();
        flags.set_flags(vec![
            FeatureFlag::new("dark_mode", true),
            FeatureFlag::new("beta_features", false),
            FeatureFlag::new("new_hero", true),
            FeatureFlag::with_variant("hero_style", true, "modern"),
            FeatureFlag::new("premium_content", false),
        ]);
        provide_context(flags);
    }

    view! { <LandingPage /> }
}

/// Main app component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Create and configure feature flags
    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    {
        let flags = FeatureFlagStore::new();

        // Set up feature flags
        flags.set_flags(vec![
            FeatureFlag::new("dark_mode", true),
            FeatureFlag::new("beta_features", false),
            FeatureFlag::new("new_hero", true),
            FeatureFlag::with_variant("hero_style", true, "modern"),
            FeatureFlag::new("premium_content", false),
        ]);

        provide_context(flags);
    }

    view! {
        <Stylesheet id="leptos" href="/pkg/feature-flags-example.css"/>
        <Title text="Feature Flags Example - leptos-store"/>
        <Meta name="description" content="Feature flags demonstration using leptos-store"/>

        <Router>
            <main class="app">
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=LandingPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Landing page with feature flags
#[component]
fn LandingPage() -> impl IntoView {
    view! {
        <div class="landing-page">
            <header class="landing-header">
                <h1>"Feature Flags Example"</h1>
                <p class="subtitle">"Conditional rendering based on feature flags"</p>
            </header>

            <FlagControls />

            <div class="feature-demos">
                <HeroSection />
                <BetaFeatures />
                <PremiumContent />
            </div>

            <div class="code-hint">
                <p>"Using the Feature component:"</p>
                <pre><code>{r#"<Feature flag="beta_features">
    <BetaWidget />
</Feature>

<FeatureVariant flag="hero_style" variant="modern">
    <ModernHero />
</FeatureVariant>"#}</code></pre>
            </div>
        </div>
    }
}

/// Flag control panel
#[component]
fn FlagControls() -> impl IntoView {
    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    {
        let flags = use_context::<FeatureFlagStore>();

        if let Some(flags) = flags {
            let flags_list = flags.clone();
            let flags_toggle = flags.clone();

            return view! {
                <div class="flag-controls">
                    <h2>"Flag Controls"</h2>
                    <div class="flag-list">
                        {move || {
                            flags_list.all_flags().into_iter().map(|flag| {
                                let flags_t = flags_toggle.clone();
                                let flag_key = flag.key.clone();
                                let flag_key_display = flag.key.clone();

                                view! {
                                    <div class="flag-item">
                                        <label class="flag-toggle">
                                            <input
                                                type="checkbox"
                                                checked=flag.enabled
                                                on:change=move |_| {
                                                    flags_t.toggle(&flag_key);
                                                }
                                            />
                                            <span class="toggle-slider"></span>
                                        </label>
                                        <span class="flag-name">{flag_key_display}</span>
                                        {flag.variant.map(|v| view! {
                                            <span class="flag-variant">{format!("({})", v)}</span>
                                        })}
                                    </div>
                                }
                            }).collect_view()
                        }}
                    </div>
                </div>
            }
            .into_any();
        }
    }

    view! {
        <div class="flag-controls">
            <p>"Feature flags require the 'templates' feature"</p>
        </div>
    }
    .into_any()
}

/// Hero section with variants
#[component]
fn HeroSection() -> impl IntoView {
    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    {
        return view! {
            <section class="hero-section">
                <h2>"Hero Section"</h2>

                <Feature flag="new_hero">
                    <div class="hero new-hero">
                        <h3>"Welcome to the Future"</h3>
                        <p>"Experience our new hero design"</p>
                        <button class="btn btn-hero">"Get Started"</button>
                    </div>
                </Feature>

                <Feature flag="new_hero" invert=true>
                    <div class="hero classic-hero">
                        <h3>"Welcome"</h3>
                        <p>"Classic hero design"</p>
                    </div>
                </Feature>

                <div class="variant-demo">
                    <p class="variant-label">"Hero Style Variant:"</p>
                    <FeatureVariant flag="hero_style" variant="modern">
                        <span class="variant-badge modern">"Modern Style Active"</span>
                    </FeatureVariant>
                    <FeatureVariant flag="hero_style" variant="classic">
                        <span class="variant-badge classic">"Classic Style Active"</span>
                    </FeatureVariant>
                    <FeatureVariant flag="hero_style" variant="minimal">
                        <span class="variant-badge minimal">"Minimal Style Active"</span>
                    </FeatureVariant>
                </div>
            </section>
        }
        .into_any();
    }

    #[cfg(not(any(feature = "hydrate", feature = "ssr")))]
    view! {
        <section class="hero-section">
            <p>"Requires 'templates' feature"</p>
        </section>
    }
    .into_any()
}

/// Beta features section
#[component]
fn BetaFeatures() -> impl IntoView {
    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    {
        return view! {
            <section class="beta-section">
                <h2>"Beta Features"</h2>

                <Feature flag="beta_features">
                    <div class="beta-content">
                        <div class="beta-badge">"BETA"</div>
                        <h3>"Experimental Features"</h3>
                        <ul>
                            <li>"AI-powered suggestions"</li>
                            <li>"Real-time collaboration"</li>
                            <li>"Advanced analytics"</li>
                        </ul>
                    </div>
                </Feature>

                <Feature flag="beta_features" invert=true>
                    <div class="beta-locked">
                        <span class="lock-icon">"🔒"</span>
                        <p>"Enable 'beta_features' flag to see beta content"</p>
                    </div>
                </Feature>
            </section>
        }
        .into_any();
    }

    #[cfg(not(any(feature = "hydrate", feature = "ssr")))]
    view! {
        <section class="beta-section">
            <p>"Requires 'templates' feature"</p>
        </section>
    }
    .into_any()
}

/// Premium content section
#[component]
fn PremiumContent() -> impl IntoView {
    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    {
        return view! {
            <section class="premium-section">
                <h2>"Premium Content"</h2>

                <Feature flag="premium_content">
                    <div class="premium-content">
                        <div class="premium-badge">"PREMIUM"</div>
                        <h3>"Exclusive Access"</h3>
                        <p>"You have access to premium features!"</p>
                    </div>
                </Feature>

                <Feature flag="premium_content" invert=true>
                    <div class="premium-upsell">
                        <h3>"Upgrade to Premium"</h3>
                        <p>"Enable 'premium_content' to unlock exclusive features"</p>
                        <button class="btn btn-upgrade">"Upgrade Now"</button>
                    </div>
                </Feature>
            </section>
        }
        .into_any();
    }

    #[cfg(not(any(feature = "hydrate", feature = "ssr")))]
    view! {
        <section class="premium-section">
            <p>"Requires 'templates' feature"</p>
        </section>
    }
    .into_any()
}
