// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Showcase components for displaying all leptos-store examples.

use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;
use leptos_store::provide_store;

use crate::showcase_store::{
    ShowcaseStore, ExampleInfo, ExampleCategory, Difficulty, EXAMPLES,
};

/// Global styles
const GLOBAL_STYLES: &str = r#"
    * {
        margin: 0;
        padding: 0;
        box-sizing: border-box;
    }
    
    :root {
        --bg-primary: #0f172a;
        --bg-secondary: #1e293b;
        --bg-tertiary: #334155;
        --text-primary: #f8fafc;
        --text-secondary: #94a3b8;
        --text-muted: #64748b;
        --accent: #3b82f6;
        --accent-hover: #2563eb;
        --border: #334155;
        --success: #22c55e;
        --warning: #eab308;
        --error: #ef4444;
    }
    
    body {
        font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        background: var(--bg-primary);
        color: var(--text-primary);
        line-height: 1.6;
        min-height: 100vh;
    }
    
    a {
        color: inherit;
        text-decoration: none;
    }
    
    button {
        font-family: inherit;
    }
    
    ::selection {
        background: var(--accent);
        color: white;
    }
    
    ::-webkit-scrollbar {
        width: 8px;
        height: 8px;
    }
    
    ::-webkit-scrollbar-track {
        background: var(--bg-secondary);
    }
    
    ::-webkit-scrollbar-thumb {
        background: var(--bg-tertiary);
        border-radius: 4px;
    }
    
    ::-webkit-scrollbar-thumb:hover {
        background: var(--text-muted);
    }
    
    @keyframes fadeIn {
        from { opacity: 0; transform: translateY(10px); }
        to { opacity: 1; transform: translateY(0); }
    }
    
    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.5; }
    }
    
    @keyframes shimmer {
        0% { background-position: -200% 0; }
        100% { background-position: 200% 0; }
    }
"#;

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
        <Title text="Leptos Store - Examples Showcase"/>
        <Meta name="description" content="Explore all leptos-store examples in one place"/>
        
        <Router>
            <Routes fallback=|| "Page not found">
                <Route path=path!("/") view=HomePage />
            </Routes>
        </Router>
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
            
            <main style="flex: 1; max-width: 1400px; margin: 0 auto; padding: 40px 24px; width: 100%;">
                // Hero section
                <section style="text-align: center; margin-bottom: 60px;">
                    <h1 style="font-size: 3.5rem; font-weight: 700; margin-bottom: 16px; background: linear-gradient(135deg, #60a5fa 0%, #a78bfa 50%, #f472b6 100%); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;">
                        "Leptos Store Examples"
                    </h1>
                    <p style="font-size: 1.25rem; color: var(--text-secondary); max-width: 600px; margin: 0 auto 32px;">
                        "Explore state management patterns, from basic counters to advanced middleware and devtools integration."
                    </p>
                    
                    // Stats
                    <div style="display: flex; justify-content: center; gap: 48px; margin-bottom: 32px;">
                        <StatItem 
                            value=move || EXAMPLES.len().to_string()
                            label="Examples"
                            icon="📚"
                        />
                        <StatItem 
                            value=move || store.visit_count().to_string()
                            label="Visited"
                            icon="✅"
                        />
                        <StatItem 
                            value=move || "4".to_string()
                            label="Categories"
                            icon="📂"
                        />
                    </div>
                </section>
                
                // Filters
                <FilterBar />
                
                // Examples grid
                <section style="margin-top: 32px;">
                    <div style="display: grid; grid-template-columns: repeat(auto-fill, minmax(340px, 1fr)); gap: 24px;">
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
        <header style="background: var(--bg-secondary); border-bottom: 1px solid var(--border); position: sticky; top: 0; z-index: 100; backdrop-filter: blur(8px);">
            <div style="max-width: 1400px; margin: 0 auto; padding: 16px 24px; display: flex; justify-content: space-between; align-items: center;">
                <div style="display: flex; align-items: center; gap: 12px;">
                    <span style="font-size: 28px;">"🏪"</span>
                    <div>
                        <h1 style="font-size: 1.25rem; font-weight: 600; line-height: 1.2;">"leptos-store"</h1>
                        <span style="font-size: 0.75rem; color: var(--text-muted);">"v0.2.0"</span>
                    </div>
                </div>
                
                <nav style="display: flex; align-items: center; gap: 24px;">
                    <a 
                        href="https://github.com/webmech/leptos-store" 
                        target="_blank"
                        style="display: flex; align-items: center; gap: 8px; color: var(--text-secondary); transition: color 0.2s;"
                    >
                        <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                            <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                        </svg>
                        "GitHub"
                    </a>
                    <a 
                        href="https://docs.rs/leptos-store" 
                        target="_blank"
                        style="display: flex; align-items: center; gap: 8px; color: var(--text-secondary); transition: color 0.2s;"
                    >
                        <span>"📖"</span>
                        "Docs"
                    </a>
                </nav>
            </div>
        </header>
    }
}

/// Stat item component
#[component]
fn StatItem<F>(
    value: F,
    label: &'static str,
    icon: &'static str,
) -> impl IntoView 
where
    F: Fn() -> String + Send + Sync + 'static,
{
    view! {
        <div style="text-align: center;">
            <div style="font-size: 2rem; margin-bottom: 4px;">{icon}</div>
            <div style="font-size: 2rem; font-weight: 700; color: var(--accent);">{value}</div>
            <div style="font-size: 0.875rem; color: var(--text-muted);">{label}</div>
        </div>
    }
}

/// Filter bar component
#[component]
fn FilterBar() -> impl IntoView {
    let store = use_context::<ShowcaseStore>().expect("ShowcaseStore not found");
    let store2 = store.clone();
    
    let search_query = RwSignal::new(String::new());
    let active_category = RwSignal::new(Option::<String>::None);
    
    let categories = [
        ExampleCategory::Core,
        ExampleCategory::State,
        ExampleCategory::Advanced,
        ExampleCategory::Integration,
    ];
    
    view! {
        <div style="display: flex; flex-wrap: wrap; gap: 16px; align-items: center; padding: 20px 24px; background: var(--bg-secondary); border-radius: 12px; border: 1px solid var(--border);">
            // Search input
            <div style="flex: 1; min-width: 250px; position: relative;">
                <span style="position: absolute; left: 14px; top: 50%; transform: translateY(-50%); color: var(--text-muted);">
                    "🔍"
                </span>
                <input
                    type="text"
                    placeholder="Search examples..."
                    style="width: 100%; padding: 12px 12px 12px 44px; background: var(--bg-primary); border: 1px solid var(--border); border-radius: 8px; color: var(--text-primary); font-size: 0.95rem; outline: none; transition: border-color 0.2s;"
                    prop:value=move || search_query.get()
                    on:input=move |ev| {
                        let value = event_target_value(&ev);
                        search_query.set(value.clone());
                        store.set_search_query(value);
                    }
                />
            </div>
            
            // Category filters
            <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                {
                    let store_all = store2.clone();
                    view! {
                        <button
                            style=move || format!(
                                "padding: 8px 16px; border-radius: 20px; font-size: 0.875rem; font-weight: 500; cursor: pointer; transition: all 0.2s; border: 1px solid {}; background: {}; color: {};",
                                if active_category.get().is_none() { "var(--accent)" } else { "var(--border)" },
                                if active_category.get().is_none() { "var(--accent)" } else { "transparent" },
                                if active_category.get().is_none() { "white" } else { "var(--text-secondary)" }
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
                    let store = store2.clone();
                    let label = cat.label();
                    let color = cat.color();
                    view! {
                        <button
                            style=move || format!(
                                "padding: 8px 16px; border-radius: 20px; font-size: 0.875rem; font-weight: 500; cursor: pointer; transition: all 0.2s; border: 1px solid {}; background: {}; color: {};",
                                if active_category.get().as_deref() == Some(label) { color } else { "var(--border)" },
                                if active_category.get().as_deref() == Some(label) { color } else { "transparent" },
                                if active_category.get().as_deref() == Some(label) { "white" } else { "var(--text-secondary)" }
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
    let url = format!("http://127.0.0.1:{}", example.port);
    let url2 = url.clone();
    let example_id = example.id.to_string();
    
    let is_visited = {
        let store = store.clone();
        let id = example.id.to_string();
        move || store.is_visited(&id)
    };
    
    view! {
        <article
            style=move || format!(
                "position: relative; background: var(--bg-secondary); border-radius: 16px; padding: 24px; border: 1px solid {}; transition: all 0.3s ease; cursor: pointer; animation: fadeIn 0.4s ease-out; {}",
                if is_hovered.get() { "var(--accent)" } else { "var(--border)" },
                if is_hovered.get() { "transform: translateY(-4px); box-shadow: 0 20px 40px -15px rgba(59, 130, 246, 0.2);" } else { "" }
            )
            on:mouseenter=move |_| is_hovered.set(true)
            on:mouseleave=move |_| is_hovered.set(false)
            on:click={
                let store = store.clone();
                let id = example_id.clone();
                let url = url2.clone();
                move |_| {
                    store.mark_visited(&id);
                    // Open in new tab (only works in WASM)
                    #[cfg(target_arch = "wasm32")]
                    let _ = web_sys::window()
                        .and_then(|w| w.open_with_url_and_target(&url, "_blank").ok());
                    #[cfg(not(target_arch = "wasm32"))]
                    let _ = &url; // Silence unused variable warning on SSR
                }
            }
        >
            // Category badge
            <div style=format!(
                "position: absolute; top: 16px; right: 16px; padding: 4px 10px; border-radius: 12px; font-size: 0.7rem; font-weight: 600; text-transform: uppercase; letter-spacing: 0.5px; background: {}20; color: {};",
                example.category.color(), example.category.color()
            )>
                {example.category.label()}
            </div>
            
            // Visited indicator
            <Show when=is_visited>
                <div style="position: absolute; top: 16px; left: 16px; width: 8px; height: 8px; background: var(--success); border-radius: 50%;" title="Visited" />
            </Show>
            
            // Icon and title
            <div style="display: flex; align-items: center; gap: 16px; margin-bottom: 16px;">
                <div style="width: 56px; height: 56px; background: var(--bg-tertiary); border-radius: 14px; display: flex; align-items: center; justify-content: center; font-size: 28px;">
                    {example.icon}
                </div>
                <div>
                    <h3 style="font-size: 1.25rem; font-weight: 600; margin-bottom: 4px;">{example.name}</h3>
                    <DifficultyBadge difficulty=example.difficulty />
                </div>
            </div>
            
            // Description
            <p style="color: var(--text-secondary); font-size: 0.95rem; line-height: 1.6; margin-bottom: 20px;">
                {example.description}
            </p>
            
            // Features
            <div style="display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 20px;">
                {example.features.iter().map(|feature| {
                    view! {
                        <span style="padding: 4px 10px; background: var(--bg-primary); border-radius: 6px; font-size: 0.8rem; color: var(--text-muted); font-family: 'JetBrains Mono', monospace;">
                            {*feature}
                        </span>
                    }
                }).collect_view()}
            </div>
            
            // Port info and launch button
            <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 16px; border-top: 1px solid var(--border);">
                <span style="font-family: 'JetBrains Mono', monospace; font-size: 0.8rem; color: var(--text-muted);">
                    {format!("localhost:{}", example.port)}
                </span>
                <div style=move || format!(
                    "display: flex; align-items: center; gap: 8px; padding: 8px 16px; background: {}; border-radius: 8px; font-size: 0.875rem; font-weight: 500; transition: background 0.2s;",
                    if is_hovered.get() { "var(--accent)" } else { "var(--bg-tertiary)" }
                )>
                    <span>"Launch"</span>
                    <span style="font-size: 1rem;">"→"</span>
                </div>
            </div>
        </article>
    }
}

/// Difficulty badge component
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
                            "width: 6px; height: 6px; border-radius: 50%; {}",
                            if active { format!("background: {};", difficulty.color()) } else { "background: var(--bg-tertiary);".to_string() }
                        ) />
                    }
                }).collect_view()}
            </div>
            <span style=format!("font-size: 0.75rem; color: {};", difficulty.color())>
                {difficulty.label()}
            </span>
        </div>
    }
}

/// Footer component
#[component]
fn Footer() -> impl IntoView {
    view! {
        <footer style="background: var(--bg-secondary); border-top: 1px solid var(--border); padding: 32px 24px; margin-top: auto;">
            <div style="max-width: 1400px; margin: 0 auto; display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: 24px;">
                <div>
                    <p style="color: var(--text-secondary); font-size: 0.9rem;">
                        "Built with "
                        <span style="color: var(--accent);">"Leptos"</span>
                        " + "
                        <span style="color: var(--accent);">"leptos-store"</span>
                    </p>
                    <p style="color: var(--text-muted); font-size: 0.8rem; margin-top: 4px;">
                        "© 2026 web-mech. Apache-2.0 License."
                    </p>
                </div>
                
                <div style="display: flex; gap: 24px;">
                    <a href="https://leptos.dev" target="_blank" style="color: var(--text-muted); font-size: 0.875rem; transition: color 0.2s;">
                        "Leptos"
                    </a>
                    <a href="https://crates.io/crates/leptos-store" target="_blank" style="color: var(--text-muted); font-size: 0.875rem; transition: color 0.2s;">
                        "Crates.io"
                    </a>
                    <a href="https://github.com/webmech/leptos-store/issues" target="_blank" style="color: var(--text-muted); font-size: 0.875rem; transition: color 0.2s;">
                        "Report Issue"
                    </a>
                </div>
            </div>
        </footer>
    }
}
