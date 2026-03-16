// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! UI Components for the Middleware Example
//!
//! This module provides Leptos components that demonstrate
//! how middleware observes store operations.

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use leptos_store::prelude::*;

#[cfg(any(feature = "hydrate", feature = "ssr"))]
use leptos_store::middleware::{LoggingMiddleware, MiddlewareStore, TimingMiddleware};

use crate::task_store::{TaskFilter, TaskStore};

/// Embeddable demo component for the showcase.
///
/// Wraps the TaskStore in MiddlewareStore with logging/timing middleware.
#[component]
pub fn Demo() -> impl IntoView {
    let store = TaskStore::new();

    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    {
        let middleware_store = MiddlewareStore::new(store.clone());
        middleware_store.add_middleware(LoggingMiddleware::new());
        middleware_store.add_middleware(TimingMiddleware::new());
        provide_context(middleware_store);
    }

    provide_store(store);
    view! { <TaskPage /> }
}

/// Main app component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Create the base store
    let store = TaskStore::new();

    // Create middleware-wrapped version for operations that should be logged
    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    {
        let middleware_store = MiddlewareStore::new(store.clone());

        // Add logging middleware - logs all mutations to console
        middleware_store.add_middleware(LoggingMiddleware::new());

        // Add timing middleware - warns on slow operations
        middleware_store.add_middleware(TimingMiddleware::new());

        provide_context(middleware_store);
    }

    provide_store(store);

    view! {
        <Stylesheet id="leptos" href="/pkg/middleware-example.css"/>
        <Title text="Middleware Example - leptos-store"/>
        <Meta name="description" content="Middleware demonstration using leptos-store"/>

        <Router>
            <main class="app">
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=TaskPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Task page component
#[component]
fn TaskPage() -> impl IntoView {
    view! {
        <div class="task-page">
            <div class="task-card">
                <h1>"Middleware Example"</h1>
                <p class="subtitle">"Open browser console (F12) to see middleware logging"</p>

                <MiddlewareDemo />

                <hr style="margin: 2rem 0; border-color: rgba(255,255,255,0.1);" />

                <TaskInput />
                <TaskList />
                <TaskFooter />

                <div class="code-hint">
                    <p>"How middleware works:"</p>
                    <pre><code>{r#"// 1. Wrap store in MiddlewareStore
let mw_store = MiddlewareStore::new(store);

// 2. Add middleware
mw_store.add_middleware(LoggingMiddleware::new());

// 3. Use mutate() to run operations through middleware
mw_store.mutate("add_task", || {
    store.add_task(title);
});"#}</code></pre>
                </div>
            </div>
        </div>
    }
}

/// Demo panel showing middleware in action
#[component]
fn MiddlewareDemo() -> impl IntoView {
    let store = use_store::<TaskStore>();

    #[cfg(any(feature = "hydrate", feature = "ssr"))]
    let middleware_store = use_context::<MiddlewareStore<TaskStore>>();

    // Counter for demo operations
    let counter = RwSignal::new(0);
    let log_messages = RwSignal::new(Vec::<String>::new());

    let on_middleware_demo = move |_| {
        counter.update(|c| *c += 1);
        let count = counter.get();

        #[cfg(any(feature = "hydrate", feature = "ssr"))]
        if let Some(ref mw) = middleware_store {
            let store_clone = store.clone();
            let title = format!("Task #{}", count);
            let log_messages_clone = log_messages;

            // Add a message before we call mutate
            log_messages_clone.update(|msgs| {
                msgs.push(format!(">>> Calling mutate('demo_add_task')..."));
            });

            // This will trigger the middleware chain!
            let result = mw.mutate("demo_add_task", move || {
                store_clone.add_task(title);
            });

            // Log the result
            log_messages_clone.update(|msgs| {
                match result {
                    Ok(()) => msgs.push("<<< Middleware completed successfully".to_string()),
                    Err(e) => msgs.push(format!("<<< Middleware error: {:?}", e)),
                }
                // Keep only last 5 messages
                if msgs.len() > 5 {
                    msgs.remove(0);
                }
            });
        }

        #[cfg(not(any(feature = "hydrate", feature = "ssr")))]
        {
            let title = format!("Task #{}", count);
            store.add_task(title);
        }
    };

    view! {
        <div class="middleware-demo">
            <h3>"Middleware Demo"</h3>
            <p class="demo-desc">
                "Click the button below. The operation goes through middleware which logs to the browser console."
            </p>

            <button class="btn btn-demo" on:click=on_middleware_demo>
                "Add Task via Middleware"
            </button>

            <div class="log-output">
                <p class="log-label">"Activity Log:"</p>
                <div class="log-messages">
                    {move || {
                        log_messages.get().into_iter().map(|msg| {
                            view! { <div class="log-msg">{msg}</div> }
                        }).collect_view()
                    }}
                </div>
            </div>

            <p class="demo-hint">
                "Check browser console (F12) for detailed middleware logs from LoggingMiddleware"
            </p>
        </div>
    }
}

/// Task input component
#[component]
fn TaskInput() -> impl IntoView {
    let store = use_store::<TaskStore>();
    let input_ref = NodeRef::<leptos::html::Input>::new();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if let Some(input) = input_ref.get() {
            let value = input.value();
            if !value.trim().is_empty() {
                // Direct store call (no middleware)
                store.add_task(value);
                input.set_value("");
            }
        }
    };

    view! {
        <form class="task-input" on:submit=on_submit>
            <input
                node_ref=input_ref
                type="text"
                placeholder="What needs to be done? (direct, no middleware)"
                class="task-input-field"
            />
            <button type="submit" class="btn btn-add">"Add"</button>
        </form>
    }
}

/// Task list component
#[component]
fn TaskList() -> impl IntoView {
    let store = use_store::<TaskStore>();
    let store_for_list = store.clone();

    view! {
        <ul class="task-list">
            {move || {
                store_for_list
                    .filtered_tasks()
                    .into_iter()
                    .map(|task| {
                        let store_toggle = store.clone();
                        let store_remove = store.clone();
                        let task_id = task.id;
                        let item_class = if task.completed { "task-item completed" } else { "task-item" };

                        view! {
                            <li class=item_class>
                                <label class="task-checkbox">
                                    <input
                                        type="checkbox"
                                        checked=task.completed
                                        on:change=move |_| store_toggle.toggle_task(task_id)
                                    />
                                    <span class="checkmark"></span>
                                </label>
                                <span class="task-title">{task.title.clone()}</span>
                                <button
                                    class="btn-remove"
                                    on:click=move |_| store_remove.remove_task(task_id)
                                >
                                    "×"
                                </button>
                            </li>
                        }
                    })
                    .collect_view()
            }}
        </ul>
    }
}

/// Task footer with filters and stats
#[component]
fn TaskFooter() -> impl IntoView {
    let store = use_store::<TaskStore>();
    let store_active = store.clone();
    let store_completed = store.clone();
    let _store_filter = store.clone();
    let store_all = store.clone();
    let store_active_filter = store.clone();
    let store_completed_filter = store.clone();
    let store_clear = store.clone();

    let filter_all = store.clone();
    let filter_active = store.clone();
    let filter_completed_check = store.clone();

    view! {
        <div class="task-footer">
            <div class="task-stats">
                <span class="stat">
                    <strong>{move || store_active.active_count()}</strong>
                    " active"
                </span>
                <span class="stat">
                    <strong>{move || store_completed.completed_count()}</strong>
                    " completed"
                </span>
            </div>

            <div class="task-filters">
                <button
                    class={move || if filter_all.current_filter() == TaskFilter::All { "filter-btn active" } else { "filter-btn" }}
                    on:click=move |_| store_all.set_filter(TaskFilter::All)
                >
                    "All"
                </button>
                <button
                    class={move || if filter_active.current_filter() == TaskFilter::Active { "filter-btn active" } else { "filter-btn" }}
                    on:click=move |_| store_active_filter.set_filter(TaskFilter::Active)
                >
                    "Active"
                </button>
                <button
                    class={move || if filter_completed_check.current_filter() == TaskFilter::Completed { "filter-btn active" } else { "filter-btn" }}
                    on:click=move |_| store_completed_filter.set_filter(TaskFilter::Completed)
                >
                    "Completed"
                </button>
            </div>

            <button
                class="btn btn-clear"
                on:click=move |_| store_clear.clear_completed()
            >
                "Clear Completed"
            </button>
        </div>
    }
}
