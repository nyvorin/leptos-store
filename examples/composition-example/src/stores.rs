// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Domain Stores for Composition Example
//!
//! This module defines multiple domain stores that will be composed
//! together using RootStore.

use leptos::prelude::*;
use leptos_store::store::Store;

// ============================================================================
// Auth Store
// ============================================================================

/// Authentication state.
#[derive(Debug, Clone, Default)]
pub struct AuthState {
    pub user: Option<User>,
    pub loading: bool,
}

/// User information.
#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub name: String,
    pub email: String,
}

/// Authentication store.
#[derive(Clone)]
pub struct AuthStore {
    state: RwSignal<AuthState>,
}

impl AuthStore {
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(AuthState::default()),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        self.state.with(|s| s.user.is_some())
    }

    pub fn user_name(&self) -> Option<String> {
        self.state.with(|s| s.user.as_ref().map(|u| u.name.clone()))
    }

    pub fn login(&self, name: String, email: String) {
        self.state.update(|s| {
            s.user = Some(User { id: 1, name, email });
            s.loading = false;
        });
    }

    pub fn logout(&self) {
        self.state.update(|s| {
            s.user = None;
        });
    }
}

impl Default for AuthStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for AuthStore {
    type State = AuthState;

    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}

// ============================================================================
// Cart Store
// ============================================================================

/// Cart item.
#[derive(Debug, Clone)]
pub struct CartItem {
    pub id: u32,
    pub name: String,
    pub price: f64,
    pub quantity: u32,
}

/// Cart state.
#[derive(Debug, Clone, Default)]
pub struct CartState {
    pub items: Vec<CartItem>,
}

/// Shopping cart store.
#[derive(Clone)]
pub struct CartStore {
    state: RwSignal<CartState>,
}

impl CartStore {
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(CartState::default()),
        }
    }

    pub fn items(&self) -> Vec<CartItem> {
        self.state.with(|s| s.items.clone())
    }

    pub fn item_count(&self) -> usize {
        self.state
            .with(|s| s.items.iter().map(|i| i.quantity as usize).sum())
    }

    pub fn total(&self) -> f64 {
        self.state
            .with(|s| s.items.iter().map(|i| i.price * i.quantity as f64).sum())
    }

    pub fn add_item(&self, name: String, price: f64) {
        self.state.update(|s| {
            let id = s.items.len() as u32 + 1;
            s.items.push(CartItem {
                id,
                name,
                price,
                quantity: 1,
            });
        });
    }

    pub fn remove_item(&self, id: u32) {
        self.state.update(|s| {
            s.items.retain(|i| i.id != id);
        });
    }

    pub fn clear(&self) {
        self.state.update(|s| {
            s.items.clear();
        });
    }
}

impl Default for CartStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for CartStore {
    type State = CartState;

    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}

// ============================================================================
// UI Store
// ============================================================================

/// UI state.
#[derive(Debug, Clone, Default)]
pub struct UiState {
    pub theme: Theme,
    pub sidebar_open: bool,
    pub notifications: Vec<Notification>,
}

/// Theme options.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Theme {
    #[default]
    Light,
    Dark,
}

/// A notification.
#[derive(Debug, Clone)]
pub struct Notification {
    pub id: u32,
    pub message: String,
    pub notification_type: NotificationType,
}

/// Notification types.
#[derive(Debug, Clone, Copy)]
pub enum NotificationType {
    Info,
    Success,
    Warning,
    Error,
}

/// UI store.
#[derive(Clone)]
pub struct UiStore {
    state: RwSignal<UiState>,
    next_notification_id: RwSignal<u32>,
}

impl UiStore {
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(UiState::default()),
            next_notification_id: RwSignal::new(1),
        }
    }

    pub fn theme(&self) -> Theme {
        self.state.with(|s| s.theme)
    }

    pub fn is_dark(&self) -> bool {
        self.state.with(|s| s.theme == Theme::Dark)
    }

    pub fn toggle_theme(&self) {
        self.state.update(|s| {
            s.theme = match s.theme {
                Theme::Light => Theme::Dark,
                Theme::Dark => Theme::Light,
            };
        });
    }

    pub fn notifications(&self) -> Vec<Notification> {
        self.state.with(|s| s.notifications.clone())
    }

    pub fn add_notification(&self, message: String, notification_type: NotificationType) {
        let id = self.next_notification_id.get();
        self.next_notification_id.update(|n| *n += 1);

        self.state.update(|s| {
            s.notifications.push(Notification {
                id,
                message,
                notification_type,
            });
        });
    }

    pub fn dismiss_notification(&self, id: u32) {
        self.state.update(|s| {
            s.notifications.retain(|n| n.id != id);
        });
    }
}

impl Default for UiStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Store for UiStore {
    type State = UiState;

    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}
