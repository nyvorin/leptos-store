// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! CSR Todo Example
//!
//! This example demonstrates how to use leptos-store in a
//! client-side rendered (CSR) application — no SSR, no hydration,
//! no server. It showcases:
//!
//! - Store definition with the `store!` macro
//! - Getters, mutators, and actions
//! - CSR-only store initialization with `provide_store`
//! - Reactive UI updates with a todo-list pattern

use leptos::prelude::*;
use leptos_store::prelude::*;
use leptos_store::store;

store! {
    pub TodoStore {
        state TodoState {
            items: Vec<TodoItem> = Vec::new(),
            next_id: u32 = 1,
            filter: TodoFilter = TodoFilter::All,
        }

        getters {
            visible_items(this) -> Vec<TodoItem> {
                this.read(|s| {
                    match s.filter {
                        TodoFilter::All => s.items.clone(),
                        TodoFilter::Active => s.items.iter().filter(|i| !i.done).cloned().collect(),
                        TodoFilter::Completed => s.items.iter().filter(|i| i.done).cloned().collect(),
                    }
                })
            }

            active_count(this) -> usize {
                this.read(|s| s.items.iter().filter(|i| !i.done).count())
            }

            total_count(this) -> usize {
                this.read(|s| s.items.len())
            }
        }

        mutators {
            push_item(this, text: String) {
                this.mutate(|s| {
                    s.items.push(TodoItem { id: s.next_id, text, done: false });
                    s.next_id += 1;
                });
            }

            toggle_item(this, id: u32) {
                this.mutate(|s| {
                    if let Some(item) = s.items.iter_mut().find(|i| i.id == id) {
                        item.done = !item.done;
                    }
                });
            }

            remove_item(this, id: u32) {
                this.mutate(|s| { s.items.retain(|i| i.id != id); });
            }

            set_filter(this, filter: TodoFilter) {
                this.mutate(|s| s.filter = filter);
            }
        }

        actions {
            add_todo(this, text: String) {
                if !text.trim().is_empty() {
                    this.push_item(text.trim().to_string());
                }
            }

            toggle_todo(this, id: u32) { this.toggle_item(id); }
            delete_todo(this, id: u32) { this.remove_item(id); }
            set_todo_filter(this, filter: TodoFilter) { this.set_filter(filter); }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TodoItem {
    pub id: u32,
    pub text: String,
    pub done: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum TodoFilter {
    #[default]
    All,
    Active,
    Completed,
}

/// Main app component. Demonstrates CSR-only store initialization.
///
/// In CSR mode, just create the store and provide it — no server,
/// no hydration, no serialization needed.
#[component]
pub fn App() -> impl IntoView {
    let store = TodoStore::new();
    provide_store(store);

    view! {
        <div class="todo-app">
            <h1>"Leptos Store \u{2014} CSR Todo"</h1>
            <p class="subtitle">"Client-side only state management"</p>
            <TodoInput />
            <TodoFilters />
            <TodoList />
            <TodoFooter />
        </div>
    }
}

#[component]
fn TodoInput() -> impl IntoView {
    let store = use_store::<TodoStore>();
    let (input, set_input) = signal(String::new());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        store.add_todo(input.get());
        set_input.set(String::new());
    };

    view! {
        <form on:submit=on_submit class="todo-input">
            <input
                type="text"
                placeholder="What needs to be done?"
                prop:value=input
                on:input=move |ev| set_input.set(event_target_value(&ev))
            />
            <button type="submit">"Add"</button>
        </form>
    }
}

#[component]
fn TodoFilters() -> impl IntoView {
    let store = use_store::<TodoStore>();

    let store_all = store.clone();
    let store_active = store.clone();
    let store_completed = store.clone();

    view! {
        <div class="todo-filters">
            <button on:click=move |_| store_all.set_todo_filter(TodoFilter::All)>"All"</button>
            <button on:click=move |_| store_active.set_todo_filter(TodoFilter::Active)>"Active"</button>
            <button on:click=move |_| store_completed.set_todo_filter(TodoFilter::Completed)>"Completed"</button>
        </div>
    }
}

#[component]
fn TodoList() -> impl IntoView {
    let store = use_store::<TodoStore>();

    view! {
        <ul class="todo-list">
            {move || {
                store.visible_items().into_iter().map(|item| {
                    let id = item.id;
                    let store_toggle = store.clone();
                    let store_delete = store.clone();
                    view! {
                        <li class:done=item.done>
                            <input
                                type="checkbox"
                                prop:checked=item.done
                                on:change=move |_| store_toggle.toggle_todo(id)
                            />
                            <span>{item.text.clone()}</span>
                            <button class="delete" on:click=move |_| store_delete.delete_todo(id)>"x"</button>
                        </li>
                    }
                }).collect_view()
            }}
        </ul>
    }
}

#[component]
fn TodoFooter() -> impl IntoView {
    let store = use_store::<TodoStore>();

    let store_active = store.clone();
    let store_total = store.clone();

    view! {
        <div class="todo-footer">
            <span>{move || store_active.active_count()} " items remaining"</span>
            <span>" | "</span>
            <span>{move || store_total.total_count()} " total"</span>
        </div>
    }
}
