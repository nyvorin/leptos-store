// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! UI Components for the Auth Store Example
//!
//! This module provides Leptos components that demonstrate
//! how to use the AuthStore in a real application.
//!
//! Supports SSR (Server-Side Rendering), hydration, and CSR modes.
//!
//! # Hydration Support
//!
//! When the `hydrate` feature is enabled, this example demonstrates:
//! - Server-side state serialization with `provide_hydrated_store`
//! - Client-side state restoration with `use_hydrated_store`
//!
//! The store state is automatically transferred from server to client,
//! ensuring no hydration mismatches.

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use leptos_store::prelude::*;

use crate::auth_store::{AuthStore, LoginCredentials};

/// Embeddable demo component for the showcase.
///
/// Creates a fresh (unauthenticated) store without hydration script reading.
#[component]
pub fn Demo() -> impl IntoView {
    let store = AuthStore::new();
    provide_store(store);
    view! { <AuthRouter /> }
}

/// Shell component that wraps the entire application.
///
/// Provides meta context and routing for SSR support.
///
/// # Hydration
///
/// When built with the `hydrate` feature:
/// - On the server: Creates store and renders hydration script
/// - On the client: Restores store state from serialized data
#[component]
pub fn App() -> impl IntoView {
    // Provides context for <Title> and <Meta> components
    provide_meta_context();

    // On SSR: Store is provided by main.rs before rendering App
    // On hydrate (client): Read from hydration script and provide to context
    #[cfg(feature = "hydrate")]
    {
        use crate::auth_store::AuthState;
        use leptos_store::hydration::{has_hydration_data, read_hydration_data};

        // Try to hydrate from server-rendered data
        if has_hydration_data("auth_store") {
            if let Ok(data) = read_hydration_data("auth_store") {
                if let Ok(state) = serde_json::from_str::<AuthState>(&data) {
                    let store = AuthStore::with_state(state);
                    provide_store(store);
                } else {
                    // Fallback to fresh store
                    provide_store(AuthStore::new());
                }
            } else {
                provide_store(AuthStore::new());
            }
        } else {
            // No hydration data (CSR mode)
            provide_store(AuthStore::new());
        }
    }

    view! {
        <Stylesheet id="leptos" href="/pkg/auth-store-example.css"/>
        <Title text="Auth Store Example"/>
        <Meta name="description" content="SSR Example for leptos-store"/>

        <Router>
            <main class="app">
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=AuthRouter/>
                </Routes>
            </main>
        </Router>
    }
}

/// Router component that shows login or dashboard based on auth state.
#[component]
fn AuthRouter() -> impl IntoView {
    let store = use_store::<AuthStore>();

    view! {
        <div class="auth-router">
            {move || {
                let store = store.clone();
                if store.is_authenticated() {
                    view! { <Dashboard /> }.into_any()
                } else {
                    view! { <LoginPage /> }.into_any()
                }
            }}
        </div>
    }
}

/// Login page component.
#[component]
fn LoginPage() -> impl IntoView {
    let store = use_store::<AuthStore>();

    // Local form state
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (remember_me, set_remember_me) = signal(false);

    // Clone store for each closure
    let store_submit = store.clone();
    let store_error = store.clone();
    let store_loading1 = store.clone();
    let store_loading2 = store.clone();

    // Form submission handler
    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();

        let credentials = LoginCredentials {
            email: email.get(),
            password: password.get(),
            remember_me: remember_me.get(),
        };

        store_submit.login(credentials);
    };

    view! {
        <div class="login-page">
            <div class="login-card">
                <h1>"Welcome Back"</h1>
                <p class="subtitle">"Sign in to your account"</p>

                // Error display
                {move || {
                    let store = store_error.clone();
                    store.error().map(|err| {
                        view! {
                            <div class="error-message">
                                {err.to_string()}
                            </div>
                        }
                    })
                }}

                <form on:submit=on_submit>
                    <div class="form-group">
                        <label for="email">"Email"</label>
                        <input
                            type="email"
                            id="email"
                            name="email"
                            placeholder="you@example.com"
                            prop:value=email
                            on:input:target=move |ev| {
                                set_email.set(ev.target().value());
                            }
                        />
                    </div>

                    <div class="form-group">
                        <label for="password">"Password"</label>
                        <input
                            type="password"
                            id="password"
                            name="password"
                            placeholder="••••••••"
                            prop:value=password
                            on:input:target=move |ev| {
                                set_password.set(ev.target().value());
                            }
                        />
                    </div>

                    <div class="form-group checkbox">
                        <input
                            type="checkbox"
                            id="remember"
                            name="remember"
                            prop:checked=remember_me
                            on:change:target=move |ev| {
                                set_remember_me.set(ev.target().checked());
                            }
                        />
                        <label for="remember">"Remember me"</label>
                    </div>

                    <button
                        type="submit"
                        class="btn-primary"
                        disabled=move || store_loading1.is_loading()
                    >
                        {move || if store_loading2.is_loading() { "Signing in..." } else { "Sign In" }}
                    </button>
                </form>

                <p class="demo-hint">
                    "Demo: Enter any email and password to log in"
                </p>
            </div>
        </div>
    }
}

/// Dashboard component shown after login.
#[component]
fn Dashboard() -> impl IntoView {
    let store = use_store::<AuthStore>();

    // Clone store for each closure
    let store_logout = store.clone();
    let store_name = store.clone();
    let store_email = store.clone();
    let store_user = store.clone();
    let store_status = store.clone();

    let on_logout = move |_| {
        store_logout.logout();
    };

    view! {
        <div class="dashboard">
            <header class="dashboard-header">
                <h1>"Dashboard"</h1>
                <div class="user-menu">
                    <UserAvatar />
                    <button class="btn-secondary" on:click=on_logout>
                        "Sign Out"
                    </button>
                </div>
            </header>

            <main class="dashboard-content">
                <div class="welcome-card">
                    <h2>"Welcome, " {move || store_name.display_name()} "!"</h2>
                    <p>"You are now logged in to your account."</p>
                </div>

                <div class="info-cards">
                    <InfoCard
                        title="Email"
                        value=Signal::derive(move || store_email.user_email().unwrap_or_default())
                    />
                    <InfoCard
                        title="User ID"
                        value=Signal::derive(move || store_user.current_user().map(|u| u.id).unwrap_or_default())
                    />
                    <InfoCard
                        title="Status"
                        value=Signal::derive(move || if store_status.is_authenticated() { "Authenticated".to_string() } else { "Not authenticated".to_string() })
                    />
                </div>
            </main>
        </div>
    }
}

/// User avatar component.
#[component]
fn UserAvatar() -> impl IntoView {
    let store = use_store::<AuthStore>();

    let store_user = store.clone();
    let store_initials = store.clone();

    view! {
        <div class="avatar">
            {move || {
                let user = store_user.current_user();
                if let Some(url) = user.as_ref().and_then(|u| u.avatar_url.clone()) {
                    view! {
                        <img src=url alt="User avatar" />
                    }.into_any()
                } else {
                    let initials = store_initials.user_initials();
                    view! {
                        <span class="avatar-initials">
                            {initials}
                        </span>
                    }.into_any()
                }
            }}
        </div>
    }
}

/// Info card component for displaying user information.
#[component]
fn InfoCard(title: &'static str, value: Signal<String>) -> impl IntoView {
    view! {
        <div class="info-card">
            <h3>{title}</h3>
            <p>{value}</p>
        </div>
    }
}
