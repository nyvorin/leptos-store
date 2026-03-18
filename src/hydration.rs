// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Hydration support for SSR stores.
//!
//! This module provides utilities for serializing store state on the server
//! and deserializing it on the client during hydration, ensuring state
//! consistency and avoiding hydration mismatches.
//!
//! # Overview
//!
//! Hydration is the process where client-side code takes over server-rendered
//! HTML and makes it interactive. For state management, this means:
//!
//! 1. **Server**: Serialize store state to JSON and embed in HTML
//! 2. **Client**: Read serialized state and initialize stores with it
//!
//! This ensures that the client's reactive signals start with the same values
//! as the server, preventing hydration mismatches.
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_store::prelude::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, Debug, Default, Serialize, Deserialize)]
//! pub struct CounterState {
//!     pub count: i32,
//! }
//!
//! #[derive(Clone)]
//! pub struct CounterStore {
//!     state: RwSignal<CounterState>,
//! }
//!
//! impl HydratableStore for CounterStore {
//!     fn serialize_state(&self) -> Result<String, StoreHydrationError> {
//!         let state = self.state.get();
//!         serde_json::to_string(&state)
//!             .map_err(|e| StoreHydrationError::Serialization(e.to_string()))
//!     }
//!
//!     fn from_hydrated_state(data: &str) -> Result<Self, StoreHydrationError> {
//!         let state: CounterState = serde_json::from_str(data)
//!             .map_err(|e| StoreHydrationError::Deserialization(e.to_string()))?;
//!         Ok(Self {
//!             state: RwSignal::new(state),
//!         })
//!     }
//!
//!     fn store_key() -> &'static str {
//!         "counter"
//!     }
//! }
//! ```

#[cfg(feature = "hydrate")]
use crate::store::Store;
use thiserror::Error;

/// Errors that can occur during store hydration.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum StoreHydrationError {
    /// Failed to serialize state.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Failed to deserialize state.
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Hydration data not found in DOM.
    #[error("Hydration data not found for key: {0}")]
    NotFound(String),

    /// Invalid hydration data format.
    #[error("Invalid hydration data: {0}")]
    InvalidData(String),

    /// DOM access error (WASM-specific).
    #[error("DOM error: {0}")]
    DomError(String),
}

/// Trait for stores that support SSR hydration.
///
/// Implement this trait to enable automatic state transfer between
/// server and client during SSR hydration.
///
/// # Type Bounds
///
/// When the `hydrate` feature is enabled, the state type must implement
/// `serde::Serialize` and `serde::de::DeserializeOwned`.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::prelude::*;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Clone, Default, Serialize, Deserialize)]
/// struct MyState {
///     value: i32,
/// }
///
/// #[derive(Clone)]
/// struct MyStore {
///     state: RwSignal<MyState>,
/// }
///
/// impl HydratableStore for MyStore {
///     fn serialize_state(&self) -> Result<String, StoreHydrationError> {
///         serde_json::to_string(&self.state.get())
///             .map_err(|e| StoreHydrationError::Serialization(e.to_string()))
///     }
///
///     fn from_hydrated_state(data: &str) -> Result<Self, StoreHydrationError> {
///         let state: MyState = serde_json::from_str(data)
///             .map_err(|e| StoreHydrationError::Deserialization(e.to_string()))?;
///         Ok(Self { state: RwSignal::new(state) })
///     }
///
///     fn store_key() -> &'static str {
///         "my_store"
///     }
/// }
/// ```
#[cfg(feature = "hydrate")]
pub trait HydratableStore: Store + Sized {
    /// Serialize the store's state to a JSON string.
    ///
    /// This is called on the server during SSR to capture the current
    /// state for transfer to the client.
    fn serialize_state(&self) -> Result<String, StoreHydrationError>;

    /// Create a new store from serialized state data.
    ///
    /// This is called on the client during hydration to restore the
    /// store's state from the server-rendered data.
    fn from_hydrated_state(data: &str) -> Result<Self, StoreHydrationError>;

    /// Returns a unique key for this store type.
    ///
    /// This key is used to identify the store's data in the hydration
    /// script tag. Must be unique across all stores in the application.
    fn store_key() -> &'static str;
}

/// The ID prefix used for hydration script tags.
pub const HYDRATION_SCRIPT_PREFIX: &str = "__LEPTOS_STORE_STATE__";

/// Generate the full script element ID for a store.
#[cfg(feature = "hydrate")]
pub fn hydration_script_id(store_key: &str) -> String {
    format!("{HYDRATION_SCRIPT_PREFIX}{store_key}")
}

/// Serialize a store's state to JSON for embedding in HTML.
///
/// # Arguments
///
/// * `store` - The store to serialize
///
/// # Returns
///
/// A JSON string representation of the store's state.
#[cfg(feature = "hydrate")]
pub fn serialize_store_state<S: HydratableStore>(store: &S) -> Result<String, StoreHydrationError> {
    store.serialize_state()
}

/// Read hydration data from the DOM.
///
/// This function looks for a script tag with the store's hydration ID
/// and extracts the serialized state data.
///
/// # Arguments
///
/// * `store_key` - The unique key for the store
///
/// # Returns
///
/// The serialized state data as a string.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub fn read_hydration_data(store_key: &str) -> Result<String, StoreHydrationError> {
    use wasm_bindgen::JsCast;

    let window = web_sys::window()
        .ok_or_else(|| StoreHydrationError::DomError("No window object".to_string()))?;

    let document = window
        .document()
        .ok_or_else(|| StoreHydrationError::DomError("No document object".to_string()))?;

    let script_id = hydration_script_id(store_key);
    let element = document
        .get_element_by_id(&script_id)
        .ok_or_else(|| StoreHydrationError::NotFound(store_key.to_string()))?;

    let script = element
        .dyn_into::<web_sys::HtmlScriptElement>()
        .map_err(|_| StoreHydrationError::InvalidData("Element is not a script tag".to_string()))?;

    let content = script.text().map_err(|e| {
        StoreHydrationError::DomError(format!("Failed to read script content: {:?}", e))
    })?;

    Ok(content)
}

/// Stub for non-WASM targets.
#[cfg(all(feature = "hydrate", not(target_arch = "wasm32")))]
pub fn read_hydration_data(store_key: &str) -> Result<String, StoreHydrationError> {
    Err(StoreHydrationError::DomError(format!(
        "DOM access not available on this platform for key: {store_key}"
    )))
}

/// Hydrate a store from DOM data.
///
/// This function reads the serialized state from the DOM and creates
/// a new store instance with the hydrated state.
///
/// # Type Parameters
///
/// * `S` - The store type to hydrate
///
/// # Returns
///
/// A new store instance with the hydrated state, or an error if
/// hydration fails.
#[cfg(feature = "hydrate")]
pub fn hydrate_store<S: HydratableStore>() -> Result<S, StoreHydrationError> {
    let data = read_hydration_data(S::store_key())?;
    S::from_hydrated_state(&data)
}

/// Check if hydration data is available for a store.
///
/// This is useful for conditional hydration logic where you want
/// to fall back to default state if no hydration data exists.
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
pub fn has_hydration_data(store_key: &str) -> bool {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            let script_id = hydration_script_id(store_key);
            return document.get_element_by_id(&script_id).is_some();
        }
    }
    false
}

/// Stub for non-WASM targets.
#[cfg(all(feature = "hydrate", not(target_arch = "wasm32")))]
pub fn has_hydration_data(_store_key: &str) -> bool {
    false
}

/// Generate the HTML for a hydration script tag.
///
/// This is used during SSR to embed the serialized store state
/// in the HTML document.
///
/// # Arguments
///
/// * `store_key` - The unique key for the store
/// * `data` - The serialized state data
///
/// # Returns
///
/// An HTML string containing the script tag with the embedded data.
#[cfg(feature = "hydrate")]
pub fn hydration_script_html(store_key: &str, data: &str) -> String {
    let script_id = hydration_script_id(store_key);
    // Escape any script closing tags in the data
    let escaped_data = data.replace("</script>", "<\\/script>");
    format!(r#"<script id="{script_id}" type="application/json">{escaped_data}</script>"#)
}

/// A builder for creating hydration-aware stores.
///
/// This builder provides a fluent API for creating stores that
/// automatically handle hydration on the client.
#[cfg(feature = "hydrate")]
pub struct HydrationBuilder<S: HydratableStore> {
    fallback: Option<S>,
}

#[cfg(feature = "hydrate")]
impl<S: HydratableStore> Default for HydrationBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "hydrate")]
impl<S: HydratableStore> HydrationBuilder<S> {
    /// Create a new hydration builder.
    pub fn new() -> Self {
        Self { fallback: None }
    }

    /// Set a fallback store to use if hydration fails.
    ///
    /// If hydration data is not found or deserialization fails,
    /// this fallback store will be used instead.
    pub fn with_fallback(mut self, store: S) -> Self {
        self.fallback = Some(store);
        self
    }

    /// Build the store, attempting hydration first.
    ///
    /// This will try to hydrate from DOM data. If hydration fails
    /// and a fallback was provided, the fallback will be used.
    ///
    /// # Panics
    ///
    /// Panics if hydration fails and no fallback was provided.
    pub fn build(self) -> S {
        match hydrate_store::<S>() {
            Ok(store) => store,
            Err(e) => {
                if let Some(fallback) = self.fallback {
                    fallback
                } else {
                    panic!("Store hydration failed and no fallback provided: {e}")
                }
            }
        }
    }

    /// Build the store, returning a Result.
    ///
    /// This will try to hydrate from DOM data. If hydration fails
    /// and a fallback was provided, the fallback will be returned.
    pub fn try_build(self) -> Result<S, StoreHydrationError> {
        match hydrate_store::<S>() {
            Ok(store) => Ok(store),
            Err(e) => {
                if let Some(fallback) = self.fallback {
                    Ok(fallback)
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hydration_error_display() {
        let err = StoreHydrationError::Serialization("test error".to_string());
        assert_eq!(err.to_string(), "Serialization error: test error");

        let err = StoreHydrationError::Deserialization("parse error".to_string());
        assert_eq!(err.to_string(), "Deserialization error: parse error");

        let err = StoreHydrationError::NotFound("my_store".to_string());
        assert_eq!(
            err.to_string(),
            "Hydration data not found for key: my_store"
        );

        let err = StoreHydrationError::InvalidData("bad format".to_string());
        assert_eq!(err.to_string(), "Invalid hydration data: bad format");

        let err = StoreHydrationError::DomError("no window".to_string());
        assert_eq!(err.to_string(), "DOM error: no window");
    }

    #[test]
    fn test_hydration_script_id() {
        #[cfg(feature = "hydrate")]
        {
            let id = hydration_script_id("my_store");
            assert_eq!(id, "__LEPTOS_STORE_STATE__my_store");
        }
    }

    #[test]
    fn test_hydration_script_html() {
        #[cfg(feature = "hydrate")]
        {
            let html = hydration_script_html("counter", r#"{"count":42}"#);
            assert!(html.contains(r#"id="__LEPTOS_STORE_STATE__counter""#));
            assert!(html.contains(r#"type="application/json""#));
            assert!(html.contains(r#"{"count":42}"#));
        }
    }

    #[test]
    fn test_hydration_script_html_escapes_script_tags() {
        #[cfg(feature = "hydrate")]
        {
            let html = hydration_script_html("test", r#"{"value":"</script>"}"#);
            assert!(html.contains(r#"<\/script>"#));
            assert!(!html.contains(r#"</script>"}"#));
        }
    }

    // ========================================================================
    // Integration tests for the full hydration workflow
    // ========================================================================

    #[cfg(feature = "hydrate")]
    mod hydration_integration {
        use super::*;
        use crate::store::Store;
        use leptos::prelude::*;
        use serde::{Deserialize, Serialize};

        /// Test state with various field types
        #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
        struct TestState {
            count: i32,
            name: String,
            items: Vec<String>,
            optional: Option<bool>,
        }

        /// Test store that implements HydratableStore
        #[derive(Clone)]
        struct TestHydratableStore {
            state: RwSignal<TestState>,
        }

        impl TestHydratableStore {
            fn new() -> Self {
                Self {
                    state: RwSignal::new(TestState::default()),
                }
            }

            fn with_state(state: TestState) -> Self {
                Self {
                    state: RwSignal::new(state),
                }
            }
        }

        impl Store for TestHydratableStore {
            type State = TestState;

            fn state(&self) -> leptos::prelude::ReadSignal<Self::State> {
                self.state.read_only()
            }
        }

        impl HydratableStore for TestHydratableStore {
            fn serialize_state(&self) -> Result<String, StoreHydrationError> {
                let state = self.state.get();
                serde_json::to_string(&state)
                    .map_err(|e| StoreHydrationError::Serialization(e.to_string()))
            }

            fn from_hydrated_state(data: &str) -> Result<Self, StoreHydrationError> {
                let state: TestState = serde_json::from_str(data)
                    .map_err(|e| StoreHydrationError::Deserialization(e.to_string()))?;
                Ok(Self::with_state(state))
            }

            fn store_key() -> &'static str {
                "test_store"
            }
        }

        #[test]
        fn test_store_serialization_roundtrip() {
            // Create a store with specific state
            let original_state = TestState {
                count: 42,
                name: "Test Store".to_string(),
                items: vec!["item1".to_string(), "item2".to_string()],
                optional: Some(true),
            };

            let store = TestHydratableStore::with_state(original_state.clone());

            // Serialize
            let serialized = store
                .serialize_state()
                .expect("Serialization should succeed");

            // Verify JSON structure
            assert!(serialized.contains("42"));
            assert!(serialized.contains("Test Store"));
            assert!(serialized.contains("item1"));

            // Deserialize into new store
            let restored_store = TestHydratableStore::from_hydrated_state(&serialized)
                .expect("Deserialization should succeed");

            // Verify state matches
            let restored_state = restored_store.state.get();
            assert_eq!(restored_state, original_state);
        }

        #[test]
        fn test_store_default_state_roundtrip() {
            let store = TestHydratableStore::new();

            let serialized = store
                .serialize_state()
                .expect("Serialization should succeed");
            let restored = TestHydratableStore::from_hydrated_state(&serialized)
                .expect("Deserialization should succeed");

            assert_eq!(restored.state.get(), TestState::default());
        }

        #[test]
        fn test_store_key_is_correct() {
            assert_eq!(TestHydratableStore::store_key(), "test_store");
        }

        #[test]
        fn test_full_hydration_html_generation() {
            let state = TestState {
                count: 100,
                name: "Hydration Test".to_string(),
                items: vec!["a".to_string(), "b".to_string()],
                optional: None,
            };

            let store = TestHydratableStore::with_state(state);
            let serialized = store.serialize_state().unwrap();

            // Generate the full hydration HTML
            let html = hydration_script_html(TestHydratableStore::store_key(), &serialized);

            // Verify the HTML structure
            assert!(html.starts_with("<script"));
            assert!(html.ends_with("</script>"));
            assert!(html.contains("__LEPTOS_STORE_STATE__test_store"));
            assert!(html.contains("application/json"));
            assert!(html.contains("100")); // count value
            assert!(html.contains("Hydration Test")); // name value
        }

        #[test]
        fn test_serialize_store_state_helper() {
            let store = TestHydratableStore::with_state(TestState {
                count: 999,
                ..Default::default()
            });

            let serialized = serialize_store_state(&store).unwrap();
            assert!(serialized.contains("999"));
        }

        #[test]
        fn test_hydration_builder_with_fallback() {
            // Since we can't read from DOM in tests, the builder should use fallback
            let fallback = TestHydratableStore::with_state(TestState {
                count: 123,
                name: "Fallback".to_string(),
                ..Default::default()
            });

            let store = HydrationBuilder::<TestHydratableStore>::new()
                .with_fallback(fallback)
                .try_build()
                .expect("Should succeed with fallback");

            // Should get the fallback state (since DOM isn't available in tests)
            assert_eq!(store.state.get().count, 123);
            assert_eq!(store.state.get().name, "Fallback");
        }

        #[test]
        fn test_deserialization_error_handling() {
            // Invalid JSON
            let result = TestHydratableStore::from_hydrated_state("not valid json");
            assert!(result.is_err());

            if let Err(StoreHydrationError::Deserialization(msg)) = result {
                assert!(!msg.is_empty());
            } else {
                panic!("Expected Deserialization error");
            }

            // Valid JSON but wrong structure
            let result = TestHydratableStore::from_hydrated_state(r#"{"wrong":"field"}"#);
            assert!(result.is_err());
        }

        #[test]
        fn test_special_characters_in_state() {
            let state = TestState {
                count: 0,
                name: r#"Test with "quotes" and <tags> and </script>"#.to_string(),
                items: vec!["<script>alert('xss')</script>".to_string()],
                optional: None,
            };

            let store = TestHydratableStore::with_state(state.clone());
            let serialized = store.serialize_state().unwrap();

            // The serialization should handle special characters
            let restored = TestHydratableStore::from_hydrated_state(&serialized).unwrap();
            assert_eq!(restored.state.get(), state);

            // The HTML output should escape script tags
            let html = hydration_script_html(TestHydratableStore::store_key(), &serialized);
            // Script tags in the content should be escaped
            assert!(!html.contains("</script>\""));
        }

        #[test]
        fn test_empty_state_roundtrip() {
            let state = TestState {
                count: 0,
                name: String::new(),
                items: vec![],
                optional: None,
            };

            let store = TestHydratableStore::with_state(state.clone());
            let serialized = store.serialize_state().unwrap();
            let restored = TestHydratableStore::from_hydrated_state(&serialized).unwrap();

            assert_eq!(restored.state.get(), state);
        }

        #[test]
        fn test_large_state_roundtrip() {
            // Test with a larger state to ensure no size issues
            let state = TestState {
                count: i32::MAX,
                name: "x".repeat(10000),
                items: (0..1000).map(|i| format!("item_{i}")).collect(),
                optional: Some(true),
            };

            let store = TestHydratableStore::with_state(state.clone());
            let serialized = store.serialize_state().unwrap();
            let restored = TestHydratableStore::from_hydrated_state(&serialized).unwrap();

            assert_eq!(restored.state.get(), state);
        }
    }
}
