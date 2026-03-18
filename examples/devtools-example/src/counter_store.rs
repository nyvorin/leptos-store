// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Counter Store for Devtools Example
//!
//! A simple counter store to demonstrate devtools inspection.

use leptos::prelude::*;
use leptos_store::store::Store;
use serde::Serialize;

/// Counter state.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CounterState {
    pub count: i32,
    pub history: Vec<i32>,
}

/// Counter store for devtools demonstration.
#[derive(Clone)]
pub struct CounterStore {
    state: RwSignal<CounterState>,
}

impl CounterStore {
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(CounterState::default()),
        }
    }

    // === Getters ===

    pub fn count(&self) -> i32 {
        self.state.with(|s| s.count)
    }

    pub fn doubled(&self) -> i32 {
        self.state.with(|s| s.count * 2)
    }

    pub fn history(&self) -> Vec<i32> {
        self.state.with(|s| s.history.clone())
    }

    pub fn history_len(&self) -> usize {
        self.state.with(|s| s.history.len())
    }

    // === Actions ===

    pub fn increment(&self) {
        self.state.update(|s| {
            s.history.push(s.count);
            s.count += 1;
        });
    }

    pub fn decrement(&self) {
        self.state.update(|s| {
            s.history.push(s.count);
            s.count -= 1;
        });
    }

    pub fn set(&self, value: i32) {
        self.state.update(|s| {
            s.history.push(s.count);
            s.count = value;
        });
    }

    pub fn reset(&self) {
        self.state.update(|s| {
            s.history.push(s.count);
            s.count = 0;
        });
    }

    pub fn clear_history(&self) {
        self.state.update(|s| {
            s.history.clear();
        });
    }
}

impl Default for CounterStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for CounterStore {
    type State = CounterState;

    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}
