// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Feature flag store template.
//!
//! This module provides a complete feature flag management system including:
//!
//! - Feature flag state management
//! - Local overrides for development
//! - Remote sync capabilities
//! - Leptos component for conditional rendering
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_store::templates::feature_flags::*;
//!
//! #[component]
//! fn App() -> impl IntoView {
//!     let flags = FeatureFlagStore::new();
//!     flags.set_flags(vec![
//!         FeatureFlag::new("dark_mode", true),
//!         FeatureFlag::new("beta_features", false),
//!     ]);
//!     provide_context(flags);
//!
//!     view! {
//!         <Feature flag="dark_mode">
//!             <DarkModeComponent />
//!         </Feature>
//!     }
//! }
//! ```

use crate::store::Store;
use leptos::prelude::*;
use std::collections::HashMap;
use thiserror::Error;

#[cfg(feature = "hydrate")]
use serde::{Deserialize, Serialize};

// ============================================================================
// Types
// ============================================================================

/// A feature flag definition.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "hydrate", derive(Serialize, Deserialize))]
pub struct FeatureFlag {
    /// Unique key for the flag.
    pub key: String,
    /// Whether the flag is enabled.
    pub enabled: bool,
    /// Optional variant for A/B testing.
    pub variant: Option<String>,
    /// Additional metadata.
    #[cfg_attr(feature = "hydrate", serde(default))]
    pub metadata: HashMap<String, String>,
    /// Description of the flag (for documentation).
    pub description: Option<String>,
}

impl FeatureFlag {
    /// Create a new feature flag.
    pub fn new(key: impl Into<String>, enabled: bool) -> Self {
        Self {
            key: key.into(),
            enabled,
            variant: None,
            metadata: HashMap::new(),
            description: None,
        }
    }

    /// Create a feature flag with a variant.
    pub fn with_variant(key: impl Into<String>, enabled: bool, variant: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            enabled,
            variant: Some(variant.into()),
            metadata: HashMap::new(),
            description: None,
        }
    }

    /// Add a description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// User context for flag evaluation.
///
/// This context is sent to the flag provider to enable
/// user-specific flag values.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "hydrate", derive(Serialize, Deserialize))]
pub struct UserContext {
    /// User ID (if authenticated).
    pub user_id: Option<String>,
    /// User email (if available).
    pub email: Option<String>,
    /// User attributes for targeting.
    pub attributes: HashMap<String, String>,
    /// Environment (e.g., "development", "staging", "production").
    pub environment: Option<String>,
}

impl UserContext {
    /// Create a new empty user context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a context for an authenticated user.
    pub fn authenticated(user_id: impl Into<String>) -> Self {
        Self {
            user_id: Some(user_id.into()),
            ..Default::default()
        }
    }

    /// Set the user ID.
    pub fn with_user_id(mut self, id: impl Into<String>) -> Self {
        self.user_id = Some(id.into());
        self
    }

    /// Set the email.
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Set the environment.
    pub fn with_environment(mut self, env: impl Into<String>) -> Self {
        self.environment = Some(env.into());
        self
    }

    /// Add an attribute.
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

// ============================================================================
// Errors
// ============================================================================

/// Errors that can occur during flag operations.
#[derive(Debug, Error, Clone)]
pub enum FlagError {
    /// Flag not found.
    #[error("Flag not found: {0}")]
    NotFound(String),

    /// Failed to fetch flags from remote.
    #[error("Failed to fetch flags: {0}")]
    FetchFailed(String),

    /// Invalid flag configuration.
    #[error("Invalid flag configuration: {0}")]
    InvalidConfig(String),

    /// Network error.
    #[error("Network error: {0}")]
    Network(String),
}

// ============================================================================
// State
// ============================================================================

/// Feature flag store state.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "hydrate", derive(Serialize, Deserialize))]
pub struct FeatureFlagState {
    /// All flags indexed by key.
    flags: HashMap<String, FeatureFlag>,
    /// Local overrides (for development).
    #[cfg_attr(feature = "hydrate", serde(skip))]
    overrides: HashMap<String, bool>,
    /// Whether flags have been loaded.
    loaded: bool,
    /// Last error (transient, not serialized).
    #[cfg_attr(feature = "hydrate", serde(skip))]
    error: Option<FlagError>,
    /// Loading state (transient).
    #[cfg_attr(feature = "hydrate", serde(skip))]
    loading: bool,
}

impl FeatureFlagState {
    /// Check if a flag is enabled.
    ///
    /// This checks overrides first, then the actual flag value.
    pub fn is_enabled(&self, key: &str) -> bool {
        // Check overrides first
        if let Some(&override_value) = self.overrides.get(key) {
            return override_value;
        }

        // Check actual flag
        self.flags.get(key).map(|f| f.enabled).unwrap_or(false)
    }

    /// Get a flag's variant.
    pub fn get_variant(&self, key: &str) -> Option<String> {
        self.flags.get(key).and_then(|f| f.variant.clone())
    }

    /// Get a flag by key.
    pub fn get_flag(&self, key: &str) -> Option<&FeatureFlag> {
        self.flags.get(key)
    }
}

// ============================================================================
// Store
// ============================================================================

/// Feature flag store for managing feature flags.
///
/// # Example
///
/// ```rust
/// use leptos_store::templates::feature_flags::{FeatureFlagStore, FeatureFlag};
///
/// let store = FeatureFlagStore::new();
///
/// // Set flags (e.g., from server response)
/// store.set_flags(vec![
///     FeatureFlag::new("new_checkout", true),
///     FeatureFlag::with_variant("homepage_hero", true, "variant_a"),
/// ]);
///
/// // Check flags
/// if store.is_enabled("new_checkout") {
///     // Show new checkout flow
/// }
///
/// // Set local override for development
/// store.set_override("experimental_feature", true);
/// ```
#[derive(Clone)]
pub struct FeatureFlagStore {
    state: RwSignal<FeatureFlagState>,
}

impl Default for FeatureFlagStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureFlagStore {
    /// Create a new feature flag store.
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(FeatureFlagState::default()),
        }
    }

    /// Create a store with initial flags.
    pub fn with_flags(flags: Vec<FeatureFlag>) -> Self {
        let store = Self::new();
        store.set_flags(flags);
        store
    }

    // ========================================================================
    // Getters
    // ========================================================================

    /// Check if a flag is enabled.
    ///
    /// This checks local overrides first, then the actual flag value.
    /// Returns `false` for unknown flags.
    pub fn is_enabled(&self, key: &str) -> bool {
        self.state.with(|s| s.is_enabled(key))
    }

    /// Get a flag's variant.
    pub fn get_variant(&self, key: &str) -> Option<String> {
        self.state.with(|s| s.get_variant(key))
    }

    /// Get all flags.
    pub fn all_flags(&self) -> Vec<FeatureFlag> {
        self.state.with(|s| s.flags.values().cloned().collect())
    }

    /// Get a specific flag.
    pub fn get_flag(&self, key: &str) -> Option<FeatureFlag> {
        self.state.with(|s| s.flags.get(key).cloned())
    }

    /// Get all flag keys.
    pub fn flag_keys(&self) -> Vec<String> {
        self.state.with(|s| s.flags.keys().cloned().collect())
    }

    /// Check if flags have been loaded.
    pub fn is_loaded(&self) -> bool {
        self.state.with(|s| s.loaded)
    }

    /// Check if currently loading.
    pub fn is_loading(&self) -> bool {
        self.state.with(|s| s.loading)
    }

    /// Get the current error.
    pub fn error(&self) -> Option<FlagError> {
        self.state.with(|s| s.error.clone())
    }

    /// Get all current overrides.
    pub fn overrides(&self) -> HashMap<String, bool> {
        self.state.with(|s| s.overrides.clone())
    }

    // ========================================================================
    // Mutators
    // ========================================================================

    /// Set flags from a list.
    ///
    /// This replaces all existing flags.
    pub fn set_flags(&self, flags: Vec<FeatureFlag>) {
        self.state.update(|s| {
            s.flags = flags.into_iter().map(|f| (f.key.clone(), f)).collect();
            s.loaded = true;
            s.error = None;
        });
    }

    /// Add or update a single flag.
    pub fn set_flag(&self, flag: FeatureFlag) {
        self.state.update(|s| {
            s.flags.insert(flag.key.clone(), flag);
        });
    }

    /// Remove a flag.
    pub fn remove_flag(&self, key: &str) {
        self.state.update(|s| {
            s.flags.remove(key);
        });
    }

    /// Set a local override for a flag.
    ///
    /// Overrides take precedence over actual flag values.
    /// Useful for development and testing.
    pub fn set_override(&self, key: impl Into<String>, enabled: bool) {
        self.state.update(|s| {
            s.overrides.insert(key.into(), enabled);
        });
    }

    /// Remove a local override.
    pub fn remove_override(&self, key: &str) {
        self.state.update(|s| {
            s.overrides.remove(key);
        });
    }

    /// Clear all local overrides.
    pub fn clear_overrides(&self) {
        self.state.update(|s| {
            s.overrides.clear();
        });
    }

    /// Set loading state.
    pub fn set_loading(&self, loading: bool) {
        self.state.update(|s| {
            s.loading = loading;
        });
    }

    /// Set error state.
    pub fn set_error(&self, error: Option<FlagError>) {
        self.state.update(|s| {
            s.error = error;
            s.loading = false;
        });
    }

    /// Clear all flags.
    pub fn clear(&self) {
        self.state.update(|s| {
            s.flags.clear();
            s.loaded = false;
        });
    }

    // ========================================================================
    // Actions
    // ========================================================================

    /// Enable a flag (sets it to true).
    pub fn enable(&self, key: &str) {
        self.state.update(|s| {
            if let Some(flag) = s.flags.get_mut(key) {
                flag.enabled = true;
            }
        });
    }

    /// Disable a flag (sets it to false).
    pub fn disable(&self, key: &str) {
        self.state.update(|s| {
            if let Some(flag) = s.flags.get_mut(key) {
                flag.enabled = false;
            }
        });
    }

    /// Toggle a flag.
    pub fn toggle(&self, key: &str) {
        self.state.update(|s| {
            if let Some(flag) = s.flags.get_mut(key) {
                flag.enabled = !flag.enabled;
            }
        });
    }
}

impl Store for FeatureFlagStore {
    type State = FeatureFlagState;

    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}

// ============================================================================
// Hydration Support
// ============================================================================

#[cfg(feature = "hydrate")]
impl crate::hydration::HydratableStore for FeatureFlagStore {
    fn serialize_state(&self) -> Result<String, crate::hydration::StoreHydrationError> {
        let state = self.state.get_untracked();
        serde_json::to_string(&state)
            .map_err(|e| crate::hydration::StoreHydrationError::Serialization(e.to_string()))
    }

    fn from_hydrated_state(data: &str) -> Result<Self, crate::hydration::StoreHydrationError> {
        let state: FeatureFlagState = serde_json::from_str(data)
            .map_err(|e| crate::hydration::StoreHydrationError::Deserialization(e.to_string()))?;
        Ok(Self {
            state: RwSignal::new(state),
        })
    }

    fn store_key() -> &'static str {
        "feature_flags"
    }
}

// ============================================================================
// Feature Component
// ============================================================================

/// Component that conditionally renders children based on a feature flag.
///
/// # Example
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_store::templates::feature_flags::*;
///
/// #[component]
/// fn App() -> impl IntoView {
///     view! {
///         <Feature flag="new_feature">
///             <NewFeatureComponent />
///         </Feature>
///
///         // With fallback
///         <Feature flag="premium_feature">
///             <PremiumContent />
///             <Fallback slot>
///                 <UpgradePrompt />
///             </Fallback>
///         </Feature>
///     }
/// }
/// ```
#[component]
pub fn Feature(
    /// The feature flag key to check.
    flag: &'static str,
    /// Whether to invert the condition (render if flag is disabled).
    #[prop(optional)]
    invert: bool,
    /// Children to render if the flag check passes.
    children: ChildrenFn,
) -> impl IntoView {
    let store = use_context::<FeatureFlagStore>();

    let is_enabled = move || {
        store
            .as_ref()
            .map(|s| {
                let enabled = s.is_enabled(flag);
                if invert { !enabled } else { enabled }
            })
            .unwrap_or(false)
    };

    move || {
        if is_enabled() {
            children().into_any()
        } else {
            ().into_any()
        }
    }
}

/// Component that renders content based on a flag variant.
///
/// # Example
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_store::templates::feature_flags::*;
///
/// #[component]
/// fn App() -> impl IntoView {
///     view! {
///         <FeatureVariant flag="hero_style" variant="modern">
///             <ModernHero />
///         </FeatureVariant>
///         <FeatureVariant flag="hero_style" variant="classic">
///             <ClassicHero />
///         </FeatureVariant>
///     }
/// }
/// ```
#[component]
pub fn FeatureVariant(
    /// The feature flag key.
    flag: &'static str,
    /// The variant to match.
    variant: &'static str,
    /// Children to render if the variant matches.
    children: ChildrenFn,
) -> impl IntoView {
    let store = use_context::<FeatureFlagStore>();

    let matches = move || {
        store
            .as_ref()
            .and_then(|s| s.get_variant(flag))
            .map(|v| v == variant)
            .unwrap_or(false)
    };

    move || {
        if matches() {
            children().into_any()
        } else {
            ().into_any()
        }
    }
}

// ============================================================================
// Context Helpers
// ============================================================================

/// Provide a feature flag store to the component tree.
pub fn provide_feature_flags(store: FeatureFlagStore) {
    provide_context(store);
}

/// Access the feature flag store from context.
pub fn use_feature_flags() -> FeatureFlagStore {
    use_context::<FeatureFlagStore>().expect("FeatureFlagStore not found in context")
}

/// Check if a feature is enabled (convenience function).
pub fn use_feature(flag: &'static str) -> impl Fn() -> bool + Clone {
    let store = use_context::<FeatureFlagStore>();
    move || store.as_ref().map(|s| s.is_enabled(flag)).unwrap_or(false)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_flag_creation() {
        let flag = FeatureFlag::new("test", true);
        assert_eq!(flag.key, "test");
        assert!(flag.enabled);
        assert!(flag.variant.is_none());
    }

    #[test]
    fn test_feature_flag_with_variant() {
        let flag = FeatureFlag::with_variant("ab_test", true, "variant_a");
        assert_eq!(flag.key, "ab_test");
        assert_eq!(flag.variant, Some("variant_a".to_string()));
    }

    #[test]
    fn test_feature_flag_with_metadata() {
        let flag = FeatureFlag::new("test", true)
            .with_description("Test flag")
            .with_metadata("owner", "team-a");

        assert_eq!(flag.description, Some("Test flag".to_string()));
        assert_eq!(flag.metadata.get("owner"), Some(&"team-a".to_string()));
    }

    #[test]
    fn test_user_context() {
        let ctx = UserContext::authenticated("user123")
            .with_email("test@example.com")
            .with_environment("production")
            .with_attribute("plan", "premium");

        assert_eq!(ctx.user_id, Some("user123".to_string()));
        assert_eq!(ctx.email, Some("test@example.com".to_string()));
        assert_eq!(ctx.environment, Some("production".to_string()));
        assert_eq!(ctx.attributes.get("plan"), Some(&"premium".to_string()));
    }

    #[test]
    fn test_feature_flag_store_creation() {
        let store = FeatureFlagStore::new();
        assert!(!store.is_loaded());
        assert!(!store.is_loading());
    }

    #[test]
    fn test_feature_flag_store_set_flags() {
        let store = FeatureFlagStore::new();
        store.set_flags(vec![
            FeatureFlag::new("flag1", true),
            FeatureFlag::new("flag2", false),
        ]);

        assert!(store.is_loaded());
        assert!(store.is_enabled("flag1"));
        assert!(!store.is_enabled("flag2"));
        assert!(!store.is_enabled("flag3")); // Unknown flag
    }

    #[test]
    fn test_feature_flag_store_overrides() {
        let store = FeatureFlagStore::new();
        store.set_flags(vec![FeatureFlag::new("feature", false)]);

        // Flag is disabled by default
        assert!(!store.is_enabled("feature"));

        // Override to enabled
        store.set_override("feature", true);
        assert!(store.is_enabled("feature"));

        // Remove override
        store.remove_override("feature");
        assert!(!store.is_enabled("feature"));

        // Clear all overrides
        store.set_override("feature", true);
        store.clear_overrides();
        assert!(!store.is_enabled("feature"));
    }

    #[test]
    fn test_feature_flag_store_variants() {
        let store = FeatureFlagStore::new();
        store.set_flags(vec![FeatureFlag::with_variant("hero", true, "modern")]);

        assert_eq!(store.get_variant("hero"), Some("modern".to_string()));
        assert_eq!(store.get_variant("unknown"), None);
    }

    #[test]
    fn test_feature_flag_store_toggle() {
        let store = FeatureFlagStore::new();
        store.set_flags(vec![FeatureFlag::new("test", false)]);

        assert!(!store.is_enabled("test"));

        store.toggle("test");
        assert!(store.is_enabled("test"));

        store.toggle("test");
        assert!(!store.is_enabled("test"));
    }

    #[test]
    fn test_feature_flag_store_enable_disable() {
        let store = FeatureFlagStore::new();
        store.set_flags(vec![FeatureFlag::new("test", false)]);

        store.enable("test");
        assert!(store.is_enabled("test"));

        store.disable("test");
        assert!(!store.is_enabled("test"));
    }

    #[test]
    fn test_feature_flag_store_with_flags() {
        let store = FeatureFlagStore::with_flags(vec![
            FeatureFlag::new("a", true),
            FeatureFlag::new("b", false),
        ]);

        assert!(store.is_loaded());
        assert_eq!(store.flag_keys().len(), 2);
    }

    #[test]
    fn test_feature_flag_store_remove_flag() {
        let store = FeatureFlagStore::with_flags(vec![
            FeatureFlag::new("a", true),
            FeatureFlag::new("b", true),
        ]);

        assert_eq!(store.all_flags().len(), 2);

        store.remove_flag("a");
        assert_eq!(store.all_flags().len(), 1);
        assert!(store.get_flag("a").is_none());
    }

    #[test]
    fn test_flag_error_display() {
        assert!(
            FlagError::NotFound("test".to_string())
                .to_string()
                .contains("not found")
        );
        assert!(
            FlagError::FetchFailed("timeout".to_string())
                .to_string()
                .contains("fetch")
        );
    }
}
