// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Notes Store with Persistence
//!
//! This module demonstrates the persistence system by creating a simple
//! notes store that saves to localStorage.

use leptos::prelude::*;
use leptos_store::store::Store;
use serde::{Deserialize, Serialize};

// ============================================================================
// Note State
// ============================================================================

/// A single note item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: u32,
    pub title: String,
    pub content: String,
    pub created_at: u64,
}

/// State for the notes store.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotesState {
    pub notes: Vec<Note>,
    pub next_id: u32,
    pub selected_id: Option<u32>,
}

// ============================================================================
// Notes Store
// ============================================================================

/// A notes management store demonstrating persistence.
#[derive(Clone)]
pub struct NotesStore {
    state: RwSignal<NotesState>,
}

impl NotesStore {
    /// Create a new notes store.
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(NotesState::default()),
        }
    }

    /// Create a store with initial state.
    pub fn with_state(state: NotesState) -> Self {
        Self {
            state: RwSignal::new(state),
        }
    }

    // === Getters ===

    /// Get all notes.
    pub fn notes(&self) -> Vec<Note> {
        self.state.with(|s| s.notes.clone())
    }

    /// Get the note count.
    pub fn count(&self) -> usize {
        self.state.with(|s| s.notes.len())
    }

    /// Get the selected note.
    pub fn selected_note(&self) -> Option<Note> {
        self.state.with(|s| {
            s.selected_id
                .and_then(|id| s.notes.iter().find(|n| n.id == id).cloned())
        })
    }

    /// Get the selected note ID.
    pub fn selected_id(&self) -> Option<u32> {
        self.state.with(|s| s.selected_id)
    }

    // === Actions (Public API) ===

    /// Add a new note.
    pub fn add_note(&self, title: String, content: String) {
        self.state.update(|s| {
            let note = Note {
                id: s.next_id,
                title,
                content,
                created_at: current_timestamp(),
            };
            s.notes.push(note);
            s.selected_id = Some(s.next_id);
            s.next_id += 1;
        });
    }

    /// Update a note.
    pub fn update_note(&self, id: u32, title: String, content: String) {
        self.state.update(|s| {
            if let Some(note) = s.notes.iter_mut().find(|n| n.id == id) {
                note.title = title;
                note.content = content;
            }
        });
    }

    /// Delete a note.
    pub fn delete_note(&self, id: u32) {
        self.state.update(|s| {
            s.notes.retain(|n| n.id != id);
            if s.selected_id == Some(id) {
                s.selected_id = s.notes.first().map(|n| n.id);
            }
        });
    }

    /// Select a note.
    pub fn select_note(&self, id: u32) {
        self.state.update(|s| {
            s.selected_id = Some(id);
        });
    }

    /// Clear all notes.
    pub fn clear_all(&self) {
        self.state.update(|s| {
            s.notes.clear();
            s.selected_id = None;
        });
    }

    /// Load state (for persistence).
    pub fn load_state(&self, state: NotesState) {
        self.state.set(state);
    }

    /// Get current state (for persistence).
    pub fn get_state(&self) -> NotesState {
        self.state.get()
    }
}

impl Default for NotesStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for NotesStore {
    type State = NotesState;

    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}

/// Get current timestamp in milliseconds.
fn current_timestamp() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}
