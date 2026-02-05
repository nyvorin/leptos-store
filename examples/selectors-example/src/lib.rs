// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Selectors Example
//!
//! This example demonstrates fine-grained reactivity using the selector system
//! from leptos-store. It showcases:
//!
//! - `create_selector` - Extract a specific slice from store state
//! - `combine_selectors` - Combine two selectors into a derived value
//! - `map_selector` - Transform a selector's output
//! - `filter_selector` - Conditionally emit values from a selector
//! - `selector!` macro - Declare multiple selectors at once
//!
//! Each panel subscribes to a specific state slice and only re-renders
//! when that slice changes, demonstrating fine-grained reactivity.

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use leptos_store::prelude::*;
use leptos_store::{selector, store};

// ============================================================================
// Store with multiple domains (user + cart) to demonstrate selectors
// ============================================================================

store! {
    pub DashboardStore {
        state DashboardState {
            user_name: String = String::from("Alice"),
            user_role: String = String::from("admin"),
            cart_items: Vec<String> = Vec::new(),
            cart_discount: f64 = 10.0,
            notification_count: u32 = 0,
        }

        getters {}

        mutators {
            set_user_name(this, name: String) {
                this.mutate(|s| s.user_name = name);
            }
            set_user_role(this, role: String) {
                this.mutate(|s| s.user_role = role);
            }
            add_cart_item(this, item: String) {
                this.mutate(|s| s.cart_items.push(item));
            }
            clear_cart(this) {
                this.mutate(|s| s.cart_items.clear());
            }
            set_discount(this, pct: f64) {
                this.mutate(|s| s.cart_discount = pct);
            }
            set_notifications(this, count: u32) {
                this.mutate(|s| s.notification_count = count);
            }
        }

        actions {
            update_name(this, name: String) {
                this.set_user_name(name);
            }
            update_role(this, role: String) {
                this.set_user_role(role);
            }
            add_item(this, item: String) {
                this.add_cart_item(item);
            }
            clear_all_items(this) {
                this.clear_cart();
            }
            update_discount(this, pct: f64) {
                this.set_discount(pct);
            }
            increment_notifications(this) {
                let current = this.read(|s| s.notification_count);
                this.set_notifications(current + 1);
            }
        }
    }
}

// ============================================================================
// App component
// ============================================================================

/// Main app component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let store = DashboardStore::new();
    provide_store(store);

    view! {
        <Stylesheet id="leptos" href="/pkg/selectors-example.css"/>
        <Title text="Selectors Example - leptos-store"/>
        <Meta name="description" content="Fine-grained reactivity with selectors in leptos-store"/>

        <Router>
            <main>
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=DashboardPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Dashboard page displaying all selector panels
#[component]
fn DashboardPage() -> impl IntoView {
    view! {
        <div class="dashboard">
            <h1>"Selectors Example"</h1>
            <p class="subtitle">
                "Each panel uses a different selector type and only re-renders when its slice changes"
            </p>

            <div class="controls">
                <h2>"Controls"</h2>
                <ControlPanel/>
            </div>

            <div class="panels">
                <UserPanel/>
                <CartPanel/>
                <NotificationPanel/>
                <FilteredCartPanel/>
            </div>
        </div>
    }
}

// ============================================================================
// Control panel - mutates different parts of state
// ============================================================================

/// Control panel that mutates different parts of the store state.
/// Buttons here trigger changes that only affect specific panels.
#[component]
fn ControlPanel() -> impl IntoView {
    let store = use_store::<DashboardStore>();
    let counter = RwSignal::new(0u32);

    let store_name = store.clone();
    let store_item = store.clone();
    let store_disc = store.clone();
    let store_notif = store.clone();
    let store_clear = store.clone();

    view! {
        <div class="control-panel">
            <button on:click=move |_| {
                counter.update(|c| *c += 1);
                store_name.update_name(format!("User_{}", counter.get()));
            }>
                "Change Name"
            </button>
            <button on:click=move |_| {
                counter.update(|c| *c += 1);
                store_item.add_item(format!("Item_{}", counter.get()));
            }>
                "Add Cart Item"
            </button>
            <button on:click=move |_| {
                store_disc.update_discount(20.0);
            }>
                "Set 20% Discount"
            </button>
            <button on:click=move |_| {
                store_notif.increment_notifications();
            }>
                "Add Notification"
            </button>
            <button on:click=move |_| {
                store_clear.clear_all_items();
            }>
                "Clear Cart"
            </button>
        </div>
    }
}

// ============================================================================
// UserPanel - demonstrates create_selector and selector! macro
// ============================================================================

/// Uses `create_selector` and the `selector!` macro to extract user fields.
/// Only re-renders when user name or role changes.
#[component]
fn UserPanel() -> impl IntoView {
    let store = use_store::<DashboardStore>();

    // Demonstrate the selector! macro: creates multiple selectors at once
    selector! {
        store: &store,
        user_name: |s: &DashboardState| -> String { s.user_name.clone() },
        user_role: |s: &DashboardState| -> String { s.user_role.clone() },
    }

    view! {
        <div class="panel">
            <h3>"User Panel (create_selector + selector! macro)"</h3>
            <p>"Name: " {move || user_name.get()}</p>
            <p>"Role: " {move || user_role.get()}</p>
            <p class="hint">
                "Only re-renders when user name or role changes. "
                "Cart and notification changes have no effect."
            </p>
        </div>
    }
}

// ============================================================================
// CartPanel - demonstrates combine_selectors
// ============================================================================

/// Uses `combine_selectors` to derive a cart summary from item count + discount.
/// Only re-renders when item count or discount changes.
#[component]
fn CartPanel() -> impl IntoView {
    let store = use_store::<DashboardStore>();

    let item_count = create_selector(&store, |s| s.cart_items.len());
    let discount = create_selector(&store, |s| s.cart_discount);

    // combine_selectors: derive a formatted summary from two selectors
    let summary = combine_selectors(item_count, discount, |count, disc| {
        let subtotal = *count as f64 * 9.99;
        let total = subtotal * (1.0 - disc / 100.0);
        format!("{count} items, ${subtotal:.2} - {disc}% = ${total:.2}")
    });

    view! {
        <div class="panel">
            <h3>"Cart Summary (combine_selectors)"</h3>
            <p>{move || summary.get()}</p>
            <p class="hint">
                "Combines item count and discount into a single derived value. "
                "Only re-renders when either input selector changes."
            </p>
        </div>
    }
}

// ============================================================================
// NotificationPanel - demonstrates map_selector
// ============================================================================

/// Uses `map_selector` to transform a notification count into a display string.
/// Only re-renders when the notification count changes.
#[component]
fn NotificationPanel() -> impl IntoView {
    let store = use_store::<DashboardStore>();

    let notif_count = create_selector(&store, |s| s.notification_count);

    // map_selector: transform count into a human-readable badge string
    let notif_badge = map_selector(notif_count, |count| {
        if *count == 0 {
            "No notifications".to_string()
        } else {
            format!(
                "{count} notification{}",
                if *count == 1 { "" } else { "s" }
            )
        }
    });

    view! {
        <div class="panel">
            <h3>"Notifications (map_selector)"</h3>
            <p>{move || notif_badge.get()}</p>
            <p class="hint">
                "Transforms the raw notification count into a display string. "
                "Only re-renders when notification count changes."
            </p>
        </div>
    }
}

// ============================================================================
// FilteredCartPanel - demonstrates filter_selector
// ============================================================================

/// Uses `filter_selector` to only show cart items when the cart is non-empty.
/// Emits `None` when the cart is empty, `Some(items)` otherwise.
#[component]
fn FilteredCartPanel() -> impl IntoView {
    let store = use_store::<DashboardStore>();

    let items = create_selector(&store, |s| s.cart_items.clone());

    // filter_selector: only emit when predicate is true
    let non_empty_items = filter_selector(items, |items| !items.is_empty());

    view! {
        <div class="panel">
            <h3>"Active Cart (filter_selector)"</h3>
            {move || match non_empty_items.get() {
                Some(items) => view! {
                    <ul>
                        {items.into_iter().map(|item| view! { <li>{item}</li> }).collect_view()}
                    </ul>
                }.into_any(),
                None => view! {
                    <p class="empty">"Cart is empty — add items to see them here"</p>
                }.into_any(),
            }}
            <p class="hint">
                "Uses filter_selector to conditionally render. "
                "Returns None when cart is empty, Some(items) otherwise."
            </p>
        </div>
    }
}

// ============================================================================
// Hydration / CSR entry points
// ============================================================================

/// Hydration entry point - called on the client to hydrate the SSR HTML
#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

/// CSR entry point - mounts the app directly (no SSR)
#[cfg(feature = "csr")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
