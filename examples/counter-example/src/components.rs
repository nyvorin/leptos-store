// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! UI Components for the Counter Example
//!
//! This module provides Leptos components that demonstrate
//! how to use a CounterStore with leptos-store.

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use leptos_store::prelude::*;

use crate::counter_store::CounterStore;

/// Main app component
#[component]
pub fn App() -> impl IntoView {
    // Provides context for <Title> and <Meta> components
    provide_meta_context();

    // Create and provide the counter store
    let store = CounterStore::new();
    provide_store(store);

    view! {
        <Stylesheet id="leptos" href="/pkg/counter-example.css"/>
        <Title text="Counter Example - leptos-store"/>
        <Meta name="description" content="Simple counter example using leptos-store"/>

        <Router>
            <main class="app">
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=CounterPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Counter page component
#[component]
fn CounterPage() -> impl IntoView {
    view! {
        <div class="counter-page">
            <div class="counter-card">
                <h1>"Counter Store Example"</h1>
                <p class="subtitle">"Using the " <code>"store!"</code> " macro"</p>

                <Counter />

                <div class="code-hint">
                    <p>"Using the " <code>"store!"</code> " macro (Enterprise Mode):"</p>
                    <pre><code>{r#"store! {
    pub CounterStore {
        state CounterState { count: i32 = 0 }

        getters {
            doubled(this) -> i32 { this.read(|s| s.count * 2) }
        }

        mutators {  // PRIVATE - internal only
            add_to_count(this, n: i32) { this.mutate(|s| s.count += n); }
        }

        actions {   // PUBLIC - external API
            increment(this) { this.add_to_count(1); }
        }
    }
}"#}</code></pre>
                </div>
            </div>
        </div>
    }
}

/// Counter display and controls
#[component]
fn Counter() -> impl IntoView {
    let store = use_store::<CounterStore>();

    // Clone store for each closure
    let store_count = store.clone();
    let store_doubled = store.clone();
    let store_status = store.clone();
    let store_inc = store.clone();
    let store_dec = store.clone();
    let store_reset = store.clone();
    let store_prime = store.clone();

    view! {
        <div class="counter">
            // Current count display
            <div class="count-display">
                <span class="count-value">
                    {move || store_count.state().get().count}
                </span>
            </div>

            // Button controls
            <div class="button-group">
                <button
                    class="btn btn-decrement"
                    on:click=move |_| store_dec.decrement()
                    aria-label="Decrement"
                >
                    "−"
                </button>

                <button
                    class="btn btn-reset"
                    on:click=move |_| store_reset.reset()
                    aria-label="Reset"
                >
                    "Reset"
                </button>

                <button
                    class="btn btn-increment"
                    on:click=move |_| store_inc.increment()
                    aria-label="Increment"
                >
                    "+"
                </button>
            </div>

            // Derived values (getters)
            <div class="info-panel">
                <div class="info-item">
                    <span class="info-label">"Doubled:"</span>
                    <span class="info-value">{move || store_doubled.doubled()}</span>
                </div>
                <div class="info-item">
                    <span class="info-label">"Prime:"</span>
                    <span class="info-value">{move || store_prime.is_prime()}</span>
                </div>
                <div class="info-item">
                    <span class="info-label">"Status:"</span>
                    <span class="info-value">
                        {move || {
                            if store_status.is_positive() {
                                "Positive ↑"
                            } else if store_status.is_negative() {
                                "Negative ↓"
                            } else {
                                "Zero"
                            }
                        }}
                    </span>
                </div>
            </div>
        </div>
    }
}
