// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Task Store with Middleware
//!
//! This module demonstrates the middleware system by creating a simple
//! task store with logging, timing, and validation middleware.

use leptos::prelude::*;
use leptos_store::store::Store;

// ============================================================================
// Task State
// ============================================================================

/// A single task item.
#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub completed: bool,
}

/// State for the task store.
#[derive(Debug, Clone, Default)]
pub struct TaskState {
    pub tasks: Vec<Task>,
    pub next_id: u32,
    pub filter: TaskFilter,
}

/// Filter for displaying tasks.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TaskFilter {
    #[default]
    All,
    Active,
    Completed,
}

// ============================================================================
// Task Store
// ============================================================================

/// A task management store demonstrating middleware integration.
#[derive(Clone)]
pub struct TaskStore {
    state: RwSignal<TaskState>,
}

impl TaskStore {
    /// Create a new task store.
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(TaskState::default()),
        }
    }

    // === Getters ===

    /// Get all tasks based on the current filter.
    pub fn filtered_tasks(&self) -> Vec<Task> {
        self.state.with(|s| {
            s.tasks
                .iter()
                .filter(|t| match s.filter {
                    TaskFilter::All => true,
                    TaskFilter::Active => !t.completed,
                    TaskFilter::Completed => t.completed,
                })
                .cloned()
                .collect()
        })
    }

    /// Get the total task count.
    pub fn total_count(&self) -> usize {
        self.state.with(|s| s.tasks.len())
    }

    /// Get the active (incomplete) task count.
    pub fn active_count(&self) -> usize {
        self.state
            .with(|s| s.tasks.iter().filter(|t| !t.completed).count())
    }

    /// Get the completed task count.
    pub fn completed_count(&self) -> usize {
        self.state
            .with(|s| s.tasks.iter().filter(|t| t.completed).count())
    }

    /// Get the current filter.
    pub fn current_filter(&self) -> TaskFilter {
        self.state.with(|s| s.filter)
    }

    // === Actions (Public API) ===

    /// Add a new task.
    pub fn add_task(&self, title: String) {
        if title.trim().is_empty() {
            return;
        }
        self.state.update(|s| {
            let task = Task {
                id: s.next_id,
                title: title.trim().to_string(),
                completed: false,
            };
            s.tasks.push(task);
            s.next_id += 1;
        });
    }

    /// Remove a task by ID.
    pub fn remove_task(&self, id: u32) {
        self.state.update(|s| {
            s.tasks.retain(|t| t.id != id);
        });
    }

    /// Toggle a task's completed status.
    pub fn toggle_task(&self, id: u32) {
        self.state.update(|s| {
            if let Some(task) = s.tasks.iter_mut().find(|t| t.id == id) {
                task.completed = !task.completed;
            }
        });
    }

    /// Set the filter.
    pub fn set_filter(&self, filter: TaskFilter) {
        self.state.update(|s| {
            s.filter = filter;
        });
    }

    /// Clear all completed tasks.
    pub fn clear_completed(&self) {
        self.state.update(|s| {
            s.tasks.retain(|t| !t.completed);
        });
    }
}

impl Default for TaskStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for TaskStore {
    type State = TaskState;

    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}
