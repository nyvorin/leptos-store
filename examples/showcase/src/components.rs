// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Showcase components for displaying all leptos-store examples.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::hooks::use_navigate;
use leptos_router::path;
use leptos_store::provide_store;

use crate::demos;
use crate::showcase_store::{Difficulty, ExampleCategory, ExampleInfo, ShowcaseStore};

/// Global styles
const GLOBAL_STYLES: &str = r#"
    * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
    }

    :root {
        --bg: #09090b;
        --bg-card: rgba(255,255,255,0.03);
        --border: rgba(255,255,255,0.06);
        --border-hover: rgba(255,255,255,0.12);
        --text-primary: #fafafa;
        --text-secondary: rgba(255,255,255,0.4);
        --text-muted: rgba(255,255,255,0.25);
        --accent-core: #818cf8;
        --accent-state: #c084fc;
        --accent-advanced: #fbbf24;
        --accent-integration: #34d399;
    }

    body {
        font-family: Inter, -apple-system, 'SF Pro Display', system-ui, sans-serif;
        background: var(--bg);
        color: var(--text-primary);
        line-height: 1.6;
        min-height: 100vh;
        -webkit-font-smoothing: antialiased;
    }

    a {
        color: inherit;
        text-decoration: none;
    }

    button {
        font-family: inherit;
    }

    ::selection {
        background: #818cf8;
        color: white;
    }

    ::-webkit-scrollbar {
        width: 6px;
        height: 6px;
    }

    ::-webkit-scrollbar-track {
        background: transparent;
    }

    ::-webkit-scrollbar-thumb {
        background: rgba(255,255,255,0.08);
        border-radius: 3px;
    }

    ::-webkit-scrollbar-thumb:hover {
        background: rgba(255,255,255,0.15);
    }

    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(10px); }
        to { opacity: 1; transform: translateY(0); }
    }
"#;

/// Render an SVG icon based on example id
fn render_icon(id: &str, color: &str) -> impl IntoView {
    match id {
        "counter" => view! {
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <line x1="12" y1="5" x2="12" y2="19" />
                <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
        }
        .into_any(),
        "auth-store" => view! {
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                <path d="M7 11V7a5 5 0 0 1 10 0v4" />
            </svg>
        }
        .into_any(),
        "token-explorer" => view! {
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="9" cy="9" r="7" />
                <circle cx="15" cy="15" r="7" />
            </svg>
        }
        .into_any(),
        "middleware" => view! {
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M12 2L2 7l10 5 10-5-10-5z" />
                <path d="M2 17l10 5 10-5" />
                <path d="M2 12l10 5 10-5" />
            </svg>
        }
        .into_any(),
        "persistence" => view! {
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z" />
                <polyline points="17 21 17 13 7 13 7 21" />
                <polyline points="7 3 7 8 15 8" />
            </svg>
        }
        .into_any(),
        "composition" => view! {
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="7" height="7" rx="1" />
                <rect x="14" y="3" width="7" height="7" rx="1" />
                <rect x="3" y="14" width="7" height="7" rx="1" />
                <rect x="14" y="14" width="7" height="7" rx="1" />
            </svg>
        }
        .into_any(),
        "feature-flags" => view! {
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 15s1-1 4-1 5 2 8 2 4-1 4-1V3s-1 1-4 1-5-2-8-2-4 1-4 1z" />
                <line x1="4" y1="22" x2="4" y2="15" />
            </svg>
        }
        .into_any(),
        "devtools" => view! {
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z" />
            </svg>
        }
        .into_any(),
        "csr" => view! {
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                <polyline points="9 11 12 14 22 4" />
            </svg>
        }
        .into_any(),
        "selectors" => view! {
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10" />
                <circle cx="12" cy="12" r="6" />
                <circle cx="12" cy="12" r="2" />
            </svg>
        }
        .into_any(),
        _ => view! {
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="12" r="10" />
            </svg>
        }
        .into_any(),
    }
}

/// Main application component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let store = ShowcaseStore::new();
    provide_store(store.clone());
    provide_context(store);

    view! {
        <Stylesheet id="leptos" href="/pkg/showcase.css"/>
        <Style>{GLOBAL_STYLES}</Style>
        <Title text="leptos-store — Examples Showcase"/>
        <Meta name="description" content="Explore all leptos-store examples in one place"/>

        <Router>
            <Routes fallback=|| "Page not found">
                <Route path=path!("/") view=HomePage />
                <Route path=path!("/counter") view=|| view! {
                    <ExampleWrapper title="Counter" id="counter">
                        <demos::CounterDemo />
                    </ExampleWrapper>
                } />
                <Route path=path!("/auth") view=|| view! {
                    <ExampleWrapper title="Auth Store" id="auth-store">
                        <demos::AuthDemo />
                    </ExampleWrapper>
                } />
                <Route path=path!("/token-explorer") view=|| view! {
                    <ExampleWrapper title="Token Explorer" id="token-explorer">
                        <demos::TokenExplorerDemo />
                    </ExampleWrapper>
                } />
                <Route path=path!("/middleware") view=|| view! {
                    <ExampleWrapper title="Middleware" id="middleware">
                        <demos::MiddlewareDemo />
                    </ExampleWrapper>
                } />
                <Route path=path!("/persistence") view=|| view! {
                    <ExampleWrapper title="Persistence" id="persistence">
                        <demos::PersistenceDemo />
                    </ExampleWrapper>
                } />
                <Route path=path!("/composition") view=|| view! {
                    <ExampleWrapper title="Composition" id="composition">
                        <demos::CompositionDemo />
                    </ExampleWrapper>
                } />
                <Route path=path!("/feature-flags") view=|| view! {
                    <ExampleWrapper title="Feature Flags" id="feature-flags">
                        <demos::FeatureFlagsDemo />
                    </ExampleWrapper>
                } />
                <Route path=path!("/devtools") view=|| view! {
                    <ExampleWrapper title="DevTools" id="devtools">
                        <demos::DevtoolsDemo />
                    </ExampleWrapper>
                } />
                <Route path=path!("/csr") view=|| view! {
                    <ExampleWrapper title="CSR Todo" id="csr">
                        <demos::CsrDemo />
                    </ExampleWrapper>
                } />
                <Route path=path!("/selectors") view=|| view! {
                    <ExampleWrapper title="Selectors" id="selectors">
                        <demos::SelectorsDemo />
                    </ExampleWrapper>
                } />
            </Routes>
        </Router>
    }
}

/// Wrapper component for individual example pages.
///
/// Provides a header with back-to-gallery link, title, and CSS scoping
/// via `.demo-{id}` wrapper div.
#[component]
fn ExampleWrapper(title: &'static str, id: &'static str, children: Children) -> impl IntoView {
    let store = use_context::<ShowcaseStore>().expect("ShowcaseStore not found");
    store.mark_visited(id);

    let wrapper_class = format!("demo-{id}");

    view! {
        <div style="min-height: 100vh; display: flex; flex-direction: column; background: var(--bg);">
            <header style="background: rgba(9,9,11,0.8); backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); border-bottom: 1px solid var(--border); height: 56px; display: flex; align-items: center; padding: 0 24px; gap: 16px; position: sticky; top: 0; z-index: 100;">
                <a
                    href="/"
                    style="display: flex; align-items: center; gap: 6px; color: var(--text-secondary); font-size: 13px; transition: color 0.2s;"
                >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M19 12H5" />
                        <path d="M12 19l-7-7 7-7" />
                    </svg>
                    <span>"Gallery"</span>
                </a>
                <span style="color: var(--border);">"|"</span>
                <h1 style="font-size: 14px; font-weight: 600;">{title}</h1>
            </header>
            <main style="flex: 1;">
                <div class=wrapper_class>
                    {children()}
                </div>
            </main>
        </div>
    }
}

/// Home page with all examples
#[component]
fn HomePage() -> impl IntoView {
    let store = use_context::<ShowcaseStore>().expect("ShowcaseStore not found");
    let store2 = store.clone();
    let store3 = store.clone();

    view! {
        <div style="min-height: 100vh; display: flex; flex-direction: column;">
            <Header />

            <main style="flex: 1; max-width: 1200px; margin: 0 auto; padding: 0 32px; width: 100%;">
                // Hero section
                <section style="position: relative; text-align: center; padding: 80px 0 48px;">
                    // Radial indigo glow
                    <div style="position: absolute; top: -100px; left: 50%; transform: translateX(-50%); width: 600px; height: 400px; background: radial-gradient(ellipse, rgba(99,102,241,0.15) 0%, transparent 70%); pointer-events: none;" />

                    // Status pill
                    <div style="position: relative; display: inline-flex; align-items: center; gap: 8px; padding: 6px 16px; border-radius: 20px; border: 1px solid var(--border); font-size: 11px; text-transform: uppercase; letter-spacing: 0.05em; color: var(--text-secondary); margin-bottom: 24px;">
                        <div style="width: 6px; height: 6px; border-radius: 50%; background: #22c55e;" />
                        "10 Interactive Examples"
                    </div>

                    <h1 style="position: relative; font-size: 48px; font-weight: 700; letter-spacing: -0.04em; margin-bottom: 16px; color: var(--text-primary);">
                        "leptos-store"
                    </h1>
                    <p style="position: relative; font-size: 18px; color: var(--text-muted); max-width: 500px; margin: 0 auto;">
                        "Enterprise-grade state management for Leptos"
                    </p>
                </section>

                // Segmented control
                <SegmentedControl />

                // Examples grid
                <section style="margin-top: 24px; margin-bottom: 48px;">
                    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 16px;">
                        {move || {
                            store2.get_filtered_examples()
                                .into_iter()
                                .map(|example| {
                                    let store = store3.clone();
                                    view! { <ExampleCard example=example store=store /> }
                                })
                                .collect_view()
                        }}
                    </div>
                </section>
            </main>

            <Footer />
        </div>
    }
}

/// Header component
#[component]
fn Header() -> impl IntoView {
    view! {
        <header style="background: rgba(9,9,11,0.8); backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); border-bottom: 1px solid var(--border); position: sticky; top: 0; z-index: 100;">
            <div style="max-width: 1200px; margin: 0 auto; padding: 0 32px; height: 56px; display: flex; justify-content: space-between; align-items: center;">
                <div style="display: flex; align-items: center; gap: 10px;">
                    <span style="font-size: 15px; font-weight: 600; color: var(--text-primary);">"leptos-store"</span>
                    <span style="font-size: 11px; color: var(--text-muted); background: rgba(255,255,255,0.04); padding: 2px 8px; border-radius: 6px;">
                        {format!("v{}", env!("CARGO_PKG_VERSION"))}
                    </span>
                </div>

                <nav style="display: flex; align-items: center; gap: 20px;">
                    <a
                        href="https://github.com/nyvorin/leptos-store"
                        target="_blank"
                        style="display: flex; align-items: center; color: var(--text-secondary); transition: color 0.2s;"
                    >
                        <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                        </svg>
                    </a>
                    <a
                        href="https://docs.rs/leptos-store"
                        target="_blank"
                        style="font-size: 13px; color: var(--text-secondary); transition: color 0.2s;"
                    >
                        "Docs"
                    </a>
                </nav>
            </div>
        </header>
    }
}

/// Segmented control for category filtering (replaces FilterBar)
#[component]
fn SegmentedControl() -> impl IntoView {
    let store = use_context::<ShowcaseStore>().expect("ShowcaseStore not found");
    let active_category = RwSignal::new(Option::<String>::None);

    let categories = [
        ExampleCategory::Core,
        ExampleCategory::State,
        ExampleCategory::Advanced,
        ExampleCategory::Integration,
    ];

    view! {
        <div style="display: flex; justify-content: center; margin-bottom: 8px;">
            <div style="display: inline-flex; background: rgba(255,255,255,0.04); border-radius: 10px; padding: 3px;">
                {
                    let store_all = store.clone();
                    view! {
                        <button
                            style=move || format!(
                                "padding: 8px 18px; border-radius: 8px; font-size: 13px; font-weight: 500; cursor: pointer; border: none; transition: all 0.2s ease; {}",
                                if active_category.get().is_none() {
                                    "background: rgba(255,255,255,0.08); color: #fff;"
                                } else {
                                    "background: transparent; color: rgba(255,255,255,0.4);"
                                }
                            )
                            on:click=move |_| {
                                active_category.set(None);
                                store_all.set_category_filter(None);
                            }
                        >
                            "All"
                        </button>
                    }
                }

                {categories.into_iter().map(|cat| {
                    let store = store.clone();
                    let label = cat.label();
                    view! {
                        <button
                            style=move || format!(
                                "padding: 8px 18px; border-radius: 8px; font-size: 13px; font-weight: 500; cursor: pointer; border: none; transition: all 0.2s ease; {}",
                                if active_category.get().as_deref() == Some(label) {
                                    "background: rgba(255,255,255,0.08); color: #fff;"
                                } else {
                                    "background: transparent; color: rgba(255,255,255,0.4);"
                                }
                            )
                            on:click=move |_| {
                                active_category.set(Some(label.to_string()));
                                store.set_category_filter(Some(label.to_string()));
                            }
                        >
                            {label}
                        </button>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

/// Example card component
#[component]
fn ExampleCard(example: &'static ExampleInfo, store: ShowcaseStore) -> impl IntoView {
    let is_hovered = RwSignal::new(false);
    let navigate = use_navigate();
    let route = example.route;
    let example_id = example.id.to_string();
    let color = example.category.color();

    view! {
        <article
            style=move || format!(
                "background: var(--bg-card); border: 1px solid {}; border-radius: 16px; padding: 24px; cursor: pointer; transition: all 0.2s ease; animation: fadeIn 0.4s ease-out; {}",
                if is_hovered.get() { "var(--border-hover)" } else { "var(--border)" },
                if is_hovered.get() { "transform: translateY(-2px);" } else { "" }
            )
            on:mouseenter=move |_| is_hovered.set(true)
            on:mouseleave=move |_| is_hovered.set(false)
            on:click={
                let store = store.clone();
                let id = example_id.clone();
                let navigate = navigate.clone();
                move |_| {
                    store.mark_visited(&id);
                    navigate(route, Default::default());
                }
            }
        >
            // Top row: icon + category pill
            <div style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 16px;">
                <div style=format!(
                    "width: 40px; height: 40px; border-radius: 12px; display: flex; align-items: center; justify-content: center; background: linear-gradient(135deg, {}15, {}08);",
                    color, color
                )>
                    {render_icon(example.id, color)}
                </div>
                <span style=format!(
                    "padding: 4px 10px; border-radius: 10px; font-size: 11px; font-weight: 500; letter-spacing: 0.02em; background: {}12; color: {};",
                    color, color
                )>
                    {example.category.label()}
                </span>
            </div>

            // Title
            <h3 style="font-size: 15px; font-weight: 600; letter-spacing: -0.01em; margin-bottom: 6px; color: var(--text-primary);">
                {example.name}
            </h3>

            // Description
            <p style="font-size: 13px; color: var(--text-secondary); line-height: 1.55; margin-bottom: 16px;">
                {example.description}
            </p>

            // Feature tags
            <div style="display: flex; flex-wrap: wrap; gap: 6px; margin-bottom: 16px;">
                {example.features.iter().map(|feature| {
                    view! {
                        <span style="padding: 3px 8px; background: rgba(255,255,255,0.04); border-radius: 5px; font-size: 11px; color: var(--text-muted); font-family: 'JetBrains Mono', ui-monospace, monospace;">
                            {*feature}
                        </span>
                    }
                }).collect_view()}
            </div>

            // Bottom divider + launch + difficulty
            <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 14px; border-top: 1px solid var(--border);">
                <div style=format!(
                    "display: flex; align-items: center; gap: 4px; font-size: 12px; font-weight: 500; color: {};",
                    color
                )>
                    <span>"Launch"</span>
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke=color stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                        <path d="M5 12h14" />
                        <path d="M12 5l7 7-7 7" />
                    </svg>
                </div>
                <DifficultyBadge difficulty=example.difficulty />
            </div>
        </article>
    }
}

/// Difficulty badge component with dot indicators
#[component]
fn DifficultyBadge(difficulty: Difficulty) -> impl IntoView {
    let dots = match difficulty {
        Difficulty::Beginner => 1,
        Difficulty::Intermediate => 2,
        Difficulty::Advanced => 3,
    };

    view! {
        <div style="display: flex; align-items: center; gap: 6px;">
            <div style="display: flex; gap: 3px;">
                {(0..3).map(|i| {
                    let active = i < dots;
                    view! {
                        <div style=format!(
                            "width: 5px; height: 5px; border-radius: 50%; {}",
                            if active {
                                format!("background: {};", difficulty.color())
                            } else {
                                "background: rgba(255,255,255,0.1);".to_string()
                            }
                        ) />
                    }
                }).collect_view()}
            </div>
            <span style="font-size: 10px; color: var(--text-muted);">
                {difficulty.label()}
            </span>
        </div>
    }
}

/// Footer component
#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer style="border-top: 1px solid var(--border); padding: 32px; margin-top: auto;">
            <div style="max-width: 1200px; margin: 0 auto; display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 16px;">
                <p style="font-size: 12px; color: var(--text-muted);">
                    "\u{00A9} 2026 nyvorin. Apache-2.0 License."
                </p>
                <div style="display: flex; gap: 20px;">
                    <a href="https://leptos.dev" target="_blank" style="font-size: 12px; color: var(--text-muted); transition: color 0.2s;">
                        "Leptos"
                    </a>
                    <a href="https://crates.io/crates/leptos-store" target="_blank" style="font-size: 12px; color: var(--text-muted); transition: color 0.2s;">
                        "crates.io"
                    </a>
                    <a href="https://github.com/nyvorin/leptos-store/issues" target="_blank" style="font-size: 12px; color: var(--text-muted); transition: color 0.2s;">
                        "Issues"
                    </a>
                </div>
            </div>
        </footer>
    }
}
