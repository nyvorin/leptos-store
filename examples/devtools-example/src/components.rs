// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! UI Components for the Devtools Example

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use leptos_store::prelude::*;

#[cfg(feature = "hydrate")]
use leptos_store::devtools::{DevtoolsConfig, StoreInspector, init_devtools, register_store_json};

use crate::counter_store::CounterStore;

/// Main app component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Initialize devtools (client-side only)
    #[cfg(feature = "hydrate")]
    {
        init_devtools(DevtoolsConfig::default());
    }

    // Create and provide the counter store
    let store = CounterStore::new();

    // Register with devtools (client-side only)
    #[cfg(feature = "hydrate")]
    {
        register_store_json(&store, "counter");
    }

    provide_store(store);

    view! {
        <Stylesheet id="leptos" href="/pkg/devtools-example.css"/>
        <Title text="Devtools Example - leptos-store"/>
        <Meta name="description" content="Devtools demonstration using leptos-store"/>

        <Router>
            <main class="app">
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=DevtoolsPage/>
                </Routes>
            </main>
        </Router>

        <DevtoolsInspector />
    }
}

/// Devtools inspector wrapper (only renders in hydrate mode)
#[component]
fn DevtoolsInspector() -> impl IntoView {
    // Use a signal that flips to true after hydration to avoid SSR/client mismatch
    let is_mounted = RwSignal::new(false);

    // Set mounted to true after initial render (client-side only)
    Effect::new(move |_| {
        is_mounted.set(true);
    });

    view! {
        <Show when=move || is_mounted.get()>
            {
                #[cfg(feature = "hydrate")]
                { view! { <StoreInspector /> } }
                #[cfg(not(feature = "hydrate"))]
                { view! { <div></div> } }
            }
        </Show>
    }
}

/// Devtools demonstration page
#[component]
fn DevtoolsPage() -> impl IntoView {
    view! {
        <div class="devtools-page">
            <header class="devtools-header">
                <h1>"Devtools Example"</h1>
                <p class="subtitle">"Store inspection and debugging tools"</p>
            </header>

            <div class="devtools-layout">
                <CounterDemo />
                <ConsoleApiGuide />
            </div>

            <div class="code-hint">
                <p>"Initialize devtools:"</p>
                <pre><code>{r#"// In your app setup
#[cfg(feature = "devtools")]
init_devtools();

// Add the inspector component
<StoreInspector />

// Console API available at:
// window.__LEPTOS_STORE__.getStores()
// window.__LEPTOS_STORE__.getState("counter")
// window.__LEPTOS_STORE__.getEvents()"#}</code></pre>
            </div>
        </div>
    }
}

/// Counter demonstration
#[component]
fn CounterDemo() -> impl IntoView {
    let store = use_store::<CounterStore>();

    let store_count = store.clone();
    let store_doubled = store.clone();
    let store_history_len = store.clone();
    let store_history_values = store.clone();
    let store_inc = store.clone();
    let store_dec = store.clone();
    let store_reset = store.clone();
    let store_set = store.clone();
    let store_clear = store.clone();

    view! {
        <div class="panel counter-demo">
            <h2>"Counter Store"</h2>
            <p class="panel-desc">"Interact with the store to see devtools in action"</p>

            <div class="counter-display">
                <span class="count-value">{move || store_count.count()}</span>
                <span class="count-doubled">"(doubled: " {move || store_doubled.doubled()} ")"</span>
            </div>

            <div class="button-group">
                <button class="btn btn-dec" on:click=move |_| store_dec.decrement()>
                    "−"
                </button>
                <button class="btn btn-reset" on:click=move |_| store_reset.reset()>
                    "Reset"
                </button>
                <button class="btn btn-inc" on:click=move |_| store_inc.increment()>
                    "+"
                </button>
            </div>

            <div class="quick-set">
                <span>"Quick set:"</span>
                {[10, 50, 100].into_iter().map(|v| {
                    let store_s = store_set.clone();
                    view! {
                        <button
                            class="btn btn-small"
                            on:click=move |_| store_s.set(v)
                        >
                            {v}
                        </button>
                    }
                }).collect_view()}
            </div>

            <div class="history-panel">
                <h3>"History (" {move || store_history_len.history_len()} " entries)"</h3>
                <div class="history-values">
                    {move || {
                        store_history_values.history()
                            .into_iter()
                            .rev()
                            .take(10)
                            .map(|v| view! {
                                <span class="history-value">{v}</span>
                            })
                            .collect_view()
                    }}
                </div>
                <button
                    class="btn btn-clear"
                    on:click=move |_| store_clear.clear_history()
                >
                    "Clear History"
                </button>
            </div>
        </div>
    }
}

/// Console API guide
#[component]
fn ConsoleApiGuide() -> impl IntoView {
    view! {
        <div class="panel console-guide">
            <h2>"Console API"</h2>
            <p class="panel-desc">"Open browser DevTools console and try these commands"</p>

            <div class="api-commands">
                <div class="api-command">
                    <code>"__LEPTOS_STORE__.help()"</code>
                    <span class="api-desc">"Show all available commands"</span>
                </div>

                <div class="api-command">
                    <code>"__LEPTOS_STORE__.getStores()"</code>
                    <span class="api-desc">"List all registered stores"</span>
                </div>

                <div class="api-command">
                    <code>"__LEPTOS_STORE__.getState('counter')"</code>
                    <span class="api-desc">"Get state of a specific store"</span>
                </div>

                <div class="api-command">
                    <code>"__LEPTOS_STORE__.getEvents()"</code>
                    <span class="api-desc">"View event history"</span>
                </div>
            </div>

            <div class="inspector-hint">
                <h3>"Store Inspector"</h3>
                <p>"Look for the floating panel in the bottom-right corner!"</p>
                <ul>
                    <li>"Click to expand/collapse"</li>
                    <li>"View registered stores"</li>
                    <li>"Monitor event history"</li>
                </ul>
            </div>
        </div>
    }
}
