// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! UI Components for the Token Explorer Example
//!
//! Displays Solana token data with price changes, stats, and filtering.
//! Features:
//! - SSR data fetching with per-request freshness
//! - Client-side polling every 30 seconds
//! - URL-based search and filtering (works with SSR and CSR)
//! - Shareable URLs that preserve filter state

use leptos::prelude::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    NavigateOptions,
    components::{Route, Router, Routes},
    hooks::{use_navigate, use_query_map},
    path,
};
use leptos_store::prelude::*;

use crate::token_store::{SortField, Token, TokenStore, fetch_tokens};

// ============================================================================
// URL Query Parameter Handling
// ============================================================================

/// Query parameter keys
mod query_keys {
    pub const SEARCH: &str = "q";
    pub const SORT: &str = "sort";
    pub const DIRECTION: &str = "dir";
}

/// Parse SortField from URL query parameter
fn parse_sort_field(value: &str) -> SortField {
    match value.to_lowercase().as_str() {
        "mcap" | "marketcap" => SortField::MarketCap,
        "price" => SortField::Price,
        "change" | "24h" => SortField::PriceChange24h,
        "liq" | "liquidity" => SortField::Liquidity,
        "holders" => SortField::Holders,
        "volume" => SortField::Volume24h,
        _ => SortField::MarketCap, // Default
    }
}

/// Convert SortField to URL query parameter value
fn sort_field_to_param(field: &SortField) -> &'static str {
    match field {
        SortField::MarketCap => "mcap",
        SortField::Price => "price",
        SortField::PriceChange24h => "change",
        SortField::Liquidity => "liq",
        SortField::Holders => "holders",
        SortField::Volume24h => "volume",
    }
}

/// Parse sort direction from URL query parameter
fn parse_sort_direction(value: &str) -> bool {
    // Returns true for descending
    match value.to_lowercase().as_str() {
        "asc" | "a" => false,
        _ => true, // Default to descending
    }
}

/// Convert sort direction to URL query parameter value
fn direction_to_param(desc: bool) -> &'static str {
    if desc { "desc" } else { "asc" }
}

/// Build query string from current filter state
fn build_query_string(search: &str, sort: &SortField, desc: bool) -> String {
    let mut params = Vec::new();

    if !search.is_empty() {
        params.push(format!(
            "{}={}",
            query_keys::SEARCH,
            urlencoding::encode(search)
        ));
    }

    // Only include sort params if not default
    if *sort != SortField::MarketCap || !desc {
        params.push(format!(
            "{}={}",
            query_keys::SORT,
            sort_field_to_param(sort)
        ));
        params.push(format!(
            "{}={}",
            query_keys::DIRECTION,
            direction_to_param(desc)
        ));
    }

    if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    }
}

/// Polling interval in milliseconds (30 seconds)
#[cfg(feature = "hydrate")]
const POLL_INTERVAL_MS: u32 = 30_000;

/// Read hydration data from a script tag in the DOM
#[cfg(feature = "hydrate")]
fn read_hydration_script(store_key: &str) -> Option<String> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window()?;
    let document = window.document()?;
    let script_id = format!("__leptos_store_{}", store_key);
    let element = document.get_element_by_id(&script_id)?;
    let script = element.dyn_into::<web_sys::HtmlScriptElement>().ok()?;
    let text = script.text().ok()?;
    Some(text)
}

/// Format number with thousands separator
fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let bytes: Vec<_> = s.bytes().rev().collect();
    let chunks: Vec<_> = bytes
        .chunks(3)
        .map(|chunk| String::from_utf8(chunk.to_vec()).unwrap())
        .collect();
    chunks.join(",").chars().rev().collect()
}

/// Embeddable demo component for the showcase.
///
/// Sets up the TokenStore and renders an embedded explorer without URL sync.
#[component]
pub fn Demo() -> impl IntoView {
    provide_store(TokenStore::new());
    view! { <TokenExplorerEmbed /> }
}

/// Embedded token explorer without URL parameter sync.
///
/// Keeps fetch, polling, search, sort, and detail panel functionality
/// but removes `use_navigate()` / `use_query_map()` dependencies so it
/// can be mounted inside another router.
#[component]
fn TokenExplorerEmbed() -> impl IntoView {
    let store = use_store::<TokenStore>();

    let (last_updated, set_last_updated) = signal(String::new());
    let (is_refreshing, set_is_refreshing) = signal(false);

    // Fetch tokens on mount
    let tokens_resource = Resource::new(
        || (),
        move |_| async move { fetch_tokens().await },
    );

    {
        let store = store.clone();
        Effect::new(move |_| {
            if let Some(Ok(response)) = tokens_resource.get() {
                store.set_tokens(response.tokens);
                set_last_updated.set(response.fetched_at);
                set_is_refreshing.set(false);
            }
        });
    }

    // Client-side polling every 30 seconds
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;

        let store = store.clone();
        let (interval_id, set_interval_id) = signal::<Option<i32>>(None);

        Effect::new(move |_| {
            let store = store.clone();
            let window = web_sys::window().expect("no global window");

            let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                let store = store.clone();
                set_is_refreshing.set(true);
                leptos::task::spawn_local(async move {
                    match fetch_tokens().await {
                        Ok(response) => {
                            store.set_tokens(response.tokens);
                            set_last_updated.set(response.fetched_at);
                        }
                        Err(e) => {
                            store.set_error(Some(format!("Refresh failed: {}", e)));
                        }
                    }
                    set_is_refreshing.set(false);
                });
            }) as Box<dyn Fn()>);

            let id = window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    POLL_INTERVAL_MS as i32,
                )
                .expect("failed to set interval");

            set_interval_id.set(Some(id));
            callback.forget();
        });

        on_cleanup(move || {
            if let Some(id) = interval_id.get_untracked() {
                if let Some(window) = web_sys::window() {
                    window.clear_interval_with_handle(id);
                }
            }
        });
    }

    // Manual refresh
    let refresh_action = leptos::prelude::Action::new(move |_: &()| {
        let store = store.clone();
        async move {
            set_is_refreshing.set(true);
            match fetch_tokens().await {
                Ok(response) => {
                    store.set_tokens(response.tokens);
                    set_last_updated.set(response.fetched_at);
                }
                Err(e) => {
                    store.set_error(Some(format!("Refresh failed: {e}")));
                }
            }
            set_is_refreshing.set(false);
        }
    });

    // No-op URL updater (embedded mode doesn't sync to URL)
    let noop_url = move |_search: String, _sort: SortField, _desc: bool| {};

    view! {
        <div class="token-explorer">
            <Header
                last_updated=last_updated
                is_refreshing=is_refreshing
                on_refresh=move |_| { let _ = refresh_action.dispatch(()); }
            />
            <SearchAndFilter update_url=noop_url initial_search=String::new() />
            <Suspense fallback=move || view! { <LoadingState /> }>
                <TokenGrid />
            </Suspense>
            <TokenDetail />
        </div>
    }
}

/// Main application shell
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    // On server: Store is already provided by main.rs
    // On client (hydrate): Read serialized state and create store
    #[cfg(feature = "hydrate")]
    {
        use leptos_store::hydration::HydratableStore;

        // Try to hydrate from serialized data, fallback to empty store
        let store = if let Some(data) = read_hydration_script("token_store") {
            TokenStore::from_hydrated_state(&data).unwrap_or_else(|_| TokenStore::new())
        } else {
            TokenStore::new()
        };
        provide_store(store);
    }

    // On CSR (no SSR): just create empty store
    #[cfg(all(not(feature = "hydrate"), not(feature = "ssr")))]
    {
        provide_store(TokenStore::new());
    }

    view! {
        <Stylesheet id="leptos" href="/pkg/token-explorer-example.css"/>
        <Title text="Token Explorer - Solana Tokens"/>
        <Meta name="description" content="Explore Solana tokens with real-time data"/>

        <Router>
            <main class="app">
                <Routes fallback=|| "Page not found">
                    <Route path=path!("/") view=TokenExplorer/>
                </Routes>
            </main>
        </Router>
    }
}

/// Main token explorer page with SSR data fetching, client-side polling,
/// and URL-based search/filtering
#[component]
fn TokenExplorer() -> impl IntoView {
    let store = use_store::<TokenStore>();
    let navigate = use_navigate();
    let query_map = use_query_map();

    // Signal to track last update time
    let (last_updated, set_last_updated) = signal(String::new());
    let (is_refreshing, set_is_refreshing) = signal(false);

    // Read initial filter state from URL query parameters (once, not reactive)
    // This ensures SSR renders the correct filtered list
    let params = query_map.get_untracked();
    let initial_search = params
        .get(query_keys::SEARCH)
        .map(|s| s.to_string())
        .unwrap_or_default();
    let initial_sort = params
        .get(query_keys::SORT)
        .map(|s| parse_sort_field(s.as_str()))
        .unwrap_or(SortField::MarketCap);
    let initial_desc = params
        .get(query_keys::DIRECTION)
        .map(|s| parse_sort_direction(s.as_str()))
        .unwrap_or(true);

    // Initialize store with URL params (non-reactive, runs once)
    store.set_search_query(initial_search.clone());
    store.set_sort_field_direct(initial_sort.clone(), initial_desc);

    // Track the last URL we navigated to, to avoid redundant navigations
    let (last_url, set_last_url) = signal(build_query_string(
        &initial_search,
        &initial_sort,
        initial_desc,
    ));

    // Create a resource that fetches tokens on mount (works for SSR and CSR)
    let tokens_resource = Resource::new(
        || (), // No reactive dependencies - fetch once on mount
        move |_| async move { fetch_tokens().await },
    );

    // Effect to update store when resource loads
    {
        let store = store.clone();
        Effect::new(move |_| {
            if let Some(Ok(response)) = tokens_resource.get() {
                store.set_tokens(response.tokens);
                set_last_updated.set(response.fetched_at);
                set_is_refreshing.set(false);
            }
        });
    }

    // Client-side polling every 30 seconds
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;

        let store = store.clone();
        let (interval_id, set_interval_id) = signal::<Option<i32>>(None);

        Effect::new(move |_| {
            let store = store.clone();

            // Set up the polling interval using web_sys
            let window = web_sys::window().expect("no global window");

            let callback = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
                let store = store.clone();
                set_is_refreshing.set(true);

                // Spawn the async fetch
                leptos::task::spawn_local(async move {
                    match fetch_tokens().await {
                        Ok(response) => {
                            store.set_tokens(response.tokens);
                            set_last_updated.set(response.fetched_at);
                        }
                        Err(e) => {
                            store.set_error(Some(format!("Refresh failed: {}", e)));
                        }
                    }
                    set_is_refreshing.set(false);
                });
            }) as Box<dyn Fn()>);

            let id = window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    POLL_INTERVAL_MS as i32,
                )
                .expect("failed to set interval");

            set_interval_id.set(Some(id));

            // Prevent the closure from being dropped
            callback.forget();
        });

        // Clean up interval on unmount
        on_cleanup(move || {
            if let Some(id) = interval_id.get_untracked() {
                if let Some(window) = web_sys::window() {
                    window.clear_interval_with_handle(id);
                }
            }
        });
    }

    // Manual refresh action
    let refresh_action = leptos::prelude::Action::new(move |_: &()| {
        let store = store.clone();
        async move {
            set_is_refreshing.set(true);
            match fetch_tokens().await {
                Ok(response) => {
                    store.set_tokens(response.tokens);
                    set_last_updated.set(response.fetched_at);
                }
                Err(e) => {
                    store.set_error(Some(format!("Refresh failed: {e}")));
                }
            }
            set_is_refreshing.set(false);
        }
    });

    // Function to update URL with current filter state (avoids redundant navigation)
    let update_url = {
        let navigate = navigate.clone();
        move |search: String, sort: SortField, desc: bool| {
            let query = build_query_string(&search, &sort, desc);

            // Only navigate if URL actually changed
            let current_url = last_url.get_untracked();
            if query != current_url {
                set_last_url.set(query.clone());
                let path = format!("/{query}");
                navigate(
                    &path,
                    NavigateOptions {
                        replace: true, // Don't create new history entry for filter changes
                        ..Default::default()
                    },
                );
            }
        }
    };

    view! {
        <div class="token-explorer">
            <Header
                last_updated=last_updated
                is_refreshing=is_refreshing
                on_refresh=move |_| { let _ = refresh_action.dispatch(()); }
            />
            <SearchAndFilter update_url=update_url.clone() initial_search=initial_search />
            <Suspense fallback=move || view! { <LoadingState /> }>
                <TokenGrid />
            </Suspense>
            <TokenDetail />
        </div>
    }
}

/// Loading state component
#[component]
fn LoadingState() -> impl IntoView {
    view! {
        <div class="loading-state">
            <div class="loading-spinner"></div>
            <p>"Loading tokens..."</p>
        </div>
    }
}

/// Page header with last updated time and refresh button
#[component]
fn Header(
    last_updated: ReadSignal<String>,
    is_refreshing: ReadSignal<bool>,
    on_refresh: impl Fn(()) + 'static,
) -> impl IntoView {
    let store = use_store::<TokenStore>();

    view! {
        <header class="header">
            <div class="header-content">
                <div class="logo">
                    <span class="logo-icon">"◎"</span>
                    <h1>"Token Explorer"</h1>
                </div>
                <div class="header-stats">
                    <span class="stat">
                        <span class="stat-label">"Tokens"</span>
                        <span class="stat-value">{move || store.token_count()}</span>
                    </span>
                    <span class="stat update-info">
                        <span class="stat-label">"Updated"</span>
                        <span class="stat-value">
                            {move || {
                                let updated = last_updated.get();
                                if updated.is_empty() {
                                    "Loading...".to_string()
                                } else {
                                    format_time(&updated)
                                }
                            }}
                        </span>
                    </span>
                    <button
                        class="refresh-btn"
                        class:refreshing=move || is_refreshing.get()
                        on:click=move |_| on_refresh(())
                        disabled=move || is_refreshing.get()
                    >
                        {move || if is_refreshing.get() { "⟳" } else { "↻" }}
                    </button>
                </div>
            </div>
            <div class="poll-indicator">
                <span class="poll-text">"Auto-refresh every 30s"</span>
                {move || if is_refreshing.get() {
                    view! { <span class="refreshing-indicator">" • Refreshing..."</span> }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </div>
        </header>
    }
}

/// Format ISO timestamp to human-readable time
fn format_time(iso: &str) -> String {
    // Extract time portion from ISO format (HH:MM:SS)
    if let Some(t_pos) = iso.find('T') {
        let time_part = &iso[t_pos + 1..];
        if let Some(z_pos) = time_part.find('Z') {
            return time_part[..z_pos].to_string();
        }
        if time_part.len() >= 8 {
            return time_part[..8].to_string();
        }
    }
    iso.to_string()
}

/// Debounce delay for search input (milliseconds)
#[cfg(feature = "hydrate")]
const SEARCH_DEBOUNCE_MS: u32 = 300;

/// Search and filter controls with debounced URL sync
///
/// Uses a debounce pattern inspired by rxRust:
/// - Immediate UI feedback (input updates instantly)
/// - Debounced store/URL updates (waits 300ms after last keystroke)
/// - Distinct until changed (only updates if value actually changed)
#[component]
fn SearchAndFilter<F>(update_url: F, initial_search: String) -> impl IntoView
where
    F: Fn(String, SortField, bool) + Clone + Send + Sync + 'static,
{
    let store = use_store::<TokenStore>();

    // Local signal for search input (immediate UI feedback)
    let (search_input, set_search_input) = signal(initial_search.clone());

    // Track the last committed search (for distinct_until_changed behavior)
    let (last_committed, set_last_committed) = signal(initial_search);

    // Track debounce timer handle for cleanup
    #[cfg(feature = "hydrate")]
    let (timer_handle, set_timer_handle) = signal::<Option<i32>>(None);

    // Clone for different closures
    #[cfg(feature = "hydrate")]
    let store_for_debounce = store.clone();
    #[cfg(feature = "hydrate")]
    let update_url_for_debounce = update_url.clone();
    let store_clear = store.clone();
    let update_url_clear = update_url.clone();

    // Debounced search handler (client-side)
    #[cfg(feature = "hydrate")]
    let trigger_debounced_search = move |value: String| {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::prelude::*;

        // Cancel any pending timer
        if let Some(handle) = timer_handle.get_untracked() {
            if let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(handle);
            }
        }

        // Set up new debounce timer
        let store = store_for_debounce.clone();
        let update_url = update_url_for_debounce.clone();

        let callback = Closure::once(Box::new(move || {
            // distinct_until_changed: only update if value changed
            let last = last_committed.get_untracked();
            if value != last {
                set_last_committed.set(value.clone());

                // Update store
                store.set_search_query(value.clone());

                // Update URL (use untracked to avoid reactive warnings)
                let sort = store.sort_by_untracked();
                let desc = store.is_sort_desc_untracked();
                update_url(value, sort, desc);
            }
        }) as Box<dyn FnOnce()>);

        if let Some(window) = web_sys::window() {
            if let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                SEARCH_DEBOUNCE_MS as i32,
            ) {
                set_timer_handle.set(Some(handle));
            }
        }

        // Prevent closure from being dropped
        callback.forget();
    };

    // SSR fallback - no debounce
    #[cfg(not(feature = "hydrate"))]
    let trigger_debounced_search = {
        let store = store.clone();
        let update_url = update_url.clone();
        move |value: String| {
            // distinct_until_changed
            let last = last_committed.get_untracked();
            if value != last {
                set_last_committed.set(value.clone());
                store.set_search_query(value.clone());
                let sort = store.sort_by_untracked();
                let desc = store.is_sort_desc_untracked();
                update_url(value, sort, desc);
            }
        }
    };

    view! {
        <div class="controls">
            <div class="search-box">
                <input
                    type="text"
                    placeholder="Search tokens by name, symbol, or address..."
                    prop:value=move || search_input.get()
                    on:input:target={
                        let trigger = trigger_debounced_search.clone();
                        move |ev| {
                            let value = ev.target().value();
                            // Immediate UI update
                            set_search_input.set(value.clone());
                            // Debounced store/URL update
                            trigger(value);
                        }
                    }
                />
                <button
                    class="clear-search"
                    class:hidden=move || search_input.get().is_empty()
                    on:click={
                        let store_for_btn = store_clear.clone();
                        let update_url_for_btn = update_url_clear.clone();
                        move |_| {
                            set_search_input.set(String::new());
                            set_last_committed.set(String::new());
                            store_for_btn.set_search_query(String::new());

                            let sort = store_for_btn.sort_by_untracked();
                            let desc = store_for_btn.is_sort_desc_untracked();
                            update_url_for_btn(String::new(), sort, desc);
                        }
                    }
                >
                    "×"
                </button>
            </div>
            <div class="sort-buttons">
                <SortButton field=SortField::MarketCap label="MCap" update_url=update_url.clone() />
                <SortButton field=SortField::PriceChange24h label="24h %" update_url=update_url.clone() />
                <SortButton field=SortField::Liquidity label="Liq" update_url=update_url.clone() />
                <SortButton field=SortField::Holders label="Holders" update_url=update_url.clone() />
            </div>
        </div>
    }
}

/// Sort button component with URL sync
#[component]
fn SortButton<F>(field: SortField, label: &'static str, update_url: F) -> impl IntoView
where
    F: Fn(String, SortField, bool) + Clone + Send + Sync + 'static,
{
    let store = use_store::<TokenStore>();
    let field_clone = field.clone();
    let field_for_click = field.clone();
    let field_for_indicator = field.clone();

    // Clone store for different closures
    let store_active = store.clone();
    let store_click = store.clone();
    let store_indicator = store.clone();

    view! {
        <button
            class="sort-btn"
            class:active=move || store_active.sort_by() == field_clone.clone()
            on:click={
                let update_url = update_url.clone();
                move |_| {
                    // Toggle or set sort (use untracked to avoid reactive warnings)
                    let current_sort = store_click.sort_by_untracked();
                    let current_desc = store_click.is_sort_desc_untracked();

                    let (new_sort, new_desc) = if current_sort == field_for_click {
                        // Toggle direction
                        (field_for_click.clone(), !current_desc)
                    } else {
                        // New field, default to descending
                        (field_for_click.clone(), true)
                    };

                    // Update store
                    store_click.set_sort_field_direct(new_sort.clone(), new_desc);

                    // Update URL (use untracked for search query)
                    update_url(store_click.search_query_untracked(), new_sort, new_desc);
                }
            }
        >
            {label}
            {move || {
                if store_indicator.sort_by() == field_for_indicator.clone() {
                    if store_indicator.is_sort_desc() { " ↓" } else { " ↑" }
                } else {
                    ""
                }
            }}
        </button>
    }
}

/// Token grid display
#[component]
fn TokenGrid() -> impl IntoView {
    let store = use_store::<TokenStore>();

    view! {
        <div class="token-grid">
            {move || {
                let tokens = store.filtered_tokens();
                if tokens.is_empty() {
                    view! {
                        <div class="empty-state">
                            <p>"No tokens found"</p>
                        </div>
                    }.into_any()
                } else {
                    tokens.into_iter().map(|token| {
                        view! { <TokenCard token=token /> }
                    }).collect_view().into_any()
                }
            }}
        </div>
    }
}

/// Individual token card
#[component]
fn TokenCard(token: Token) -> impl IntoView {
    let store = use_store::<TokenStore>();
    let token_id = token.id.clone();

    let price_change_24h = token.price_change_24h();
    let price_change_1h = token.price_change_1h();
    let is_positive_24h = price_change_24h >= 0.0;
    let is_positive_1h = price_change_1h >= 0.0;

    let icon_url = token.icon.clone().unwrap_or_default();
    let has_icon = !icon_url.is_empty();

    let token_name = token.name.clone();
    let token_symbol = token.symbol.clone();
    let token_symbol_icon = token.symbol.clone();
    let formatted_price = token.formatted_price();
    let formatted_mcap = token.formatted_mcap();
    let formatted_liquidity = token.formatted_liquidity();
    let holder_count = format!("{}", token.holder_count);
    let short_address = token.short_address();
    let full_id = token.id.clone();
    let launchpad = token.launchpad.clone();
    let is_verified = token.is_verified();

    view! {
        <div
            class="token-card"
            on:click=move |_| store.select_token(Some(token_id.clone()))
        >
            <div class="token-header">
                <div class="token-icon">
                    {if has_icon {
                        view! { <img src=icon_url.clone() alt=token_symbol_icon.clone() /> }.into_any()
                    } else {
                        let first_char = token_symbol_icon.chars().next().unwrap_or('?').to_string();
                        view! { <span class="token-icon-placeholder">{first_char}</span> }.into_any()
                    }}
                </div>
                <div class="token-info">
                    <h3 class="token-name">{token_name}</h3>
                    <span class="token-symbol">{token_symbol}</span>
                </div>
                {if is_verified {
                    view! { <span class="verified-badge" title="Verified">"✓"</span> }.into_any()
                } else {
                    view! { <span></span> }.into_any()
                }}
            </div>

            <div class="token-price">
                <span class="price">{formatted_price}</span>
                <span
                    class="price-change"
                    class:positive=is_positive_24h
                    class:negative=!is_positive_24h
                >
                    {if is_positive_24h { "+" } else { "" }}
                    {format!("{price_change_24h:.2}%")}
                </span>
            </div>

            <div class="token-stats">
                <div class="stat-row">
                    <span class="stat-label">"MCap"</span>
                    <span class="stat-value">{formatted_mcap}</span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">"Liquidity"</span>
                    <span class="stat-value">{formatted_liquidity}</span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">"Holders"</span>
                    <span class="stat-value">{holder_count}</span>
                </div>
                <div class="stat-row">
                    <span class="stat-label">"1h"</span>
                    <span
                        class="stat-value"
                        class:positive=is_positive_1h
                        class:negative=!is_positive_1h
                    >
                        {if is_positive_1h { "+" } else { "" }}
                        {format!("{price_change_1h:.2}%")}
                    </span>
                </div>
            </div>

            <div class="token-footer">
                <span class="token-address" title=full_id>{short_address}</span>
                {launchpad.map(|lp| {
                    view! { <span class="launchpad-badge">{lp}</span> }
                })}
            </div>
        </div>
    }
}

/// Token detail modal/panel
#[component]
fn TokenDetail() -> impl IntoView {
    let store = use_store::<TokenStore>();
    let store_close = store.clone();
    let store_close2 = store.clone();

    view! {
        {move || {
            store.selected_token().map(|token| {
                let price_change_24h = token.price_change_24h();
                let is_positive = price_change_24h >= 0.0;
                let icon_url = token.icon.clone().unwrap_or_default();
                let has_icon = !icon_url.is_empty();

                // Pre-compute all values to avoid reference issues
                let token_name = token.name.clone();
                let token_symbol = token.symbol.clone();
                let token_symbol_icon = token.symbol.clone();
                let formatted_price = token.formatted_price();
                let formatted_mcap = token.formatted_mcap();
                let formatted_liquidity = token.formatted_liquidity();
                let holder_count = format_with_commas(token.holder_count);
                let total_supply = format!("{:.0}", token.total_supply);
                let token_id = token.id.clone();
                let twitter = token.twitter.clone();
                let website = token.website.clone();
                let stats_24h = token.stats_24h.clone();
                let audit = token.audit.clone();

                let store_close_inner = store_close.clone();
                let store_close_btn = store_close2.clone();

                view! {
                    <div class="token-detail-overlay" on:click=move |_| store_close_inner.clear_selection()>
                        <div class="token-detail" on:click=|ev| ev.stop_propagation()>
                            <button class="close-btn" on:click=move |_| store_close_btn.clear_selection()>"×"</button>

                            <div class="detail-header">
                                <div class="detail-icon">
                                    {if has_icon {
                                        view! { <img src=icon_url.clone() alt=token_symbol_icon.clone() /> }.into_any()
                                    } else {
                                        let chars: String = token_symbol_icon.chars().take(2).collect();
                                        view! { <span class="icon-placeholder">{chars}</span> }.into_any()
                                    }}
                                </div>
                                <div class="detail-title">
                                    <h2>{token_name}</h2>
                                    <span class="symbol">{token_symbol}</span>
                                </div>
                            </div>

                            <div class="detail-price">
                                <span class="big-price">{formatted_price}</span>
                                <span
                                    class="big-change"
                                    class:positive=is_positive
                                    class:negative=!is_positive
                                >
                                    {if is_positive { "▲" } else { "▼" }}
                                    {format!(" {:.2}% (24h)", price_change_24h.abs())}
                                </span>
                            </div>

                            <div class="detail-grid">
                                <div class="detail-stat">
                                    <span class="label">"Market Cap"</span>
                                    <span class="value">{formatted_mcap}</span>
                                </div>
                                <div class="detail-stat">
                                    <span class="label">"Liquidity"</span>
                                    <span class="value">{formatted_liquidity}</span>
                                </div>
                                <div class="detail-stat">
                                    <span class="label">"Holders"</span>
                                    <span class="value">{holder_count}</span>
                                </div>
                                <div class="detail-stat">
                                    <span class="label">"Total Supply"</span>
                                    <span class="value">{total_supply}</span>
                                </div>
                            </div>

                            {stats_24h.as_ref().map(|stats| {
                                let buy_vol = format!("${:.0}", stats.buy_volume);
                                let sell_vol = format!("${:.0}", stats.sell_volume);
                                let num_buys = stats.num_buys;
                                let num_sells = stats.num_sells;
                                view! {
                                    <div class="detail-section">
                                        <h3>"24h Trading Activity"</h3>
                                        <div class="detail-grid">
                                            <div class="detail-stat">
                                                <span class="label">"Buy Volume"</span>
                                                <span class="value positive">{buy_vol}</span>
                                            </div>
                                            <div class="detail-stat">
                                                <span class="label">"Sell Volume"</span>
                                                <span class="value negative">{sell_vol}</span>
                                            </div>
                                            <div class="detail-stat">
                                                <span class="label">"Buys"</span>
                                                <span class="value">{num_buys}</span>
                                            </div>
                                            <div class="detail-stat">
                                                <span class="label">"Sells"</span>
                                                <span class="value">{num_sells}</span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            })}

                            {audit.as_ref().map(|audit| {
                                let mint_disabled = audit.mint_authority_disabled;
                                let freeze_disabled = audit.freeze_authority_disabled;
                                let top_holders = format!("{:.2}%", audit.top_holders_percentage);
                                let dev_balance = format!("{:.4}%", audit.dev_balance_percentage);
                                view! {
                                    <div class="detail-section">
                                        <h3>"Security Audit"</h3>
                                        <div class="audit-badges">
                                            <span class="audit-badge" class:safe=mint_disabled>
                                                {if mint_disabled { "✓ Mint Disabled" } else { "⚠ Mint Active" }}
                                            </span>
                                            <span class="audit-badge" class:safe=freeze_disabled>
                                                {if freeze_disabled { "✓ Freeze Disabled" } else { "⚠ Freeze Active" }}
                                            </span>
                                        </div>
                                        <div class="detail-grid">
                                            <div class="detail-stat">
                                                <span class="label">"Top Holders %"</span>
                                                <span class="value">{top_holders}</span>
                                            </div>
                                            <div class="detail-stat">
                                                <span class="label">"Dev Balance %"</span>
                                                <span class="value">{dev_balance}</span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            })}

                            <div class="detail-footer">
                                <code class="full-address">{token_id}</code>
                                <div class="detail-links">
                                    {twitter.map(|url| {
                                        view! {
                                            <a href=url target="_blank" class="link-btn">"Twitter"</a>
                                        }
                                    })}
                                    {website.map(|url| {
                                        view! {
                                            <a href=url target="_blank" class="link-btn">"Website"</a>
                                        }
                                    })}
                                </div>
                            </div>
                        </div>
                    </div>
                }
            })
        }}
    }
}
