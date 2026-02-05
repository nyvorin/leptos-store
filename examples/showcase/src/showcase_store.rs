// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Showcase store for tracking user interactions and preferences.

use leptos::prelude::*;
use leptos_store::Store;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Example metadata
#[derive(Debug, Clone)]
pub struct ExampleInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub port: u16,
    pub category: ExampleCategory,
    pub features: &'static [&'static str],
    pub difficulty: Difficulty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExampleCategory {
    Core,
    State,
    Advanced,
    Integration,
}

impl ExampleCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Core => "Core",
            Self::State => "State Management",
            Self::Advanced => "Advanced",
            Self::Integration => "Integration",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Core => "#3b82f6",
            Self::State => "#8b5cf6",
            Self::Advanced => "#f59e0b",
            Self::Integration => "#10b981",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

impl Difficulty {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Beginner => "Beginner",
            Self::Intermediate => "Intermediate",
            Self::Advanced => "Advanced",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            Self::Beginner => "#22c55e",
            Self::Intermediate => "#eab308",
            Self::Advanced => "#ef4444",
        }
    }
}

/// All available examples
pub const EXAMPLES: &[ExampleInfo] = &[
    ExampleInfo {
        id: "counter",
        name: "Counter",
        description: "Basic counter demonstrating store creation, state management, and reactive updates. The perfect starting point.",
        icon: "🔢",
        port: 3001,
        category: ExampleCategory::Core,
        features: &["Store", "RwSignal", "Reactivity"],
        difficulty: Difficulty::Beginner,
    },
    ExampleInfo {
        id: "middleware",
        name: "Middleware",
        description: "Action middleware for logging, timing, and validation. Intercept mutations before and after they execute.",
        icon: "🔗",
        port: 3010,
        category: ExampleCategory::Advanced,
        features: &["Middleware", "Logging", "Timing", "Validation"],
        difficulty: Difficulty::Intermediate,
    },
    ExampleInfo {
        id: "persistence",
        name: "Persistence",
        description: "Persist store state to localStorage or sessionStorage. Automatic save/load with versioning support.",
        icon: "💾",
        port: 3020,
        category: ExampleCategory::State,
        features: &["LocalStorage", "SessionStorage", "Versioning"],
        difficulty: Difficulty::Intermediate,
    },
    ExampleInfo {
        id: "composition",
        name: "Composition",
        description: "Compose multiple stores together. Share state between stores and create complex state hierarchies.",
        icon: "🧩",
        port: 3030,
        category: ExampleCategory::Advanced,
        features: &["Store Composition", "Shared State", "Computed"],
        difficulty: Difficulty::Advanced,
    },
    ExampleInfo {
        id: "feature-flags",
        name: "Feature Flags",
        description: "Runtime feature flags with percentage rollouts, user targeting, and A/B testing capabilities.",
        icon: "🚩",
        port: 3040,
        category: ExampleCategory::Integration,
        features: &["Feature Flags", "Rollouts", "A/B Testing"],
        difficulty: Difficulty::Intermediate,
    },
    ExampleInfo {
        id: "devtools",
        name: "DevTools",
        description: "Built-in developer tools for inspecting store state, tracking events, and debugging state changes.",
        icon: "🔧",
        port: 3050,
        category: ExampleCategory::Advanced,
        features: &["Inspector", "Event Log", "State Diff"],
        difficulty: Difficulty::Beginner,
    },
    ExampleInfo {
        id: "auth-store",
        name: "Auth Store",
        description: "Authentication state management with login/logout flows, session handling, and protected routes.",
        icon: "🔐",
        port: 3000,
        category: ExampleCategory::Integration,
        features: &["Authentication", "Sessions", "Protected Routes"],
        difficulty: Difficulty::Intermediate,
    },
    ExampleInfo {
        id: "token-explorer",
        name: "Token Explorer",
        description: "Cryptocurrency token explorer with real-time data, pagination, and advanced filtering.",
        icon: "🪙",
        port: 3005,
        category: ExampleCategory::Integration,
        features: &["API Integration", "Pagination", "Filtering"],
        difficulty: Difficulty::Advanced,
    },
];

/// Showcase state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShowcaseState {
    /// IDs of examples the user has visited
    pub visited: HashSet<String>,
    /// Currently selected category filter
    pub category_filter: Option<String>,
    /// Search query
    pub search_query: String,
    /// Dark mode preference
    pub dark_mode: bool,
}

/// Showcase store
#[derive(Clone)]
pub struct ShowcaseStore {
    state: RwSignal<ShowcaseState>,
}

impl Default for ShowcaseStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ShowcaseStore {
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(ShowcaseState {
                dark_mode: true, // Default to dark mode
                ..Default::default()
            }),
        }
    }

    /// Mark an example as visited
    pub fn mark_visited(&self, example_id: &str) {
        self.state.update(|s| {
            s.visited.insert(example_id.to_string());
        });
    }

    /// Check if an example was visited
    pub fn is_visited(&self, example_id: &str) -> bool {
        self.state.get().visited.contains(example_id)
    }

    /// Set category filter
    pub fn set_category_filter(&self, category: Option<String>) {
        self.state.update(|s| {
            s.category_filter = category;
        });
    }

    /// Set search query
    pub fn set_search_query(&self, query: String) {
        self.state.update(|s| {
            s.search_query = query;
        });
    }

    /// Toggle dark mode
    pub fn toggle_dark_mode(&self) {
        self.state.update(|s| {
            s.dark_mode = !s.dark_mode;
        });
    }

    /// Get filtered examples
    pub fn get_filtered_examples(&self) -> Vec<&'static ExampleInfo> {
        let state = self.state.get();
        let query = state.search_query.to_lowercase();

        EXAMPLES
            .iter()
            .filter(|e| {
                // Category filter
                if let Some(ref cat) = state.category_filter {
                    if e.category.label() != cat {
                        return false;
                    }
                }

                // Search filter
                if !query.is_empty() {
                    let matches = e.name.to_lowercase().contains(&query)
                        || e.description.to_lowercase().contains(&query)
                        || e.features.iter().any(|f| f.to_lowercase().contains(&query));
                    if !matches {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Get visit count
    pub fn visit_count(&self) -> usize {
        self.state.get().visited.len()
    }
}

impl Store for ShowcaseStore {
    type State = ShowcaseState;

    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }

    fn name(&self) -> &'static str {
        "ShowcaseStore"
    }
}
