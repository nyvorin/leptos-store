// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Authentication Store
//!
//! This module defines the authentication store following the **Enterprise Mode** pattern:
//!
//! - **Getters**: Public, read-only derived values
//! - **Mutators**: **Private**, internal state modification only
//! - **Actions**: **Public**, the only external API for writes
//!
//! This pattern ensures that external code cannot bypass business logic
//! by calling mutators directly. All state changes must go through actions.
//!
//! # Architecture
//!
//! ```text
//! External Code (Components)
//!         │
//!         ▼
//!    ┌─────────────┐
//!    │   Actions   │  ← login(), logout() - PUBLIC
//!    └──────┬──────┘
//!           │
//!    ┌──────▼──────┐
//!    │  Mutators   │  ← set_user(), set_loading() - PRIVATE
//!    └──────┬──────┘
//!           │
//!    ┌──────▼──────┐
//!    │ RwSignal    │  ← Private field
//!    └─────────────┘
//! ```
//!
//! # Hydration Support
//!
//! This store supports SSR hydration when the `hydrate` feature is enabled.
//! The state types derive `Serialize` and `Deserialize` for state transfer
//! between server and client.

use leptos::prelude::*;
use leptos_store::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// ============================================================================
// Types
// ============================================================================

/// Represents a logged-in user.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
}

/// Authentication token.
///
/// Note: In a real application, you might want to use `#[serde(skip)]`
/// on sensitive fields or not serialize tokens at all.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: u64,
}

/// Login credentials.
///
/// Note: Password is skipped during serialization for security.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    #[serde(skip)]
    pub password: String,
    pub remember_me: bool,
}

/// Registration data.
#[derive(Clone, Debug)]
pub struct RegistrationData {
    pub email: String,
    pub password: String,
    pub name: String,
}

// ============================================================================
// Errors
// ============================================================================

/// Authentication errors.
#[derive(Debug, Error, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthError {
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Email already exists")]
    EmailExists,

    #[error("Token expired")]
    TokenExpired,

    #[error("Network error: {0}")]
    Network(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

// ============================================================================
// State
// ============================================================================

/// Authentication state.
///
/// This state is serializable for SSR hydration support.
/// Note: `loading` is skipped during serialization as it's transient state.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AuthState {
    /// Current user (None if not logged in).
    pub user: Option<User>,

    /// Authentication token.
    pub token: Option<AuthToken>,

    /// Whether authentication is in progress.
    /// Skipped during hydration - always starts as false on client.
    #[serde(skip)]
    pub loading: bool,

    /// Last error that occurred.
    /// Skipped during hydration - errors should be re-triggered if needed.
    #[serde(skip)]
    pub error: Option<AuthError>,

    /// Whether "remember me" is enabled.
    pub remember_me: bool,
}

impl AuthState {
    /// Check if user is authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.user.is_some() && self.token.is_some()
    }

    /// Check if token is expired.
    pub fn is_token_expired(&self) -> bool {
        self.token
            .as_ref()
            .map(|t| {
                // In a real app, compare with current timestamp
                t.expires_at == 0
            })
            .unwrap_or(true)
    }
}

// ============================================================================
// Store
// ============================================================================

/// Authentication store.
///
/// This store manages all authentication-related state including:
/// - Current user
/// - Authentication token
/// - Loading state
/// - Error state
///
/// # Example
///
/// ```rust,ignore
/// use auth_store_example::AuthStore;
/// use leptos_store::prelude::*;
///
/// let store = AuthStore::new();
/// provide_store(store);
///
/// // In a component
/// let store = use_store::<AuthStore>();
/// let is_logged_in = store.is_authenticated();
/// ```
#[derive(Clone)]
pub struct AuthStore {
    state: RwSignal<AuthState>,
}

impl Default for AuthStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthStore {
    /// Create a new authentication store.
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(AuthState::default()),
        }
    }

    /// Create a store with pre-existing state (useful for SSR hydration).
    pub fn with_state(state: AuthState) -> Self {
        Self {
            state: RwSignal::new(state),
        }
    }

    /// Get the state without tracking reactivity.
    ///
    /// Used for serialization during SSR hydration.
    pub fn get_state_untracked(&self) -> AuthState {
        self.state.get_untracked()
    }

    // ========================================================================
    // Getters
    // ========================================================================

    /// Check if user is authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.state.with(|s| s.is_authenticated())
    }

    /// Get current user if logged in.
    pub fn current_user(&self) -> Option<User> {
        self.state.with(|s| s.user.clone())
    }

    /// Get user's display name or a default.
    pub fn display_name(&self) -> String {
        self.state.with(|s| {
            s.user
                .as_ref()
                .map(|u| u.name.clone())
                .unwrap_or_else(|| "Guest".to_string())
        })
    }

    /// Get user's email if logged in.
    pub fn user_email(&self) -> Option<String> {
        self.state
            .with(|s| s.user.as_ref().map(|u| u.email.clone()))
    }

    /// Get user's initials for avatar placeholder.
    pub fn user_initials(&self) -> String {
        self.state.with(|s| {
            s.user
                .as_ref()
                .map(|u| {
                    u.name
                        .split_whitespace()
                        .filter_map(|word| word.chars().next())
                        .take(2)
                        .collect::<String>()
                        .to_uppercase()
                })
                .unwrap_or_else(|| "?".to_string())
        })
    }

    /// Check if loading.
    pub fn is_loading(&self) -> bool {
        self.state.with(|s| s.loading)
    }

    /// Get the current error.
    pub fn error(&self) -> Option<AuthError> {
        self.state.with(|s| s.error.clone())
    }

    /// Check if there's an error.
    pub fn has_error(&self) -> bool {
        self.state.with(|s| s.error.is_some())
    }

    // ========================================================================
    // Mutators - PRIVATE
    // ========================================================================
    //
    // These methods are internal only. External code cannot call them directly.
    // All state changes must go through the public Actions below.
    // This enforces the Enterprise Mode pattern:
    //   - Getters: public, read-only
    //   - Mutators: private, internal state modification
    //   - Actions: public, the only external write API

    /// Set the current user. (PRIVATE - use actions instead)
    #[allow(dead_code)]
    fn set_user(&self, user: Option<User>) {
        self.state.update(|s| s.user = user);
    }

    /// Set the authentication token. (PRIVATE - use actions instead)
    #[allow(dead_code)]
    fn set_token(&self, token: Option<AuthToken>) {
        self.state.update(|s| s.token = token);
    }

    /// Set loading state. (PRIVATE - use actions instead)
    fn set_loading(&self, loading: bool) {
        self.state.update(|s| s.loading = loading);
    }

    /// Set error state. (PRIVATE - use actions instead)
    fn set_error(&self, error: Option<AuthError>) {
        self.state.update(|s| s.error = error);
    }

    /// Clear error state. (PRIVATE - use actions instead)
    fn clear_error(&self) {
        self.state.update(|s| s.error = None);
    }

    /// Set remember me preference. (PRIVATE - use actions instead)
    fn set_remember_me(&self, remember: bool) {
        self.state.update(|s| s.remember_me = remember);
    }

    /// Set authenticated state (user + token together). (PRIVATE)
    fn set_authenticated(&self, user: User, token: AuthToken) {
        self.state.update(|s| {
            s.user = Some(user);
            s.token = Some(token);
            s.error = None;
            s.loading = false;
        });
    }

    /// Clear all authentication state. (PRIVATE - use logout() action instead)
    fn clear_auth(&self) {
        self.state.update(|s| {
            s.user = None;
            s.token = None;
            s.error = None;
            s.loading = false;
        });
    }

    // ========================================================================
    // Actions - PUBLIC API
    // ========================================================================
    //
    // These are the only methods external code should call to modify state.
    // Actions orchestrate private mutators to ensure business logic is enforced.

    /// Perform login action.
    ///
    /// This is a simulated login - in a real app, this would call an API.
    pub fn login(&self, credentials: LoginCredentials) {
        self.set_loading(true);
        self.clear_error();

        // Simulate API call delay would happen in async action
        // For demo, we do synchronous validation

        // Validate credentials
        if credentials.email.is_empty() {
            self.set_error(Some(AuthError::Validation("Email is required".to_string())));
            self.set_loading(false);
            return;
        }

        if credentials.password.is_empty() {
            self.set_error(Some(AuthError::Validation(
                "Password is required".to_string(),
            )));
            self.set_loading(false);
            return;
        }

        // Simulate successful login
        // In a real app, this would be an async API call
        let user = User {
            id: "user_123".to_string(),
            email: credentials.email.clone(),
            name: credentials
                .email
                .split('@')
                .next()
                .unwrap_or("User")
                .to_string(),
            avatar_url: None,
        };

        let token = AuthToken {
            access_token: "mock_access_token_xyz".to_string(),
            refresh_token: Some("mock_refresh_token_abc".to_string()),
            expires_at: 3600, // 1 hour
        };

        self.set_remember_me(credentials.remember_me);
        self.set_authenticated(user, token);
    }

    /// Perform logout action.
    pub fn logout(&self) {
        self.set_loading(true);

        // In a real app, you might want to:
        // 1. Call logout API
        // 2. Clear local storage
        // 3. Clear cookies

        self.clear_auth();
    }

    /// Attempt to restore session from storage.
    ///
    /// In a real app, this would check localStorage/cookies.
    pub fn restore_session(&self) -> bool {
        // Simulated - in real app, check storage
        false
    }
}

impl Store for AuthStore {
    type State = AuthState;

    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}

// ============================================================================
// Hydration Support
// ============================================================================

/// Implement HydratableStore for SSR hydration support.
///
/// This allows the AuthStore to:
/// - Serialize its state on the server
/// - Deserialize and restore state on the client during hydration
#[cfg(feature = "hydrate")]
impl leptos_store::hydration::HydratableStore for AuthStore {
    fn serialize_state(&self) -> Result<String, leptos_store::hydration::StoreHydrationError> {
        // Use get_untracked() since we're intentionally reading outside reactive context
        // during serialization for SSR hydration
        let state = self.state.get_untracked();
        serde_json::to_string(&state)
            .map_err(|e| leptos_store::hydration::StoreHydrationError::Serialization(e.to_string()))
    }

    fn from_hydrated_state(
        data: &str,
    ) -> Result<Self, leptos_store::hydration::StoreHydrationError> {
        let state: AuthState = serde_json::from_str(data).map_err(|e| {
            leptos_store::hydration::StoreHydrationError::Deserialization(e.to_string())
        })?;
        Ok(Self::with_state(state))
    }

    fn store_key() -> &'static str {
        "auth_store"
    }
}

// ============================================================================
// Tests
// ============================================================================
//
// Note: Tests in this module can call private mutators because they're in the
// same module. This is intentional for unit testing. External code (in other
// modules/crates) cannot access private mutators - they must use public actions.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_store_creation() {
        let store = AuthStore::new();
        assert!(!store.is_authenticated());
        assert!(!store.is_loading());
        assert!(!store.has_error());
    }

    #[test]
    fn test_auth_store_getters() {
        let store = AuthStore::new();

        // Default state
        assert_eq!(store.display_name(), "Guest");
        assert_eq!(store.user_initials(), "?");
        assert!(store.current_user().is_none());
        assert!(store.user_email().is_none());
    }

    #[test]
    fn test_auth_store_set_user() {
        let store = AuthStore::new();

        let user = User {
            id: "1".to_string(),
            email: "test@example.com".to_string(),
            name: "John Doe".to_string(),
            avatar_url: None,
        };

        store.set_user(Some(user));

        assert_eq!(store.display_name(), "John Doe");
        assert_eq!(store.user_initials(), "JD");
        assert_eq!(store.user_email(), Some("test@example.com".to_string()));
    }

    #[test]
    fn test_auth_store_set_authenticated() {
        let store = AuthStore::new();

        let user = User {
            id: "1".to_string(),
            email: "test@example.com".to_string(),
            name: "Jane Smith".to_string(),
            avatar_url: None,
        };

        let token = AuthToken {
            access_token: "token123".to_string(),
            refresh_token: None,
            expires_at: 3600,
        };

        store.set_authenticated(user, token);

        assert!(store.is_authenticated());
        assert!(!store.is_loading());
        assert!(!store.has_error());
    }

    #[test]
    fn test_auth_store_clear_auth() {
        let store = AuthStore::new();

        // Set up authenticated state
        let user = User {
            id: "1".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            avatar_url: None,
        };

        let token = AuthToken {
            access_token: "token".to_string(),
            refresh_token: None,
            expires_at: 3600,
        };

        store.set_authenticated(user, token);
        assert!(store.is_authenticated());

        // Clear auth
        store.clear_auth();
        assert!(!store.is_authenticated());
        assert!(store.current_user().is_none());
    }

    #[test]
    fn test_auth_store_login_validation() {
        let store = AuthStore::new();

        // Empty email
        store.login(LoginCredentials {
            email: "".to_string(),
            password: "password".to_string(),
            remember_me: false,
        });

        assert!(store.has_error());
        assert!(!store.is_authenticated());

        store.clear_error();

        // Empty password
        store.login(LoginCredentials {
            email: "test@example.com".to_string(),
            password: "".to_string(),
            remember_me: false,
        });

        assert!(store.has_error());
        assert!(!store.is_authenticated());
    }

    #[test]
    fn test_auth_store_login_success() {
        let store = AuthStore::new();

        store.login(LoginCredentials {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            remember_me: true,
        });

        assert!(store.is_authenticated());
        assert!(!store.has_error());
        assert_eq!(store.user_email(), Some("test@example.com".to_string()));
    }

    #[test]
    fn test_auth_store_logout() {
        let store = AuthStore::new();

        // Login first
        store.login(LoginCredentials {
            email: "test@example.com".to_string(),
            password: "password".to_string(),
            remember_me: false,
        });

        assert!(store.is_authenticated());

        // Logout
        store.logout();

        assert!(!store.is_authenticated());
    }

    #[test]
    fn test_user_initials() {
        let store = AuthStore::new();

        // Single word name
        store.set_user(Some(User {
            id: "1".to_string(),
            email: "test@example.com".to_string(),
            name: "Alice".to_string(),
            avatar_url: None,
        }));
        assert_eq!(store.user_initials(), "A");

        // Two word name
        store.set_user(Some(User {
            id: "1".to_string(),
            email: "test@example.com".to_string(),
            name: "Bob Smith".to_string(),
            avatar_url: None,
        }));
        assert_eq!(store.user_initials(), "BS");

        // Three word name (takes first two)
        store.set_user(Some(User {
            id: "1".to_string(),
            email: "test@example.com".to_string(),
            name: "Charlie David Evans".to_string(),
            avatar_url: None,
        }));
        assert_eq!(store.user_initials(), "CD");
    }

    #[test]
    fn test_auth_error_display() {
        assert_eq!(
            AuthError::InvalidCredentials.to_string(),
            "Invalid credentials"
        );
        assert_eq!(AuthError::UserNotFound.to_string(), "User not found");
        assert_eq!(AuthError::EmailExists.to_string(), "Email already exists");
        assert_eq!(AuthError::TokenExpired.to_string(), "Token expired");
        assert_eq!(
            AuthError::Network("Connection failed".to_string()).to_string(),
            "Network error: Connection failed"
        );
    }

    // ========================================================================
    // Serialization Tests (for hydration support)
    // ========================================================================

    #[test]
    fn test_user_serialization_roundtrip() {
        let user = User {
            id: "user_123".to_string(),
            email: "test@example.com".to_string(),
            name: "John Doe".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
        };

        // Serialize
        let json = serde_json::to_string(&user).unwrap();
        assert!(json.contains("user_123"));
        assert!(json.contains("test@example.com"));

        // Deserialize
        let restored: User = serde_json::from_str(&json).unwrap();
        assert_eq!(user, restored);
    }

    #[test]
    fn test_auth_token_serialization_roundtrip() {
        let token = AuthToken {
            access_token: "abc123".to_string(),
            refresh_token: Some("refresh_xyz".to_string()),
            expires_at: 3600,
        };

        let json = serde_json::to_string(&token).unwrap();
        let restored: AuthToken = serde_json::from_str(&json).unwrap();
        assert_eq!(token, restored);
    }

    #[test]
    fn test_auth_state_serialization_roundtrip() {
        let state = AuthState {
            user: Some(User {
                id: "1".to_string(),
                email: "user@test.com".to_string(),
                name: "Test User".to_string(),
                avatar_url: None,
            }),
            token: Some(AuthToken {
                access_token: "token123".to_string(),
                refresh_token: None,
                expires_at: 7200,
            }),
            loading: true,                              // Should be skipped
            error: Some(AuthError::InvalidCredentials), // Should be skipped
            remember_me: true,
        };

        let json = serde_json::to_string(&state).unwrap();

        // Verify skipped fields are not in JSON
        assert!(!json.contains("loading"));
        assert!(!json.contains("error"));

        // Verify included fields are present
        assert!(json.contains("user"));
        assert!(json.contains("remember_me"));

        // Deserialize
        let restored: AuthState = serde_json::from_str(&json).unwrap();

        // User and token should match
        assert_eq!(state.user, restored.user);
        assert_eq!(state.token, restored.token);
        assert_eq!(state.remember_me, restored.remember_me);

        // Skipped fields should be default
        assert!(!restored.loading); // default is false
        assert!(restored.error.is_none()); // default is None
    }

    #[test]
    fn test_auth_state_default_values_on_deser() {
        // Minimal JSON with only required fields
        let json = r#"{"user":null,"token":null,"remember_me":false}"#;

        let state: AuthState = serde_json::from_str(json).unwrap();

        assert!(state.user.is_none());
        assert!(state.token.is_none());
        assert!(!state.loading);
        assert!(state.error.is_none());
        assert!(!state.remember_me);
    }

    #[test]
    fn test_auth_error_serialization() {
        let errors = vec![
            AuthError::InvalidCredentials,
            AuthError::UserNotFound,
            AuthError::EmailExists,
            AuthError::TokenExpired,
            AuthError::Network("connection refused".to_string()),
            AuthError::Validation("email required".to_string()),
            AuthError::Unknown("something went wrong".to_string()),
        ];

        for error in errors {
            let json = serde_json::to_string(&error).unwrap();
            let restored: AuthError = serde_json::from_str(&json).unwrap();
            assert_eq!(error, restored);
        }
    }

    #[test]
    fn test_login_credentials_password_skipped() {
        let creds = LoginCredentials {
            email: "user@test.com".to_string(),
            password: "super_secret_password".to_string(),
            remember_me: true,
        };

        let json = serde_json::to_string(&creds).unwrap();

        // Password should NOT be in the JSON (security)
        assert!(!json.contains("super_secret_password"));
        assert!(!json.contains("password"));

        // Email and remember_me should be present
        assert!(json.contains("user@test.com"));
        assert!(json.contains("remember_me"));
    }

    // ========================================================================
    // Hydration Integration Tests
    // ========================================================================
    // These tests prove that the AuthStore can be serialized on the server
    // and deserialized on the client during SSR hydration.

    #[cfg(feature = "hydrate")]
    mod hydration_tests {
        use super::*;
        use leptos_store::hydration::{
            HydratableStore, hydration_script_html, hydration_script_id,
        };

        #[test]
        fn test_auth_store_hydration_key() {
            assert_eq!(AuthStore::store_key(), "auth_store");
        }

        #[test]
        fn test_auth_store_serialization() {
            // Create store with authenticated user
            let store = AuthStore::new();
            store.set_authenticated(
                User {
                    id: "user_456".to_string(),
                    email: "hydration@test.com".to_string(),
                    name: "Hydration Tester".to_string(),
                    avatar_url: Some("https://example.com/avatar.jpg".to_string()),
                },
                AuthToken {
                    access_token: "hydration_token_xyz".to_string(),
                    refresh_token: Some("refresh_abc".to_string()),
                    expires_at: 7200,
                },
            );
            store.set_remember_me(true);

            // Serialize the store
            let json = store.serialize_state().expect("Serialization should work");

            // Verify JSON contains the important data
            assert!(json.contains("user_456"));
            assert!(json.contains("hydration@test.com"));
            assert!(json.contains("Hydration Tester"));
            assert!(json.contains("hydration_token_xyz"));
            assert!(json.contains("remember_me"));

            // Verify transient fields are NOT in JSON
            assert!(!json.contains("loading"));
            assert!(!json.contains("error"));
        }

        #[test]
        fn test_auth_store_deserialization() {
            // Simulate JSON that would come from the server
            let server_json = r#"{
                "user": {
                    "id": "server_user_1",
                    "email": "server@example.com",
                    "name": "Server User",
                    "avatar_url": null
                },
                "token": {
                    "access_token": "server_token_123",
                    "refresh_token": null,
                    "expires_at": 3600
                },
                "remember_me": true
            }"#;

            // Deserialize into a new store (simulating client hydration)
            let hydrated_store =
                AuthStore::from_hydrated_state(server_json).expect("Hydration should succeed");

            // Verify the state was correctly restored
            assert!(hydrated_store.is_authenticated());
            assert_eq!(
                hydrated_store.user_email(),
                Some("server@example.com".to_string())
            );
            assert_eq!(hydrated_store.display_name(), "Server User");

            // Transient fields should be at default values
            assert!(!hydrated_store.is_loading());
            assert!(!hydrated_store.has_error());
        }

        #[test]
        fn test_auth_store_full_roundtrip() {
            // === SERVER SIDE ===
            // Create and populate a store on the "server"
            let server_store = AuthStore::new();
            server_store.set_authenticated(
                User {
                    id: "roundtrip_user".to_string(),
                    email: "roundtrip@test.com".to_string(),
                    name: "Roundtrip Test".to_string(),
                    avatar_url: None,
                },
                AuthToken {
                    access_token: "roundtrip_token".to_string(),
                    refresh_token: Some("roundtrip_refresh".to_string()),
                    expires_at: 86400,
                },
            );
            server_store.set_remember_me(true);

            // Set some transient state that should NOT be hydrated
            server_store.set_loading(true);
            server_store.set_error(Some(AuthError::Network("test error".to_string())));

            // Serialize for transfer to client
            let serialized = server_store
                .serialize_state()
                .expect("Server serialization should succeed");

            // === CLIENT SIDE ===
            // Deserialize on the "client"
            let client_store = AuthStore::from_hydrated_state(&serialized)
                .expect("Client hydration should succeed");

            // Verify important state was transferred
            assert!(client_store.is_authenticated());
            assert_eq!(
                client_store.user_email(),
                Some("roundtrip@test.com".to_string())
            );
            assert_eq!(client_store.display_name(), "Roundtrip Test");

            let user = client_store.current_user().expect("Should have user");
            assert_eq!(user.id, "roundtrip_user");

            // Verify transient state was NOT transferred (reset to defaults)
            assert!(!client_store.is_loading()); // Should be false, not true
            assert!(!client_store.has_error()); // Should be None, not Some(...)
        }

        #[test]
        fn test_auth_store_hydration_html_generation() {
            let store = AuthStore::new();
            store.set_user(Some(User {
                id: "html_test".to_string(),
                email: "html@test.com".to_string(),
                name: "HTML Test".to_string(),
                avatar_url: None,
            }));

            let serialized = store.serialize_state().unwrap();
            let html = hydration_script_html(AuthStore::store_key(), &serialized);

            // Verify HTML structure
            assert!(html.starts_with("<script"));
            assert!(html.ends_with("</script>"));
            assert!(html.contains(&hydration_script_id(AuthStore::store_key())));
            assert!(html.contains("application/json"));

            // Verify the data is embedded
            assert!(html.contains("html_test"));
            assert!(html.contains("html@test.com"));
        }

        #[test]
        fn test_auth_store_unauthenticated_roundtrip() {
            // Test that unauthenticated state roundtrips correctly
            let server_store = AuthStore::new();
            assert!(!server_store.is_authenticated());

            let serialized = server_store.serialize_state().unwrap();
            let client_store = AuthStore::from_hydrated_state(&serialized).unwrap();

            assert!(!client_store.is_authenticated());
            assert!(client_store.current_user().is_none());
            assert_eq!(client_store.display_name(), "Guest");
        }

        #[test]
        fn test_hydration_preserves_avatar_url() {
            let avatar = "https://cdn.example.com/avatars/user123.png?size=200&format=webp";

            let server_store = AuthStore::new();
            server_store.set_user(Some(User {
                id: "avatar_user".to_string(),
                email: "avatar@test.com".to_string(),
                name: "Avatar User".to_string(),
                avatar_url: Some(avatar.to_string()),
            }));

            let serialized = server_store.serialize_state().unwrap();
            let client_store = AuthStore::from_hydrated_state(&serialized).unwrap();

            let user = client_store.current_user().unwrap();
            assert_eq!(user.avatar_url, Some(avatar.to_string()));
        }

        #[test]
        fn test_hydration_with_special_characters() {
            // Test that special characters in user data are handled correctly
            let server_store = AuthStore::new();
            server_store.set_user(Some(User {
                id: "special_<user>".to_string(),
                email: "test+special@example.com".to_string(),
                name: r#"Test "User" <Name>"#.to_string(),
                avatar_url: Some("https://example.com/avatar?name=<test>&id=123".to_string()),
            }));

            let serialized = server_store.serialize_state().unwrap();
            let client_store = AuthStore::from_hydrated_state(&serialized).unwrap();

            let user = client_store.current_user().unwrap();
            assert_eq!(user.id, "special_<user>");
            assert_eq!(user.name, r#"Test "User" <Name>"#);
        }

        #[test]
        fn test_hydration_error_on_invalid_json() {
            let result = AuthStore::from_hydrated_state("not valid json");
            assert!(result.is_err());

            match result {
                Err(leptos_store::hydration::StoreHydrationError::Deserialization(msg)) => {
                    assert!(!msg.is_empty());
                }
                _ => panic!("Expected Deserialization error"),
            }
        }

        #[test]
        fn test_hydration_error_on_wrong_structure() {
            let result = AuthStore::from_hydrated_state(r#"{"completely":"wrong"}"#);
            assert!(result.is_err());
        }
    }
}
