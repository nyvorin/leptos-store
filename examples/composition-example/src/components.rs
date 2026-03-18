// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! UI Components for the Composition Example

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use leptos_store::composition::{RootStore, provide_root_store, use_root_store};

use crate::stores::{AuthStore, CartStore, NotificationType, UiStore};

/// Embeddable demo component for the showcase.
///
/// Composes multiple stores into a RootStore and renders the dashboard.
#[component]
pub fn Demo() -> impl IntoView {
    let auth_store = AuthStore::new();
    let cart_store = CartStore::new();
    let ui_store = UiStore::new();
    let root = RootStore::builder()
        .with_store(auth_store)
        .with_store(cart_store)
        .with_store(ui_store)
        .build();
    provide_root_store(root);
    view! { <DashboardPage /> }
}

/// Main app component
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // Create individual stores
    let auth_store = AuthStore::new();
    let cart_store = CartStore::new();
    let ui_store = UiStore::new();

    // Compose them into a RootStore
    let root = RootStore::builder()
        .with_store(auth_store)
        .with_store(cart_store)
        .with_store(ui_store)
        .build();

    // Provide the root store to the component tree
    provide_root_store(root);

    view! {
        <Stylesheet id="leptos" href="/pkg/composition-example.css"/>
        <Title text="Composition Example - leptos-store"/>
        <Meta name="description" content="Store composition demonstration using leptos-store"/>

        <Router>
            <main class="app">
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=DashboardPage/>
                </Routes>
            </main>
        </Router>
    }
}

/// Dashboard page showing composed stores
#[component]
fn DashboardPage() -> impl IntoView {
    let root = use_root_store();

    view! {
        <div class="dashboard-page">
            <header class="dashboard-header">
                <h1>"Store Composition Example"</h1>
                <p class="subtitle">
                    "Combining "
                    <strong>{root.len()}</strong>
                    " stores with RootStore"
                </p>
            </header>

            <div class="dashboard-grid">
                <AuthPanel />
                <CartPanel />
                <UiPanel />
                <CheckoutStatus />
            </div>

            <div class="code-hint">
                <p>"Composing stores with RootStore:"</p>
                <pre><code>{r#"let root = RootStore::builder()
    .with_store(AuthStore::new())
    .with_store(CartStore::new())
    .with_store(UiStore::new())
    .build();

provide_root_store(root);

// Access individual stores
let auth = use_root_store().get::<AuthStore>();"#}</code></pre>
            </div>
        </div>
    }
}

/// Auth panel component
#[component]
fn AuthPanel() -> impl IntoView {
    let root = use_root_store();
    let auth = root.expect::<AuthStore>().clone();
    let ui = root.expect::<UiStore>().clone();

    let auth_check = auth.clone();
    let auth_name = auth.clone();
    let auth_login = auth.clone();
    let auth_logout = auth.clone();
    let ui_login = ui.clone();
    let ui_logout = ui.clone();

    view! {
        <div class="panel auth-panel">
            <h2>"Auth Store"</h2>

            {move || {
                if auth_check.is_authenticated() {
                    let name = auth_name.user_name().unwrap_or_default();
                    let auth_lo = auth_logout.clone();
                    let ui_lo = ui_logout.clone();
                    view! {
                        <div class="auth-info">
                            <div class="user-avatar">{name.chars().next().unwrap_or('?')}</div>
                            <span class="user-name">{name}</span>
                            <button
                                class="btn btn-logout"
                                on:click=move |_| {
                                    auth_lo.logout();
                                    ui_lo.add_notification("Logged out".to_string(), NotificationType::Info);
                                }
                            >
                                "Logout"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    let auth_l = auth_login.clone();
                    let ui_l = ui_login.clone();
                    view! {
                        <div class="auth-login">
                            <p>"Not logged in"</p>
                            <button
                                class="btn btn-login"
                                on:click=move |_| {
                                    auth_l.login("Demo User".to_string(), "demo@example.com".to_string());
                                    ui_l.add_notification("Logged in successfully!".to_string(), NotificationType::Success);
                                }
                            >
                                "Login as Demo User"
                            </button>
                        </div>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// Cart panel component
#[component]
fn CartPanel() -> impl IntoView {
    let root = use_root_store();
    let cart = root.expect::<CartStore>().clone();

    let cart_display = cart.clone();
    let cart_count = cart.clone();
    let cart_total = cart.clone();

    // Sample products
    let products = vec![("Widget", 9.99), ("Gadget", 19.99), ("Gizmo", 14.99)];

    view! {
        <div class="panel cart-panel">
            <h2>"Cart Store"</h2>

            <div class="cart-stats">
                <div class="stat">
                    <span class="stat-value">{move || cart_count.item_count()}</span>
                    <span class="stat-label">"items"</span>
                </div>
                <div class="stat">
                    <span class="stat-value">{move || format!("${:.2}", cart_total.total())}</span>
                    <span class="stat-label">"total"</span>
                </div>
            </div>

            <div class="product-list">
                {products.into_iter().map(|(name, price)| {
                    let cart_add = cart.clone();
                    let name_owned = name.to_string();
                    view! {
                        <button
                            class="product-btn"
                            on:click=move |_| cart_add.add_item(name_owned.clone(), price)
                        >
                            {name} " $" {price}
                        </button>
                    }
                }).collect_view()}
            </div>

            <ul class="cart-items">
                {move || {
                    cart_display.items().into_iter().map(|item| {
                        let cart_remove = cart.clone();
                        let item_id = item.id;
                        view! {
                            <li class="cart-item">
                                <span>{item.name}</span>
                                <span class="item-price">"${:.2}" {item.price}</span>
                                <button
                                    class="btn-remove"
                                    on:click=move |_| cart_remove.remove_item(item_id)
                                >
                                    "×"
                                </button>
                            </li>
                        }
                    }).collect_view()
                }}
            </ul>
        </div>
    }
}

/// UI panel component
#[component]
fn UiPanel() -> impl IntoView {
    let root = use_root_store();
    let ui = root.expect::<UiStore>().clone();

    let ui_toggle = ui.clone();
    let ui_theme = ui.clone();
    let ui_notifications = ui.clone();
    let ui_dismiss = ui.clone();

    view! {
        <div class="panel ui-panel">
            <h2>"UI Store"</h2>

            <div class="theme-toggle">
                <span>"Theme:"</span>
                <button
                    class="btn btn-theme"
                    on:click=move |_| ui_toggle.toggle_theme()
                >
                    {move || if ui_theme.is_dark() { "Dark" } else { "Light" }}
                </button>
            </div>

            <div class="notifications">
                <h3>"Notifications"</h3>
                <ul class="notification-list">
                    {move || {
                        ui_notifications.notifications().into_iter().map(|n| {
                            let ui_d = ui_dismiss.clone();
                            let n_id = n.id;
                            view! {
                                <li class="notification">
                                    <span>{n.message}</span>
                                    <button
                                        class="btn-dismiss"
                                        on:click=move |_| ui_d.dismiss_notification(n_id)
                                    >
                                        "×"
                                    </button>
                                </li>
                            }
                        }).collect_view()
                    }}
                </ul>
            </div>
        </div>
    }
}

/// Checkout status - derived from multiple stores
#[component]
fn CheckoutStatus() -> impl IntoView {
    let root = use_root_store();
    let auth = root.expect::<AuthStore>().clone();
    let cart = root.expect::<CartStore>().clone();

    let auth_class = auth.clone();
    let auth_icon = auth.clone();
    let cart_class = cart.clone();
    let cart_icon = cart.clone();
    let auth_btn = auth.clone();
    let cart_btn = cart.clone();
    let auth_text = auth.clone();
    let cart_text = cart.clone();

    view! {
        <div class="panel checkout-panel">
            <h2>"Derived State"</h2>
            <p class="derived-desc">"Computed from Auth + Cart stores"</p>

            <div class="checkout-status">
                <div class="status-item">
                    <span class={move || if auth_class.is_authenticated() { "status-icon ready" } else { "status-icon" }}>
                        {move || if auth_icon.is_authenticated() { "✓" } else { "○" }}
                    </span>
                    <span>"Logged in"</span>
                </div>
                <div class="status-item">
                    <span class={move || if cart_class.item_count() > 0 { "status-icon ready" } else { "status-icon" }}>
                        {move || if cart_icon.item_count() > 0 { "✓" } else { "○" }}
                    </span>
                    <span>"Has items"</span>
                </div>
            </div>

            <button
                class="btn btn-checkout"
                disabled=move || !(auth_btn.is_authenticated() && cart_btn.item_count() > 0)
            >
                {move || if auth_text.is_authenticated() && cart_text.item_count() > 0 { "Checkout Ready!" } else { "Cannot Checkout" }}
            </button>
        </div>
    }
}
