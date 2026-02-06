// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! UI Components for the Persistence Example

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use leptos_store::prelude::*;

use crate::notes_store::NotesStore;

#[cfg(feature = "hydrate")]
use leptos::task::spawn_local;

/// Embeddable demo component for the showcase.
///
/// Sets up PersistentStore with a different key prefix to avoid conflicts.
#[component]
pub fn Demo() -> impl IntoView {
    let store = NotesStore::new();

    #[cfg(feature = "hydrate")]
    let persistent_store = {
        let adapter = LocalStorageAdapter::new();
        let ps = PersistentStore::new(store.clone(), adapter)
            .with_key("notes_store")
            .with_key_prefix("showcase_persistence")
            .with_version(1);

        let ps_load = ps.clone();
        let store_load = store.clone();
        Effect::new(move |_| {
            let ps = ps_load.clone();
            let store = store_load.clone();
            spawn_local(async move {
                if let Ok(Some(state)) = ps.load().await {
                    store.load_state(state);
                }
            });
        });

        let ps_save = ps.clone();
        let store_save = store.clone();
        Effect::new(move |_| {
            let _state = store_save.get_state();
            let ps = ps_save.clone();
            spawn_local(async move {
                let _ = ps.save().await;
            });
        });

        ps
    };

    #[cfg(feature = "hydrate")]
    provide_context(persistent_store);

    provide_store(store);
    view! { <NotesPage /> }
}

/// Main app component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Create the notes store
    let store = NotesStore::new();

    // Wrap with PersistentStore using LocalStorageAdapter (client-side only)
    #[cfg(feature = "hydrate")]
    let persistent_store = {
        let adapter = LocalStorageAdapter::new();
        let ps = PersistentStore::new(store.clone(), adapter)
            .with_key("notes_store")
            .with_key_prefix("persistence_example")
            .with_version(1);

        // Load persisted state on mount
        let ps_load = ps.clone();
        let store_load = store.clone();
        Effect::new(move |_| {
            let ps = ps_load.clone();
            let store = store_load.clone();
            spawn_local(async move {
                if let Ok(Some(state)) = ps.load().await {
                    store.load_state(state);
                }
            });
        });

        // Save state on changes (reactive effect tracks state changes)
        let ps_save = ps.clone();
        let store_save = store.clone();
        Effect::new(move |_| {
            // Track state changes by reading the state
            let _state = store_save.get_state();
            let ps = ps_save.clone();
            spawn_local(async move {
                let _ = ps.save().await;
            });
        });

        ps
    };

    // Provide the persistent store for context access
    #[cfg(feature = "hydrate")]
    provide_context(persistent_store);

    provide_store(store);

    view! {
        <Stylesheet id="leptos" href="/pkg/persistence-example.css"/>
        <Title text="Persistence Example - leptos-store"/>
        <Meta name="description" content="Persistence demonstration using leptos-store"/>

        <Router>
            <main class="app">
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=NotesPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Notes page component
#[component]
fn NotesPage() -> impl IntoView {
    view! {
        <div class="notes-page">
            <div class="notes-container">
                <header class="notes-header">
                    <h1>"Persistent Notes"</h1>
                    <p class="subtitle">"Notes are saved to localStorage automatically"</p>
                </header>

                <div class="notes-layout">
                    <NotesList />
                    <NoteEditor />
                </div>

                <div class="code-hint">
                    <p>"Using PersistentStore with LocalStorageAdapter:"</p>
                    <pre><code>{r#"// Create a persistent store
let adapter = LocalStorageAdapter::new();
let persistent = PersistentStore::new(store, adapter)
    .with_key("notes_store")
    .with_key_prefix("my_app")
    .with_version(1);

// Load persisted state on mount
spawn_local(async move {
    if let Ok(Some(state)) = persistent.load().await {
        store.load_state(state);
    }
});

// Save on changes
spawn_local(async move {
    persistent.save().await;
});"#}</code></pre>
                </div>
            </div>
        </div>
    }
}

/// Notes list sidebar
#[component]
fn NotesList() -> impl IntoView {
    let store = use_store::<NotesStore>();
    let store_count = store.clone();
    let store_for_list = store.clone();
    let store_add = store.clone();
    let store_clear = store.clone();
    let store_select = store.clone();

    view! {
        <div class="notes-sidebar">
            <div class="sidebar-header">
                <span class="note-count">
                    {move || format!("{} notes", store_count.count())}
                </span>
                <button class="btn btn-add" on:click=move |_| store_add.add_note("New Note".to_string(), "".to_string())>
                    "+ New"
                </button>
            </div>

            <ul class="notes-list">
                {move || {
                    let selected_id = store_for_list.selected_id();
                    store_for_list
                        .notes()
                        .into_iter()
                        .map(|note| {
                            let store_s = store_select.clone();
                            let note_id = note.id;
                            let is_selected = selected_id == Some(note_id);
                            let item_class = if is_selected { "note-item selected" } else { "note-item" };

                            view! {
                                <li
                                    class=item_class
                                    on:click=move |_| store_s.select_note(note_id)
                                >
                                    <span class="note-title">{note.title.clone()}</span>
                                    <span class="note-preview">
                                        {note.content.chars().take(50).collect::<String>()}
                                    </span>
                                </li>
                            }
                        })
                        .collect_view()
                }}
            </ul>

            <button
                class="btn btn-clear"
                on:click=move |_| store_clear.clear_all()
            >
                "Clear All"
            </button>
        </div>
    }
}

/// Note editor component
#[component]
fn NoteEditor() -> impl IntoView {
    let store = use_store::<NotesStore>();
    let store_selected = store.clone();
    let store_title = store.clone();
    let store_content = store.clone();
    let store_delete = store.clone();

    view! {
        <div class="note-editor">
            {move || {
                if let Some(note) = store_selected.selected_note() {
                    let note_id = note.id;
                    let st = store_title.clone();
                    let sc = store_content.clone();
                    let sd = store_delete.clone();

                    view! {
                        <div class="editor-content">
                            <input
                                type="text"
                                class="editor-title"
                                prop:value=note.title.clone()
                                on:input=move |ev| {
                                    let title = event_target_value(&ev);
                                    if let Some(current) = st.selected_note() {
                                        st.update_note(note_id, title, current.content);
                                    }
                                }
                            />
                            <textarea
                                class="editor-body"
                                prop:value=note.content.clone()
                                on:input=move |ev| {
                                    let content = event_target_value(&ev);
                                    if let Some(current) = sc.selected_note() {
                                        sc.update_note(note_id, current.title, content);
                                    }
                                }
                            />
                            <div class="editor-footer">
                                <span class="note-date">
                                    {format_date(note.created_at)}
                                </span>
                                <button
                                    class="btn btn-delete"
                                    on:click=move |_| sd.delete_note(note_id)
                                >
                                    "Delete"
                                </button>
                            </div>
                        </div>
                    }.into_any()
                } else {
                    view! {
                        <div class="editor-empty">
                            <p>"Select a note or create a new one"</p>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// Format a timestamp for display.
fn format_date(timestamp: u64) -> String {
    // Simple date formatting
    let secs = timestamp / 1000;
    let mins = (secs / 60) % 60;
    let hours = (secs / 3600) % 24;
    format!("Created at {:02}:{:02}", hours, mins)
}
