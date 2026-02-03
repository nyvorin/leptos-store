// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Devtools integration for leptos-store.
//!
//! This module provides three tiers of debugging support:
//!
//! 1. **Console API**: Exposes `window.__LEPTOS_STORE__` for browser console access
//! 2. **WASM Inspector**: A floating debug panel component
//! 3. **Browser Extension Protocol**: Redux DevTools compatible messaging
//!
//! # Feature
//!
//! Requires the `devtools` feature.
//!
//! # Console API
//!
//! ```javascript
//! // In browser console:
//! __LEPTOS_STORE__.getStores()     // List all registered stores
//! __LEPTOS_STORE__.getState("auth") // Get state of a specific store
//! __LEPTOS_STORE__.subscribe(cb)    // Subscribe to state changes
//! ```

use crate::middleware::{EventSubscriber, StoreEvent};
use crate::store::Store;
use leptos::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// ============================================================================
// Devtools State
// ============================================================================

/// Global devtools state.
static DEVTOOLS: std::sync::OnceLock<Arc<RwLock<DevtoolsState>>> = std::sync::OnceLock::new();

/// State getter function type (WASM only - uses Rc since signals aren't Send+Sync).
#[cfg(target_arch = "wasm32")]
type StateGetter = std::rc::Rc<dyn Fn() -> String>;

/// Thread-local storage for state getters (WASM only).
/// Kept separate from DEVTOOLS because Rc is not Sync.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static STATE_GETTERS: std::cell::RefCell<HashMap<String, StateGetter>> = std::cell::RefCell::new(HashMap::new());
}

/// Internal devtools state (must be Sync for static storage).
#[derive(Default)]
struct DevtoolsState {
    /// Registered stores by name.
    stores: HashMap<String, StoreInfo>,
    /// Event history.
    events: Vec<DevtoolsEvent>,
    /// Maximum events to keep.
    max_events: usize,
    /// Whether devtools is enabled.
    enabled: bool,
    /// Subscribers for state changes.
    subscribers: Vec<DevtoolsSubscriber>,
}

/// Information about a registered store.
#[derive(Clone, Debug)]
pub struct StoreInfo {
    /// Store name.
    pub name: String,
    /// Store key (for lookup).
    pub key: String,
    /// Type name.
    pub type_name: &'static str,
    /// When registered.
    pub registered_at: u64,
}

/// A devtools event.
#[derive(Clone, Debug)]
pub struct DevtoolsEvent {
    /// Event type.
    pub event_type: String,
    /// Store name.
    pub store_name: Option<String>,
    /// Event payload (JSON).
    pub payload: String,
    /// Timestamp.
    pub timestamp: u64,
}

/// A devtools subscriber.
type DevtoolsSubscriber = Box<dyn Fn(&DevtoolsEvent) + Send + Sync>;

// ============================================================================
// Devtools Initialization
// ============================================================================

/// Initialize the devtools system.
///
/// Call this once at application startup to enable devtools.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::devtools::*;
///
/// fn main() {
///     init_devtools(DevtoolsConfig::default());
///     // ... rest of app
/// }
/// ```
pub fn init_devtools(config: DevtoolsConfig) {
    let state = DevtoolsState {
        stores: HashMap::new(),
        events: Vec::new(),
        max_events: config.max_events,
        enabled: config.enabled,
        subscribers: Vec::new(),
    };

    let _ = DEVTOOLS.set(Arc::new(RwLock::new(state)));

    #[cfg(target_arch = "wasm32")]
    if config.enabled && config.expose_console_api {
        init_console_api();
    }
}

/// Configuration for devtools.
#[derive(Debug, Clone)]
pub struct DevtoolsConfig {
    /// Whether devtools is enabled.
    pub enabled: bool,
    /// Maximum events to keep in history.
    pub max_events: usize,
    /// Whether to expose the console API.
    pub expose_console_api: bool,
    /// Whether to connect to browser extension.
    pub connect_extension: bool,
}

impl Default for DevtoolsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_events: 1000,
            expose_console_api: true,
            connect_extension: true,
        }
    }
}

// ============================================================================
// Console API (Tier 1)
// ============================================================================

/// Initialize the console API (WASM only).
#[cfg(target_arch = "wasm32")]
fn init_console_api() {
    use wasm_bindgen::JsCast;

    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };

    // Create the API object
    let api = js_sys::Object::new();

    // Add getStores function - returns a JS object with store info
    let get_stores = Closure::wrap(Box::new(|| -> JsValue {
        match get_devtools_state() {
            Some(state) => {
                let result = js_sys::Object::new();
                for info in state.stores.values() {
                    let store_obj = js_sys::Object::new();
                    js_sys::Reflect::set(&store_obj, &JsValue::from_str("name"), &JsValue::from_str(&info.name)).ok();
                    js_sys::Reflect::set(&store_obj, &JsValue::from_str("type"), &JsValue::from_str(info.type_name)).ok();
                    js_sys::Reflect::set(&store_obj, &JsValue::from_str("registeredAt"), &JsValue::from_f64(info.registered_at as f64)).ok();
                    js_sys::Reflect::set(&result, &JsValue::from_str(&info.key), &store_obj).ok();
                }
                result.into()
            }
            None => JsValue::from_str("Devtools not initialized"),
        }
    }) as Box<dyn Fn() -> JsValue>);

    js_sys::Reflect::set(
        &api,
        &JsValue::from_str("getStores"),
        get_stores.as_ref().unchecked_ref(),
    )
    .ok();
    get_stores.forget();

    // Add getState function - returns a JS object with state info
    let get_state = Closure::wrap(Box::new(|key: String| -> JsValue {
        // Try to get state from thread_local getters
        let state_result = STATE_GETTERS.with(|getters| {
            getters.borrow().get(&key).map(|getter| getter())
        });
        
        match state_result {
            Some(state_str) => {
                let obj = js_sys::Object::new();
                js_sys::Reflect::set(&obj, &JsValue::from_str("key"), &JsValue::from_str(&key)).ok();
                
                // Add type info if available from devtools state
                if let Some(devtools_state) = get_devtools_state() {
                    if let Some(info) = devtools_state.stores.get(&key) {
                        js_sys::Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str(info.type_name)).ok();
                        js_sys::Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str(&info.name)).ok();
                    }
                }
                
                // Get state - check if it's JSON or Debug formatted
                if let Some(json_str) = state_str.strip_prefix("JSON:") {
                    // Parse as JSON for proper JS object
                    if let Ok(parsed) = js_sys::JSON::parse(json_str) {
                        js_sys::Reflect::set(&obj, &JsValue::from_str("state"), &parsed).ok();
                    } else {
                        js_sys::Reflect::set(&obj, &JsValue::from_str("state"), &JsValue::from_str(json_str)).ok();
                    }
                } else {
                    // Debug formatted string
                    js_sys::Reflect::set(&obj, &JsValue::from_str("state"), &JsValue::from_str(&state_str)).ok();
                }
                obj.into()
            }
            None => {
                // Return an error object
                let err = js_sys::Object::new();
                js_sys::Reflect::set(&err, &JsValue::from_str("error"), &JsValue::from_str("Store not found")).ok();
                js_sys::Reflect::set(&err, &JsValue::from_str("key"), &JsValue::from_str(&key)).ok();
                let available = js_sys::Array::new();
                if let Some(devtools_state) = get_devtools_state() {
                    for k in devtools_state.stores.keys() {
                        available.push(&JsValue::from_str(k));
                    }
                }
                js_sys::Reflect::set(&err, &JsValue::from_str("available"), &available).ok();
                err.into()
            }
        }
    }) as Box<dyn Fn(String) -> JsValue>);

    js_sys::Reflect::set(
        &api,
        &JsValue::from_str("getState"),
        get_state.as_ref().unchecked_ref(),
    )
    .ok();
    get_state.forget();

    // Add getEvents function - returns a JS array of event objects
    let get_events = Closure::wrap(Box::new(|count: Option<u32>| -> JsValue {
        let count = count.unwrap_or(10) as usize;
        match get_devtools_state() {
            Some(state) => {
                let arr = js_sys::Array::new();
                for event in state.events.iter().rev().take(count) {
                    let obj = js_sys::Object::new();
                    js_sys::Reflect::set(&obj, &JsValue::from_str("type"), &JsValue::from_str(&event.event_type)).ok();
                    js_sys::Reflect::set(&obj, &JsValue::from_str("store"), &JsValue::from_str(event.store_name.as_deref().unwrap_or("-"))).ok();
                    js_sys::Reflect::set(&obj, &JsValue::from_str("timestamp"), &JsValue::from_f64(event.timestamp as f64)).ok();
                    // Parse payload as JSON if possible
                    if let Ok(payload) = js_sys::JSON::parse(&event.payload) {
                        js_sys::Reflect::set(&obj, &JsValue::from_str("payload"), &payload).ok();
                    } else {
                        js_sys::Reflect::set(&obj, &JsValue::from_str("payload"), &JsValue::from_str(&event.payload)).ok();
                    }
                    arr.push(&obj);
                }
                arr.into()
            }
            None => JsValue::from_str("Devtools not initialized"),
        }
    }) as Box<dyn Fn(Option<u32>) -> JsValue>);

    js_sys::Reflect::set(
        &api,
        &JsValue::from_str("getEvents"),
        get_events.as_ref().unchecked_ref(),
    )
    .ok();
    get_events.forget();

    // Add help function
    let help = Closure::wrap(Box::new(|| -> JsValue {
        JsValue::from_str(
            r#"Leptos Store Devtools
=====================
Commands:
  getStores()      - List all registered stores
  getState(key)    - Get state of a specific store
  getEvents(count) - Get recent events (default: 10)
  help()           - Show this help message
"#,
        )
    }) as Box<dyn Fn() -> JsValue>);

    js_sys::Reflect::set(
        &api,
        &JsValue::from_str("help"),
        help.as_ref().unchecked_ref(),
    )
    .ok();
    help.forget();

    // Attach to window
    js_sys::Reflect::set(&window, &JsValue::from_str("__LEPTOS_STORE__"), &api).ok();

    leptos::logging::log!(
        "[Devtools] Console API initialized. Use __LEPTOS_STORE__.help() for commands."
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
fn init_console_api() {
    // No-op on non-WASM
}

/// Get the devtools state.
fn get_devtools_state() -> Option<std::sync::RwLockReadGuard<'static, DevtoolsState>> {
    DEVTOOLS.get().and_then(|s| s.read().ok())
}

/// Get mutable devtools state.
fn get_devtools_state_mut() -> Option<std::sync::RwLockWriteGuard<'static, DevtoolsState>> {
    DEVTOOLS.get().and_then(|s| s.write().ok())
}

// ============================================================================
// Store Registration
// ============================================================================

/// Register a store with devtools using Debug formatting.
///
/// The store's state will be accessible via `__LEPTOS_STORE__.getState("key")` 
/// in the browser console. State is shown as a formatted Debug string.
///
/// For proper JSON object output, use [`register_store_json`] instead.
pub fn register_store<S: Store>(store: &S, key: impl Into<String>)
where
    S::State: std::fmt::Debug + 'static,
{
    let key = key.into();
    let store_name = store.name().to_string();
    
    if let Some(mut devtools_state) = get_devtools_state_mut() {
        let info = StoreInfo {
            name: store_name.clone(),
            key: key.clone(),
            type_name: std::any::type_name::<S>(),
            registered_at: current_timestamp_ms(),
        };
        devtools_state.stores.insert(info.key.clone(), info);
        
        // Record a registration event
        devtools_state.events.push(DevtoolsEvent {
            event_type: "StoreRegistered".to_string(),
            store_name: Some(store_name.clone()),
            payload: format!(r#"{{"key":"{}"}}"#, key),
            timestamp: current_timestamp_ms(),
        });
        
    }
    
    // Store a Debug-based state getter (WASM only, in thread_local)
    #[cfg(target_arch = "wasm32")]
    {
        let state_signal = store.state();
        let getter: StateGetter = std::rc::Rc::new(move || {
            format!("{:#?}", state_signal.get_untracked())
        });
        STATE_GETTERS.with(|getters| {
            getters.borrow_mut().insert(key.clone(), getter);
        });
    }
    
    // Set up automatic state change tracking with snapshots (WASM only)
    #[cfg(target_arch = "wasm32")]
    {
        let state_signal = store.state();
        let key_for_effect = key.clone();
        let name_for_effect = store_name;
        Effect::new(move |prev_state: Option<String>| {
            // Get current state as Debug string
            let current_state = format!("{:#?}", state_signal.get());
            
            // Record event with old and new state (skip first run)
            if let Some(old_state) = prev_state {
                // Escape strings for JSON
                let old_escaped = old_state.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                let new_escaped = current_state.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                
                record_event(DevtoolsEvent {
                    event_type: "StateChanged".to_string(),
                    store_name: Some(name_for_effect.clone()),
                    payload: format!(
                        r#"{{"store":"{}","old":"{}","new":"{}"}}"#,
                        key_for_effect, old_escaped, new_escaped
                    ),
                    timestamp: current_timestamp_ms(),
                });
            }
            
            current_state
        });
    }
}

/// Register a store with devtools using JSON serialization.
///
/// The store's state will be accessible via `__LEPTOS_STORE__.getState("key")` 
/// in the browser console as a proper JavaScript object.
///
/// Requires `State: Serialize`. For types that only implement `Debug`, 
/// use [`register_store`] instead.
pub fn register_store_json<S: Store>(store: &S, key: impl Into<String>)
where
    S::State: serde::Serialize + 'static,
{
    let key = key.into();
    let store_name = store.name().to_string();
    
    if let Some(mut devtools_state) = get_devtools_state_mut() {
        let info = StoreInfo {
            name: store_name.clone(),
            key: key.clone(),
            type_name: std::any::type_name::<S>(),
            registered_at: current_timestamp_ms(),
        };
        devtools_state.stores.insert(info.key.clone(), info);
        
        // Record a registration event
        devtools_state.events.push(DevtoolsEvent {
            event_type: "StoreRegistered".to_string(),
            store_name: Some(store_name.clone()),
            payload: format!(r#"{{"key":"{}"}}"#, key),
            timestamp: current_timestamp_ms(),
        });
        
    }
    
    // Store a JSON-based state getter (WASM only, in thread_local)
    #[cfg(target_arch = "wasm32")]
    {
        let state_signal = store.state();
        let getter: StateGetter = std::rc::Rc::new(move || {
            format!("JSON:{}", serde_json::to_string(&state_signal.get_untracked()).unwrap_or_else(|_| "{}".to_string()))
        });
        STATE_GETTERS.with(|getters| {
            getters.borrow_mut().insert(key.clone(), getter);
        });
    }
    
    // Set up automatic state change tracking with JSON snapshots (WASM only)
    #[cfg(target_arch = "wasm32")]
    {
        let state_signal = store.state();
        let key_for_effect = key.clone();
        let name_for_effect = store_name;
        Effect::new(move |prev_json: Option<String>| {
            // Serialize current state to JSON
            let current_json = serde_json::to_string(&state_signal.get())
                .unwrap_or_else(|_| "{}".to_string());
            
            // Record event with old and new JSON state (skip first run)
            if let Some(old_json) = prev_json {
                record_event(DevtoolsEvent {
                    event_type: "StateChanged".to_string(),
                    store_name: Some(name_for_effect.clone()),
                    payload: format!(
                        r#"{{"store":"{}","old":{},"new":{}}}"#,
                        key_for_effect, old_json, current_json
                    ),
                    timestamp: current_timestamp_ms(),
                });
            }
            
            current_json
        });
    }
}

/// Unregister a store from devtools.
pub fn unregister_store(key: &str) {
    if let Some(mut state) = get_devtools_state_mut() {
        state.stores.remove(key);
    }
}

/// Record a devtools event.
pub fn record_event(event: DevtoolsEvent) {
    if let Some(mut state) = get_devtools_state_mut() {
        if !state.enabled {
            return;
        }

        state.events.push(event.clone());

        // Trim if over limit
        if state.events.len() > state.max_events {
            state.events.remove(0);
        }

        // Notify subscribers
        for subscriber in &state.subscribers {
            subscriber(&event);
        }
    }
}

// ============================================================================
// Event Bus Integration
// ============================================================================

/// Devtools event subscriber that records events.
pub struct DevtoolsEventSubscriber;

impl EventSubscriber for DevtoolsEventSubscriber {
    fn on_event(&self, event: &StoreEvent) {
        let devtools_event = match event {
            StoreEvent::StateChanged {
                store_id: _,
                store_name,
                timestamp,
            } => DevtoolsEvent {
                event_type: "StateChanged".to_string(),
                store_name: Some(store_name.to_string()),
                payload: "{}".to_string(),
                timestamp: *timestamp,
            },
            StoreEvent::MutationStarted {
                store_id: _,
                name,
                timestamp,
            } => DevtoolsEvent {
                event_type: "MutationStarted".to_string(),
                store_name: Some(name.to_string()),
                payload: format!(r#"{{"mutation":"{}"}}"#, name),
                timestamp: *timestamp,
            },
            StoreEvent::MutationCompleted {
                store_id: _,
                name,
                duration_ms,
                success,
            } => DevtoolsEvent {
                event_type: "MutationCompleted".to_string(),
                store_name: Some(name.to_string()),
                payload: format!(
                    r#"{{"mutation":"{}","duration_ms":{},"success":{}}}"#,
                    name, duration_ms, success
                ),
                timestamp: current_timestamp_ms(),
            },
            StoreEvent::ActionDispatched {
                store_id: _,
                action_name,
                timestamp,
                ..
            } => DevtoolsEvent {
                event_type: "ActionDispatched".to_string(),
                store_name: Some(action_name.to_string()),
                payload: format!(r#"{{"action":"{}"}}"#, action_name),
                timestamp: *timestamp,
            },
            StoreEvent::ActionCompleted {
                store_id: _,
                action_name,
                duration_ms,
                success,
            } => DevtoolsEvent {
                event_type: "ActionCompleted".to_string(),
                store_name: Some(action_name.to_string()),
                payload: format!(
                    r#"{{"action":"{}","duration_ms":{},"success":{}}}"#,
                    action_name, duration_ms, success
                ),
                timestamp: current_timestamp_ms(),
            },
            StoreEvent::Error {
                store_id: _,
                message,
                source,
            } => DevtoolsEvent {
                event_type: "Error".to_string(),
                store_name: None,
                payload: format!(r#"{{"message":"{}","source":"{:?}"}}"#, message, source),
                timestamp: current_timestamp_ms(),
            },
        };

        record_event(devtools_event);
    }

    fn name(&self) -> &'static str {
        "DevtoolsEventSubscriber"
    }
}

// ============================================================================
// Store Inspector Component (Tier 2)
// ============================================================================

/// A floating debug panel for inspecting store state.
///
/// # Example
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_store::devtools::*;
///
/// #[component]
/// fn App() -> impl IntoView {
///     view! {
///         <StoreInspector />
///         <MainContent />
///     }
/// }
/// ```
#[component]
pub fn StoreInspector(
    /// Whether collapsed by default.
    #[prop(optional, default = false)]
    collapsed: bool,
    /// Maximum events to show.
    #[prop(optional, default = 100)]
    max_events: usize,
) -> impl IntoView {
    let is_collapsed = RwSignal::new(collapsed);
    let selected_store = RwSignal::new(Option::<String>::None);
    let selected_event = RwSignal::new(Option::<usize>::None);
    let events = RwSignal::new(Vec::<DevtoolsEvent>::new());
    let active_tab = RwSignal::new("state"); // "state" or "events"
    let refresh_counter = RwSignal::new(0u32);

    // Get store list reactively
    let stores = move || {
        let _ = refresh_counter.get(); // Track refreshes
        get_devtools_state()
            .map(|s| s.stores.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    };

    // Auto-select first store if none selected
    Effect::new(move |_| {
        if selected_store.get().is_none() {
            if let Some(first) = stores().first() {
                selected_store.set(Some(first.key.clone()));
            }
        }
    });

    // Get current state for selected store (reactive via thread_local on WASM)
    #[cfg(target_arch = "wasm32")]
    let get_current_state = move || -> Option<String> {
        let _ = refresh_counter.get(); // Track refreshes
        let key = selected_store.get()?;
        STATE_GETTERS.with(|getters| {
            getters.borrow().get(&key).map(|getter| getter())
        })
    };
    
    #[cfg(not(target_arch = "wasm32"))]
    let get_current_state = move || -> Option<String> {
        None
    };

    // Update events
    let update_events = move || {
        if let Some(state) = get_devtools_state() {
            events.set(
                state
                    .events
                    .iter()
                    .rev()
                    .take(max_events)
                    .cloned()
                    .collect(),
            );
        }
        refresh_counter.update(|c| *c = c.wrapping_add(1));
    };

    // Auto-refresh events on tab switch
    Effect::new(move |_| {
        if active_tab.get() == "events" {
            update_events();
        }
    });

    // CSS styles as a const for cleaner code
    const PANEL_STYLE: &str = "position: fixed; bottom: 0; right: 0; z-index: 99999; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, monospace; font-size: 13px; width: 500px; max-height: 70vh; display: flex; flex-direction: column; box-shadow: -2px -2px 10px rgba(0,0,0,0.3); border-radius: 8px 0 0 0;";
    const HEADER_STYLE: &str = "background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%); color: #fff; padding: 12px 16px; cursor: pointer; display: flex; justify-content: space-between; align-items: center; border-radius: 8px 0 0 0; user-select: none;";
    const TAB_STYLE: &str = "background: #1f2937; display: flex; border-bottom: 1px solid #374151;";
    const TAB_BTN_STYLE: &str = "flex: 1; padding: 10px; border: none; cursor: pointer; font-size: 13px; transition: all 0.2s;";
    const CONTENT_STYLE: &str = "background: #111827; color: #e5e7eb; flex: 1; overflow: auto; min-height: 300px;";
    
    view! {
        <div
            class="leptos-store-inspector"
            style=move || if is_collapsed.get() {
                "position: fixed; bottom: 20px; right: 20px; z-index: 99999;"
            } else {
                PANEL_STYLE
            }
        >
            // Header bar
            <div
                style=HEADER_STYLE
                on:click=move |_| is_collapsed.update(|c| *c = !*c)
            >
                <div style="display: flex; align-items: center; gap: 8px;">
                    <span style="font-size: 16px;">"🔍"</span>
                    <span style="font-weight: 600;">"Leptos Store Inspector"</span>
                </div>
                <div style="display: flex; align-items: center; gap: 12px;">
                    <span style="font-size: 11px; color: #9ca3af;">
                        {move || format!("{} store(s)", stores().len())}
                    </span>
                    <span style="font-size: 18px;">{move || if is_collapsed.get() { "◀" } else { "▼" }}</span>
                </div>
            </div>

            // Content panel
            <Show when=move || !is_collapsed.get()>
                // Tab bar
                <div style=TAB_STYLE>
                    <button
                        style=move || format!(
                            "{} background: {}; color: {};",
                            TAB_BTN_STYLE,
                            if active_tab.get() == "state" { "#374151" } else { "transparent" },
                            if active_tab.get() == "state" { "#60a5fa" } else { "#9ca3af" }
                        )
                        on:click=move |_| active_tab.set("state")
                    >
                        "📊 State"
                    </button>
                    <button
                        style=move || format!(
                            "{} background: {}; color: {};",
                            TAB_BTN_STYLE,
                            if active_tab.get() == "events" { "#374151" } else { "transparent" },
                            if active_tab.get() == "events" { "#60a5fa" } else { "#9ca3af" }
                        )
                        on:click=move |_| { active_tab.set("events"); update_events(); }
                    >
                        "📜 Events"
                        <span style="margin-left: 6px; background: #4b5563; padding: 2px 6px; border-radius: 10px; font-size: 11px;">
                            {move || events.get().len()}
                        </span>
                    </button>
                </div>

                <div style=CONTENT_STYLE>
                    // State Tab
                    <Show when=move || active_tab.get() == "state">
                        <div style="display: flex; height: 100%;">
                            // Store list sidebar
                            <div style="width: 140px; border-right: 1px solid #374151; padding: 8px 0; overflow-y: auto;">
                                <div style="padding: 4px 12px; color: #6b7280; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px;">
                                    "Stores"
                                </div>
                                {move || {
                                    stores()
                                        .into_iter()
                                        .map(|store| {
                                            let key = store.key.clone();
                                            let key2 = store.key.clone();
                                            let is_selected = move || selected_store.get().as_ref() == Some(&key);
                                            view! {
                                                <div
                                                    style=move || format!(
                                                        "padding: 8px 12px; cursor: pointer; transition: all 0.15s; {}",
                                                        if is_selected() { "background: #374151; color: #60a5fa;" } else { "color: #d1d5db;" }
                                                    )
                                                    on:click=move |_| {
                                                        selected_store.set(Some(key2.clone()));
                                                        refresh_counter.update(|c| *c = c.wrapping_add(1));
                                                    }
                                                >
                                                    {store.key.clone()}
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </div>

                            // State viewer
                            <div style="flex: 1; padding: 12px; overflow: auto;">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                                    <span style="color: #9ca3af; font-size: 11px;">
                                        {move || selected_store.get().unwrap_or_else(|| "No store selected".to_string())}
                                    </span>
                                    <button
                                        style="background: #374151; border: none; color: #9ca3af; padding: 4px 10px; border-radius: 4px; cursor: pointer; font-size: 12px;"
                                        on:click=move |_| refresh_counter.update(|c| *c = c.wrapping_add(1))
                                    >
                                        "↻ Refresh"
                                    </button>
                                </div>
                                
                                // State tree
                                <div style="font-family: 'SF Mono', Monaco, 'Courier New', monospace; font-size: 12px; line-height: 1.6;">
                                    {move || {
                                        match get_current_state() {
                                            Some(state_str) => {
                                                // Check if JSON or Debug format
                                                if let Some(json_str) = state_str.strip_prefix("JSON:") {
                                                    view! { <JsonTreeView json=json_str.to_string() /> }.into_any()
                                                } else {
                                                    // Debug format - render as preformatted
                                                    view! {
                                                        <pre style="margin: 0; white-space: pre-wrap; word-break: break-word; color: #a5b4fc;">
                                                            {state_str}
                                                        </pre>
                                                    }.into_any()
                                                }
                                            }
                                            None => view! {
                                                <div style="color: #6b7280; text-align: center; padding: 40px;">
                                                    "Select a store to view its state"
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                            </div>
                        </div>
                    </Show>

                    // Events Tab
                    <Show when=move || active_tab.get() == "events">
                        <div style="display: flex; height: 100%;">
                            // Event list
                            <div style="width: 200px; border-right: 1px solid #374151; overflow-y: auto;">
                                {move || {
                                    events
                                        .get()
                                        .into_iter()
                                        .enumerate()
                                        .map(|(idx, event)| {
                                            let is_selected = move || selected_event.get() == Some(idx);
                                            let (color, icon) = match event.event_type.as_str() {
                                                "StateChanged" => ("#fbbf24", "⚡"),
                                                "StoreRegistered" => ("#34d399", "📦"),
                                                "MutationStarted" => ("#60a5fa", "▶"),
                                                "MutationCompleted" => ("#34d399", "✓"),
                                                "ActionDispatched" => ("#a78bfa", "→"),
                                                "ActionCompleted" => ("#34d399", "✓"),
                                                "Error" => ("#f87171", "✕"),
                                                _ => ("#9ca3af", "•"),
                                            };
                                            let store_name = event.store_name.clone().unwrap_or_default();
                                            let event_type = event.event_type.clone();
                                            view! {
                                                <div
                                                    style=move || format!(
                                                        "padding: 8px 12px; cursor: pointer; border-bottom: 1px solid #1f2937; transition: all 0.15s; {}",
                                                        if is_selected() { "background: #374151;" } else { "" }
                                                    )
                                                    on:click=move |_| selected_event.set(Some(idx))
                                                >
                                                    <div style=format!("color: {}; font-size: 12px; display: flex; align-items: center; gap: 6px;", color)>
                                                        <span>{icon}</span>
                                                        <span>{event_type.clone()}</span>
                                                    </div>
                                                    <div style="color: #6b7280; font-size: 11px; margin-top: 2px;">
                                                        {store_name.clone()}
                                                    </div>
                                                </div>
                                            }
                                        })
                                        .collect_view()
                                }}
                            </div>

                            // Event detail
                            <div style="flex: 1; padding: 12px; overflow: auto;">
                                {move || {
                                    match selected_event.get() {
                                        Some(idx) => {
                                            let event = events.get().get(idx).cloned();
                                            match event {
                                                Some(e) => view! { <EventDetailView event=e /> }.into_any(),
                                                None => view! {
                                                    <div style="color: #6b7280; text-align: center; padding: 40px;">
                                                        "Event not found"
                                                    </div>
                                                }.into_any()
                                            }
                                        }
                                        None => view! {
                                            <div style="color: #6b7280; text-align: center; padding: 40px;">
                                                "Select an event to view details"
                                            </div>
                                        }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </Show>
                </div>
            </Show>
        </div>
    }
}

/// JSON tree viewer component for displaying state
#[component]
fn JsonTreeView(json: String) -> impl IntoView {
    // Parse JSON and render as expandable tree
    let parsed = serde_json::from_str::<serde_json::Value>(&json);
    
    match parsed {
        Ok(value) => view! { <JsonValue value=value depth=0 /> }.into_any(),
        Err(_) => view! {
            <pre style="margin: 0; color: #f87171;">{format!("Invalid JSON: {}", json)}</pre>
        }.into_any()
    }
}

/// Recursive JSON value renderer
#[component]
fn JsonValue(value: serde_json::Value, depth: usize) -> impl IntoView {
    let indent = depth * 16;
    
    match value {
        serde_json::Value::Null => view! {
            <span style="color: #6b7280;">"null"</span>
        }.into_any(),
        serde_json::Value::Bool(b) => view! {
            <span style="color: #fbbf24;">{b.to_string()}</span>
        }.into_any(),
        serde_json::Value::Number(n) => view! {
            <span style="color: #34d399;">{n.to_string()}</span>
        }.into_any(),
        serde_json::Value::String(s) => view! {
            <span style="color: #fb923c;">"\""</span>
            <span style="color: #fbbf24;">{s}</span>
            <span style="color: #fb923c;">"\""</span>
        }.into_any(),
        serde_json::Value::Array(arr) => {
            let is_expanded = RwSignal::new(depth < 2);
            let len = arr.len();
            let items = arr.clone();
            view! {
                <div>
                    <span
                        style="cursor: pointer; color: #9ca3af; user-select: none;"
                        on:click=move |_| is_expanded.update(|e| *e = !*e)
                    >
                        {move || if is_expanded.get() { "▼ " } else { "▶ " }}
                    </span>
                    <span style="color: #60a5fa;">"["</span>
                    <Show when=move || !is_expanded.get()>
                        <span style="color: #6b7280;">{format!(" {} items ", len)}</span>
                        <span style="color: #60a5fa;">"]"</span>
                    </Show>
                    <Show when=move || is_expanded.get()>
                        <div style=format!("margin-left: {}px;", indent + 16)>
                            {items.clone().into_iter().enumerate().map(|(i, item)| {
                                view! {
                                    <div style="display: flex;">
                                        <span style="color: #6b7280; min-width: 24px;">{format!("{}:", i)}</span>
                                        <JsonValue value=item depth=depth+1 />
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                        <span style="color: #60a5fa;">"]"</span>
                    </Show>
                </div>
            }.into_any()
        }
        serde_json::Value::Object(obj) => {
            let is_expanded = RwSignal::new(depth < 2);
            let len = obj.len();
            let entries: Vec<_> = obj.into_iter().collect();
            let entries2 = entries.clone();
            view! {
                <div>
                    <span
                        style="cursor: pointer; color: #9ca3af; user-select: none;"
                        on:click=move |_| is_expanded.update(|e| *e = !*e)
                    >
                        {move || if is_expanded.get() { "▼ " } else { "▶ " }}
                    </span>
                    <span style="color: #a78bfa;">"{"</span>
                    <Show when=move || !is_expanded.get()>
                        <span style="color: #6b7280;">{format!(" {} fields ", len)}</span>
                        <span style="color: #a78bfa;">"}"</span>
                    </Show>
                    <Show when=move || is_expanded.get()>
                        <div style=format!("margin-left: {}px;", indent + 16)>
                            {entries2.clone().into_iter().map(|(key, val)| {
                                view! {
                                    <div style="display: flex; gap: 4px;">
                                        <span style="color: #60a5fa;">"\""</span>
                                        <span style="color: #93c5fd;">{key}</span>
                                        <span style="color: #60a5fa;">"\":"</span>
                                        <JsonValue value=val depth=depth+1 />
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                        <span style="color: #a78bfa;">"}"</span>
                    </Show>
                </div>
            }.into_any()
        }
    }
}

/// Event detail view component
#[component]
fn EventDetailView(event: DevtoolsEvent) -> impl IntoView {
    let payload_json: Option<serde_json::Value> = serde_json::from_str(&event.payload).ok();
    
    // Extract old/new values if present
    let (old_value, new_value) = match &payload_json {
        Some(serde_json::Value::Object(obj)) => {
            (obj.get("old").cloned(), obj.get("new").cloned())
        }
        _ => (None, None)
    };
    
    let has_diff = old_value.is_some() && new_value.is_some();
    let raw_payload = event.payload.clone();
    
    view! {
        <div style="font-family: 'SF Mono', Monaco, 'Courier New', monospace; font-size: 12px;">
            // Event header
            <div style="margin-bottom: 16px; padding-bottom: 12px; border-bottom: 1px solid #374151;">
                <div style="font-size: 14px; font-weight: 600; color: #f3f4f6; margin-bottom: 8px;">
                    {event.event_type.clone()}
                </div>
                <div style="display: grid; grid-template-columns: auto 1fr; gap: 4px 12px; color: #9ca3af; font-size: 11px;">
                    <span>"Store:"</span>
                    <span style="color: #60a5fa;">{event.store_name.clone().unwrap_or_else(|| "-".to_string())}</span>
                    <span>"Time:"</span>
                    <span>{format_timestamp(event.timestamp)}</span>
                </div>
            </div>
            
            // State diff view for StateChanged events
            <Show when=move || has_diff>
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px;">
                    // Old value
                    <div>
                        <div style="color: #f87171; font-size: 11px; margin-bottom: 6px; font-weight: 600;">
                            "← BEFORE"
                        </div>
                        <div style="background: #1f2937; padding: 8px; border-radius: 4px; border-left: 3px solid #f87171; overflow: auto; max-height: 200px;">
                            {match old_value.clone() {
                                Some(val) => view! { <JsonValue value=val depth=0 /> }.into_any(),
                                None => view! { <span style="color: #6b7280;">"-"</span> }.into_any()
                            }}
                        </div>
                    </div>
                    // New value
                    <div>
                        <div style="color: #34d399; font-size: 11px; margin-bottom: 6px; font-weight: 600;">
                            "→ AFTER"
                        </div>
                        <div style="background: #1f2937; padding: 8px; border-radius: 4px; border-left: 3px solid #34d399; overflow: auto; max-height: 200px;">
                            {match new_value.clone() {
                                Some(val) => view! { <JsonValue value=val depth=0 /> }.into_any(),
                                None => view! { <span style="color: #6b7280;">"-"</span> }.into_any()
                            }}
                        </div>
                    </div>
                </div>
            </Show>
            
            // Raw payload for other events
            <Show when=move || !has_diff>
                <div>
                    <div style="color: #9ca3af; font-size: 11px; margin-bottom: 6px;">
                        "Payload"
                    </div>
                    <div style="background: #1f2937; padding: 8px; border-radius: 4px; overflow: auto; max-height: 200px;">
                        {match payload_json.clone() {
                            Some(val) => view! { <JsonValue value=val depth=0 /> }.into_any(),
                            None => view! {
                                <pre style="margin: 0; color: #d1d5db; white-space: pre-wrap;">{raw_payload.clone()}</pre>
                            }.into_any()
                        }}
                    </div>
                </div>
            </Show>
        </div>
    }
}

/// Format timestamp as human-readable time
fn format_timestamp(ts: u64) -> String {
    // Simple formatting - just show relative or absolute time
    let now = current_timestamp_ms();
    let diff = now.saturating_sub(ts);
    
    if diff < 1000 {
        "just now".to_string()
    } else if diff < 60_000 {
        format!("{}s ago", diff / 1000)
    } else if diff < 3_600_000 {
        format!("{}m ago", diff / 60_000)
    } else {
        format!("{}h ago", diff / 3_600_000)
    }
}

// ============================================================================
// Browser Extension Protocol (Tier 3)
// ============================================================================

/// Message types for Redux DevTools protocol.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "hydrate", derive(serde::Serialize, serde::Deserialize))]
pub enum DevtoolsMessage {
    /// Initialize connection.
    Init {
        /// Instance ID.
        instance_id: String,
        /// Store names.
        stores: Vec<String>,
    },
    /// State update.
    StateUpdate {
        /// Store name.
        store: String,
        /// Action that caused the update.
        action: String,
        /// New state (JSON).
        state: String,
        /// Timestamp.
        timestamp: u64,
    },
    /// Action dispatch.
    Action {
        /// Store name.
        store: String,
        /// Action type.
        action_type: String,
        /// Action payload (JSON).
        payload: String,
    },
    /// Jump to state (time travel).
    JumpToState {
        /// Target state index.
        index: usize,
    },
    /// Import state.
    ImportState {
        /// Serialized state.
        state: String,
    },
    /// Export state.
    ExportState,
}

/// Connect to Redux DevTools extension.
#[cfg(target_arch = "wasm32")]
pub fn connect_devtools_extension(_instance_name: &str) -> bool {
    // Check if extension is available
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };

    // Check for __REDUX_DEVTOOLS_EXTENSION__
    let has_extension =
        js_sys::Reflect::get(&window, &JsValue::from_str("__REDUX_DEVTOOLS_EXTENSION__"))
            .map(|v| !v.is_undefined())
            .unwrap_or(false);

    if has_extension {
        leptos::logging::log!("[Devtools] Redux DevTools extension detected");
        // Connection logic would go here
        true
    } else {
        false
    }
}

/// Connect to devtools extension (no-op on non-WASM).
#[cfg(not(target_arch = "wasm32"))]
pub fn connect_devtools_extension(_instance_name: &str) -> bool {
    false
}

/// Send a message to the devtools extension.
#[cfg(target_arch = "wasm32")]
pub fn send_to_extension(_message: DevtoolsMessage) {
    // Implementation would use postMessage to communicate with extension
}

/// Send a message to the devtools extension (no-op on non-WASM).
#[cfg(not(target_arch = "wasm32"))]
pub fn send_to_extension(_message: DevtoolsMessage) {
    // No-op
}

// ============================================================================
// Time Travel Debugging
// ============================================================================

/// State snapshot for time travel.
#[derive(Clone, Debug)]
pub struct StateSnapshot {
    /// Snapshot index.
    pub index: usize,
    /// Store name.
    pub store: String,
    /// Action that produced this state.
    pub action: String,
    /// Serialized state.
    pub state: String,
    /// Timestamp.
    pub timestamp: u64,
}

/// Time travel debugger.
#[derive(Clone)]
pub struct TimeTravelDebugger {
    snapshots: RwSignal<Vec<StateSnapshot>>,
    current_index: RwSignal<usize>,
    max_snapshots: usize,
}

impl Default for TimeTravelDebugger {
    fn default() -> Self {
        Self::new(100)
    }
}

impl TimeTravelDebugger {
    /// Create a new time travel debugger.
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: RwSignal::new(Vec::new()),
            current_index: RwSignal::new(0),
            max_snapshots,
        }
    }

    /// Record a state snapshot.
    pub fn record(&self, store: &str, action: &str, state: String) {
        self.snapshots.update(|snapshots| {
            let index = snapshots.len();
            snapshots.push(StateSnapshot {
                index,
                store: store.to_string(),
                action: action.to_string(),
                state,
                timestamp: current_timestamp_ms(),
            });

            // Trim if over limit
            if snapshots.len() > self.max_snapshots {
                snapshots.remove(0);
                // Reindex
                for (i, s) in snapshots.iter_mut().enumerate() {
                    s.index = i;
                }
            }
        });

        self.current_index
            .set(self.snapshots.with(|s| s.len().saturating_sub(1)));
    }

    /// Jump to a specific snapshot.
    pub fn jump_to(&self, index: usize) -> Option<StateSnapshot> {
        let snapshot = self.snapshots.with(|s| s.get(index).cloned());
        if snapshot.is_some() {
            self.current_index.set(index);
        }
        snapshot
    }

    /// Go to previous snapshot.
    pub fn prev(&self) -> Option<StateSnapshot> {
        let current = self.current_index.get();
        if current > 0 {
            self.jump_to(current - 1)
        } else {
            None
        }
    }

    /// Go to next snapshot.
    pub fn next(&self) -> Option<StateSnapshot> {
        let current = self.current_index.get();
        let len = self.snapshots.with(|s| s.len());
        if current + 1 < len {
            self.jump_to(current + 1)
        } else {
            None
        }
    }

    /// Get current snapshot.
    pub fn current(&self) -> Option<StateSnapshot> {
        let index = self.current_index.get();
        self.snapshots.with(|s| s.get(index).cloned())
    }

    /// Get all snapshots.
    pub fn snapshots(&self) -> Vec<StateSnapshot> {
        self.snapshots.get()
    }

    /// Clear all snapshots.
    pub fn clear(&self) {
        self.snapshots.set(Vec::new());
        self.current_index.set(0);
    }

    /// Get snapshot count.
    pub fn len(&self) -> usize {
        self.snapshots.with(|s| s.len())
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.snapshots.with(|s| s.is_empty())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get current timestamp in milliseconds.
fn current_timestamp_ms() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devtools_config_default() {
        let config = DevtoolsConfig::default();
        assert_eq!(config.max_events, 1000);
        assert!(config.expose_console_api);
    }

    #[test]
    fn test_store_info() {
        let info = StoreInfo {
            name: "TestStore".to_string(),
            key: "test".to_string(),
            type_name: "TestStore",
            registered_at: 12345,
        };
        assert_eq!(info.key, "test");
    }

    #[test]
    fn test_devtools_event() {
        let event = DevtoolsEvent {
            event_type: "MutationCompleted".to_string(),
            store_name: Some("auth".to_string()),
            payload: "{}".to_string(),
            timestamp: 12345,
        };
        assert_eq!(event.event_type, "MutationCompleted");
    }

    #[test]
    fn test_time_travel_debugger() {
        let debugger = TimeTravelDebugger::new(10);
        assert!(debugger.is_empty());

        debugger.record("test", "increment", r#"{"count":1}"#.to_string());
        assert_eq!(debugger.len(), 1);

        debugger.record("test", "increment", r#"{"count":2}"#.to_string());
        assert_eq!(debugger.len(), 2);

        // Jump back
        let prev = debugger.prev();
        assert!(prev.is_some());
        assert_eq!(prev.unwrap().state, r#"{"count":1}"#);

        // Jump forward
        let next = debugger.next();
        assert!(next.is_some());
        assert_eq!(next.unwrap().state, r#"{"count":2}"#);

        debugger.clear();
        assert!(debugger.is_empty());
    }

    #[test]
    fn test_time_travel_max_snapshots() {
        let debugger = TimeTravelDebugger::new(3);

        for i in 0..5 {
            debugger.record("test", "action", format!(r#"{{"count":{}}}"#, i));
        }

        // Should only keep the last 3
        assert_eq!(debugger.len(), 3);

        let snapshots = debugger.snapshots();
        assert_eq!(snapshots[0].state, r#"{"count":2}"#);
        assert_eq!(snapshots[2].state, r#"{"count":4}"#);
    }

    #[test]
    fn test_state_snapshot() {
        let snapshot = StateSnapshot {
            index: 0,
            store: "auth".to_string(),
            action: "login".to_string(),
            state: r#"{"user":"test"}"#.to_string(),
            timestamp: 12345,
        };
        assert_eq!(snapshot.store, "auth");
        assert_eq!(snapshot.action, "login");
    }
}
