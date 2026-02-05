// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Macros for store definition.
//!
//! This module provides declarative macros for defining stores, state, actions,
//! and async actions with less boilerplate while maintaining type safety.
//!
//! # Available Macros
//!
//! | Macro | Purpose | Feature |
//! |-------|---------|---------|
//! | `define_state!` | Define state structs with default values | - |
//! | `define_hydratable_state!` | Define state with serde derives for hydration | `hydrate` |
//! | `define_action!` | Define synchronous action structs | - |
//! | `define_async_action!` | Define async action structs with error types | - |
//! | `impl_store!` | Implement Store trait for a type | - |
//! | `impl_hydratable_store!` | Implement HydratableStore trait | `hydrate` |
//! | `store!` | Complete store definition in one macro | - |
//!
//! # Quick Start
//!
//! ## Defining State
//!
//! ```rust
//! use leptos_store::define_state;
//!
//! define_state! {
//!     #[derive(Clone, Debug, PartialEq)]
//!     pub struct TodoState {
//!         items: Vec<String>,
//!         filter: String = "all".to_string(),
//!         loading: bool = false,
//!     }
//! }
//!
//! let state = TodoState::default();
//! assert_eq!(state.filter, "all");
//! assert!(!state.loading);
//! ```
//!
//! ## Defining Actions
//!
//! ```rust
//! use leptos_store::define_action;
//!
//! define_action! {
//!     /// Action to add a new todo item
//!     #[derive(Debug, Clone)]
//!     pub AddTodoAction {
//!         text: String,
//!         priority: u8,
//!     }
//! }
//!
//! let action = AddTodoAction::new("Buy groceries".to_string(), 1);
//! assert_eq!(action.text, "Buy groceries");
//! ```
//!
//! ## Defining Async Actions
//!
//! ```rust
//! use leptos_store::define_async_action;
//! use std::fmt;
//!
//! // Define a custom error type
//! #[derive(Debug, Clone)]
//! struct ApiError(String);
//!
//! impl fmt::Display for ApiError {
//!     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//!         write!(f, "API Error: {}", self.0)
//!     }
//! }
//!
//! impl std::error::Error for ApiError {}
//!
//! // Define the async action
//! define_async_action! {
//!     /// Fetches user data from the API
//!     #[derive(Debug, Clone)]
//!     pub FetchUserAction {
//!         user_id: String,
//!     } -> Result<String, ApiError>
//! }
//!
//! let action = FetchUserAction::new("user_123".to_string());
//! assert_eq!(action.user_id, "user_123");
//! ```

// ============================================================================
// define_state! macro
// ============================================================================

/// Define a state struct with optional default values.
///
/// This macro creates a struct with public fields and generates a `Default`
/// implementation. Fields can have explicit default values or use the type's
/// `Default` implementation.
///
/// # Syntax
///
/// ```text
/// define_state! {
///     #[derive(...)]           // Optional: derive macros
///     pub struct StateName {   // Visibility and name
///         field1: Type1,       // Uses Type1::default()
///         field2: Type2 = val, // Uses explicit value
///     }
/// }
/// ```
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use leptos_store::define_state;
///
/// define_state! {
///     #[derive(Clone, Debug, PartialEq)]
///     pub struct UserState {
///         name: String,
///         email: Option<String>,
///         age: u32 = 0,
///         active: bool = true,
///     }
/// }
///
/// let user = UserState::default();
/// assert_eq!(user.name, "");
/// assert_eq!(user.email, None);
/// assert_eq!(user.age, 0);
/// assert!(user.active);
/// ```
///
/// ## With Complex Types
///
/// ```rust
/// use leptos_store::define_state;
/// use std::collections::HashMap;
///
/// define_state! {
///     #[derive(Clone, Debug)]
///     pub struct CacheState {
///         entries: HashMap<String, String>,
///         max_size: usize = 100,
///         hits: u64 = 0,
///         misses: u64 = 0,
///     }
/// }
///
/// let cache = CacheState::default();
/// assert_eq!(cache.max_size, 100);
/// assert!(cache.entries.is_empty());
/// ```
///
/// ## With Field Attributes
///
/// ```rust
/// use leptos_store::define_state;
///
/// define_state! {
///     #[derive(Clone, Debug)]
///     pub struct FormState {
///         /// The user's username
///         username: String,
///         /// Password (not serialized)
///         #[allow(dead_code)]
///         password: String,
///         /// Remember user preference
///         remember_me: bool = false,
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_state {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $ty:ty $(= $default:expr)?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field: $ty,
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $(
                        $field: $crate::define_state!(@default $ty $(, $default)?),
                    )*
                }
            }
        }
    };

    // Default value helper - with explicit default
    (@default $ty:ty, $default:expr) => { $default };

    // Default value helper - use Default trait
    (@default $ty:ty) => { <$ty as Default>::default() };
}

// ============================================================================
// define_hydratable_state! macro (hydrate feature)
// ============================================================================

/// Define a hydratable state struct with serde derives.
///
/// This macro is similar to [`define_state!`] but automatically adds
/// `serde::Serialize` and `serde::Deserialize` derives for hydration support.
///
/// This macro is only available when the `hydrate` feature is enabled.
///
/// # Syntax
///
/// ```text
/// define_hydratable_state! {
///     #[derive(...)]           // Optional: additional derive macros
///     pub struct StateName {   // Visibility and name
///         field1: Type1,       // Uses Type1::default()
///         field2: Type2 = val, // Uses explicit value
///     }
/// }
/// ```
///
/// # Note on Serde Attributes
///
/// You can use serde attributes to customize serialization:
///
/// ```rust,ignore
/// define_hydratable_state! {
///     #[derive(Clone, Debug)]
///     pub struct SessionState {
///         user_id: String,
///         #[serde(skip)]  // Don't serialize sensitive data
///         password_hash: String,
///         #[serde(default)]  // Use default if missing during deser
///         remember_me: bool = false,
///     }
/// }
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::define_hydratable_state;
///
/// define_hydratable_state! {
///     #[derive(Clone, Debug, PartialEq)]
///     pub struct CounterState {
///         count: i32 = 0,
///         step: i32 = 1,
///     }
/// }
///
/// let state = CounterState::default();
/// assert_eq!(state.count, 0);
///
/// // Serialize to JSON
/// let json = serde_json::to_string(&state).unwrap();
/// assert!(json.contains("\"count\":0"));
///
/// // Deserialize from JSON
/// let restored: CounterState = serde_json::from_str(&json).unwrap();
/// assert_eq!(restored.count, 0);
/// ```
#[cfg(feature = "hydrate")]
#[macro_export]
macro_rules! define_hydratable_state {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field:ident : $ty:ty $(= $default:expr)?
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(::serde::Serialize, ::serde::Deserialize)]
        $vis struct $name {
            $(
                $(#[$field_meta])*
                $field_vis $field: $ty,
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $(
                        $field: $crate::define_hydratable_state!(@default $ty $(, $default)?),
                    )*
                }
            }
        }
    };

    // Default value helper - with explicit default
    (@default $ty:ty, $default:expr) => { $default };

    // Default value helper - use Default trait
    (@default $ty:ty) => { <$ty as Default>::default() };
}

/// Implement the HydratableStore trait for a store type.
///
/// This macro provides a quick way to implement the [`HydratableStore`](crate::hydration::HydratableStore)
/// trait for a store that:
/// - Already implements [`Store`](crate::store::Store)
/// - Has a state type that implements `serde::Serialize` and `serde::DeserializeOwned`
///
/// # Syntax
///
/// ```text
/// impl_hydratable_store!(StoreName, "store_key");
/// ```
///
/// # Arguments
///
/// - `StoreName` - The store type to implement HydratableStore for
/// - `"store_key"` - A unique string key for this store (used in DOM)
///
/// # Example
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_store::{impl_store, impl_hydratable_store};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Clone, Default, Serialize, Deserialize)]
/// struct CounterState {
///     count: i32,
/// }
///
/// #[derive(Clone)]
/// struct CounterStore {
///     state: RwSignal<CounterState>,
/// }
///
/// impl CounterStore {
///     pub fn new() -> Self {
///         Self { state: RwSignal::new(CounterState::default()) }
///     }
/// }
///
/// impl_store!(CounterStore, CounterState, state);
/// impl_hydratable_store!(CounterStore, "counter");
/// ```
#[cfg(feature = "hydrate")]
#[macro_export]
macro_rules! impl_hydratable_store {
    ($store:ty, $key:literal) => {
        impl $crate::hydration::HydratableStore for $store {
            fn serialize_state(&self) -> Result<String, $crate::hydration::StoreHydrationError> {
                let state = self.state.get();
                ::serde_json::to_string(&state).map_err(|e| {
                    $crate::hydration::StoreHydrationError::Serialization(e.to_string())
                })
            }

            fn from_hydrated_state(
                data: &str,
            ) -> Result<Self, $crate::hydration::StoreHydrationError> {
                let state: <Self as $crate::store::Store>::State = ::serde_json::from_str(data)
                    .map_err(|e| {
                        $crate::hydration::StoreHydrationError::Deserialization(e.to_string())
                    })?;
                Ok(Self {
                    state: ::leptos::prelude::RwSignal::new(state),
                })
            }

            fn store_key() -> &'static str {
                $key
            }
        }
    };
}

// ============================================================================
// define_action! macro
// ============================================================================

/// Define a synchronous action struct.
///
/// Actions are command objects that encapsulate the data needed to perform
/// an operation. This macro creates a struct with public fields and a
/// constructor method.
///
/// # Syntax
///
/// ```text
/// define_action! {
///     #[derive(...)]           // Optional: derive macros
///     pub ActionName {         // Visibility and name
///         field1: Type1,       // Required field
///         field2: Type2,       // Another field
///     }
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates:
/// - A struct with public fields
/// - A `new()` constructor that takes all fields as arguments
///
/// # Examples
///
/// ## Basic Action
///
/// ```rust
/// use leptos_store::define_action;
///
/// define_action! {
///     #[derive(Debug, Clone)]
///     pub IncrementAction {
///         amount: i32,
///     }
/// }
///
/// let action = IncrementAction::new(5);
/// assert_eq!(action.amount, 5);
/// ```
///
/// ## Action with Multiple Fields
///
/// ```rust
/// use leptos_store::define_action;
///
/// define_action! {
///     /// Updates user profile information
///     #[derive(Debug, Clone, PartialEq)]
///     pub UpdateProfileAction {
///         /// User ID to update
///         user_id: String,
///         /// New display name
///         name: Option<String>,
///         /// New email address
///         email: Option<String>,
///         /// New avatar URL
///         avatar_url: Option<String>,
///     }
/// }
///
/// let action = UpdateProfileAction::new(
///     "user_123".to_string(),
///     Some("John Doe".to_string()),
///     None,
///     None,
/// );
///
/// assert_eq!(action.user_id, "user_123");
/// assert_eq!(action.name, Some("John Doe".to_string()));
/// ```
///
/// ## Action for Form Submission
///
/// ```rust
/// use leptos_store::define_action;
///
/// define_action! {
///     #[derive(Debug, Clone)]
///     pub SubmitFormAction {
///         form_id: String,
///         data: std::collections::HashMap<String, String>,
///         validate: bool,
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_action {
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident : $ty:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                pub $field: $ty,
            )*
        }

        impl $name {
            /// Create a new action with the given parameters.
            pub fn new($($field: $ty),*) -> Self {
                Self { $($field),* }
            }
        }
    };
}

// ============================================================================
// define_async_action! macro
// ============================================================================

/// Define an async action struct with associated result types.
///
/// Async actions are command objects for asynchronous operations like API calls,
/// database queries, or file I/O. This macro creates a struct with fields and
/// type aliases for the success and error types.
///
/// # Syntax
///
/// ```text
/// define_async_action! {
///     #[derive(...)]              // Optional: derive macros
///     pub ActionName {            // Visibility and name
///         field1: Type1,          // Action parameters
///     } -> Result<Output, Error>  // Result type specification
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates:
/// - A struct with public fields
/// - A `new()` constructor
/// - Type aliases: `{ActionName}Output` and `{ActionName}Error`
/// - A `result_type()` method for documentation
///
/// # Examples
///
/// ## API Fetch Action
///
/// ```rust
/// use leptos_store::define_async_action;
/// use std::fmt;
///
/// #[derive(Debug, Clone)]
/// struct User { id: String, name: String }
///
/// #[derive(Debug, Clone)]
/// struct ApiError(String);
/// impl fmt::Display for ApiError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "{}", self.0)
///     }
/// }
/// impl std::error::Error for ApiError {}
///
/// define_async_action! {
///     /// Fetches a user by ID from the API
///     #[derive(Debug, Clone)]
///     pub FetchUserAction {
///         user_id: String,
///         include_profile: bool,
///     } -> Result<User, ApiError>
/// }
///
/// let action = FetchUserAction::new("user_123".to_string(), true);
/// assert_eq!(action.user_id, "user_123");
/// assert!(action.include_profile);
/// ```
///
/// ## Login Action
///
/// ```rust
/// use leptos_store::define_async_action;
/// use std::fmt;
///
/// #[derive(Debug, Clone)]
/// struct AuthToken(String);
///
/// #[derive(Debug, Clone)]
/// enum AuthError {
///     InvalidCredentials,
///     NetworkError(String),
///     RateLimited,
/// }
///
/// impl fmt::Display for AuthError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         match self {
///             Self::InvalidCredentials => write!(f, "Invalid credentials"),
///             Self::NetworkError(e) => write!(f, "Network error: {}", e),
///             Self::RateLimited => write!(f, "Rate limited"),
///         }
///     }
/// }
/// impl std::error::Error for AuthError {}
///
/// define_async_action! {
///     /// Authenticates a user with email and password
///     #[derive(Debug, Clone)]
///     pub LoginAction {
///         email: String,
///         password: String,
///         remember_me: bool,
///     } -> Result<AuthToken, AuthError>
/// }
///
/// let login = LoginAction::new(
///     "user@example.com".to_string(),
///     "password123".to_string(),
///     true,
/// );
/// ```
///
/// ## File Upload Action
///
/// ```rust
/// use leptos_store::define_async_action;
/// use std::fmt;
///
/// #[derive(Debug, Clone)]
/// struct UploadResult { url: String, size: u64 }
///
/// #[derive(Debug, Clone)]
/// struct UploadError(String);
/// impl fmt::Display for UploadError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "{}", self.0)
///     }
/// }
/// impl std::error::Error for UploadError {}
///
/// define_async_action! {
///     #[derive(Debug, Clone)]
///     pub UploadFileAction {
///         filename: String,
///         content_type: String,
///         data: Vec<u8>,
///     } -> Result<UploadResult, UploadError>
/// }
/// ```
#[macro_export]
macro_rules! define_async_action {
    // Version with Result<Output, Error>
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident : $ty:ty
            ),* $(,)?
        } -> Result<$output:ty, $error:ty>
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                pub $field: $ty,
            )*
        }

        impl $name {
            /// Create a new async action with the given parameters.
            pub fn new($($field: $ty),*) -> Self {
                Self { $($field),* }
            }

            /// Returns a description of the result type for documentation.
            #[allow(dead_code)]
            pub fn result_type_description() -> &'static str {
                concat!(
                    "Result<",
                    stringify!($output),
                    ", ",
                    stringify!($error),
                    ">"
                )
            }

            /// Returns the output type name as a string.
            #[allow(dead_code)]
            pub fn output_type_name() -> &'static str {
                stringify!($output)
            }

            /// Returns the error type name as a string.
            #[allow(dead_code)]
            pub fn error_type_name() -> &'static str {
                stringify!($error)
            }
        }
    };

    // Version with just an output type (infallible actions)
    (
        $(#[$meta:meta])*
        $vis:vis $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident : $ty:ty
            ),* $(,)?
        } -> $output:ty
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                pub $field: $ty,
            )*
        }

        impl $name {
            /// Create a new async action with the given parameters.
            pub fn new($($field: $ty),*) -> Self {
                Self { $($field),* }
            }

            /// Returns the output type name as a string.
            #[allow(dead_code)]
            pub fn output_type_name() -> &'static str {
                stringify!($output)
            }
        }
    };
}

// ============================================================================
// impl_store! macro
// ============================================================================

/// Implement the Store trait for an existing type.
///
/// This macro provides a quick way to implement the [`Store`](crate::store::Store)
/// trait for a struct that already has an `RwSignal<State>` field.
///
/// # Syntax
///
/// ```text
/// impl_store!(StoreName, StateName, field_name);
/// ```
///
/// # Arguments
///
/// - `StoreName` - The type to implement Store for
/// - `StateName` - The state type
/// - `field_name` - The name of the RwSignal field
///
/// # Example
///
/// ```rust
/// use leptos::prelude::*;
/// use leptos_store::{impl_store, store::Store};
///
/// #[derive(Clone, Default)]
/// struct CounterState {
///     count: i32,
/// }
///
/// #[derive(Clone)]
/// struct CounterStore {
///     state: RwSignal<CounterState>,
/// }
///
/// impl_store!(CounterStore, CounterState, state);
///
/// // Now CounterStore implements the Store trait
/// ```
#[macro_export]
macro_rules! impl_store {
    ($store:ty, $state:ty, $field:ident) => {
        impl $crate::store::Store for $store {
            type State = $state;

            fn state(&self) -> ::leptos::prelude::ReadSignal<Self::State> {
                self.$field.read_only()
            }
        }
    };
}

// ============================================================================
// store! macro
// ============================================================================

/// Define a complete store with state, getters, mutators, and actions.
///
/// This is the most comprehensive macro that generates a store following
/// the **Enterprise Mode** pattern:
///
/// - A state struct with public fields and Default implementation
/// - A store struct with constructor methods
/// - Store trait implementation (read/write split)
/// - **Public** getter methods for derived state
/// - **Private** mutator methods for internal state changes
/// - **Public** action methods as the only external write API
///
/// # Architecture
///
/// ```text
/// ┌─────────────────────────────────────────────────────────┐
/// │                    External Code                        │
/// │  (Components, Event Handlers, Other Modules)            │
/// └─────────────────────┬───────────────────────────────────┘
///                       │
///            ┌──────────▼──────────┐
///            │   PUBLIC Actions    │  ← Only write API
///            │   increment()       │
///            │   login()           │
///            └──────────┬──────────┘
///                       │
///            ┌──────────▼──────────┐
///            │  PRIVATE Mutators   │  ← Internal only
///            │   set_count()       │
///            │   set_loading()     │
///            └──────────┬──────────┘
///                       │
///            ┌──────────▼──────────┐
///            │   RwSignal<State>   │  ← Private field
///            └─────────────────────┘
/// ```
///
/// # Syntax
///
/// ```text
/// store! {
///     pub StoreName {
///         state StateName {
///             field1: Type1,
///             field2: Type2 = default_value,
///         }
///
///         getters {
///             // PUBLIC - read-only derived values
///             getter_name(this) -> ReturnType {
///                 this.read(|s| s.field)
///             }
///         }
///
///         mutators {
///             // PRIVATE - internal state changes
///             set_field(this, value: Type) {
///                 this.mutate(|s| s.field = value);
///             }
///         }
///
///         actions {
///             // PUBLIC - external write API
///             do_something(this, param: Type) {
///                 this.set_field(param);  // calls private mutator
///             }
///         }
///     }
/// }
/// ```
///
/// # Generated Methods
///
/// - `StoreName::new()` - Create with default state
/// - `StoreName::with_state(state)` - Create with custom state
/// - All getter methods (public)
/// - All mutator methods (private)
/// - All action methods (public)
///
/// # Example - Full Store Definition
///
/// ```rust
/// use leptos_store::store;
///
/// store! {
///     pub TodoStore {
///         state TodoState {
///             items: Vec<String>,
///             filter: String = "all".to_string(),
///         }
///
///         getters {
///             item_count(this) -> usize {
///                 this.read(|s| s.items.len())
///             }
///         }
///
///         mutators {
///             push_item(this, item: String) {
///                 this.mutate(|s| s.items.push(item));
///             }
///             set_filter(this, filter: String) {
///                 this.mutate(|s| s.filter = filter);
///             }
///         }
///
///         actions {
///             add_todo(this, text: String) {
///                 if !text.is_empty() {
///                     this.push_item(text);
///                 }
///             }
///             show_all(this) {
///                 this.set_filter("all".to_string());
///             }
///         }
///     }
/// }
///
/// let store = TodoStore::new();
/// store.add_todo("Buy milk".to_string());  // OK - action is public
/// // store.push_item("...".to_string());   // ERROR - mutator is private
/// assert_eq!(store.item_count(), 1);
/// ```
/// Define a complete store with state, getters, mutators, and actions.
///
/// This macro generates a complete store implementation following the
/// **Enterprise Mode** pattern:
///
/// - **State fields**: Public for reading, but only mutable through mutators
/// - **Getters**: Public, read-only derived values
/// - **Mutators**: **Private** - internal methods for state changes
/// - **Actions**: **Public** - the only external API for writes
///
/// This enforces a clean separation where external code cannot bypass
/// business logic by calling mutators directly.
///
/// # Syntax
///
/// ```text
/// store! {
///     pub StoreName {
///         state StateName {
///             field1: Type1,
///             field2: Type2 = default_value,
///         }
///
///         getters {
///             getter_name(this) -> ReturnType {
///                 this.read(|s| s.field)
///             }
///         }
///
///         mutators {
///             // PRIVATE - only callable from actions within this store
///             set_field(this, value: Type) {
///                 this.mutate(|s| s.field = value);
///             }
///         }
///
///         actions {
///             // PUBLIC - the external API for state changes
///             do_something(this, param: Type) {
///                 // Can call private mutators
///                 this.set_field(param);
///             }
///         }
///     }
/// }
/// ```
///
/// # Note on `this` parameter
///
/// Due to Rust 2024 edition macro hygiene rules, you must use `this` (or any
/// identifier you choose) instead of `self` in getter, mutator, and action bodies.
/// The first parameter in each is bound to `&self`.
///
/// The macro provides two helper methods:
/// - `this.read(|s| ...)` - Read state immutably (for getters)
/// - `this.mutate(|s| ...)` - Update state mutably (for mutators)
///
/// # Example
///
/// ```rust
/// use leptos_store::store;
///
/// store! {
///     pub CounterStore {
///         state CounterState {
///             count: i32 = 0,
///             loading: bool = false,
///         }
///
///         getters {
///             doubled(this) -> i32 {
///                 this.read(|s| s.count * 2)
///             }
///             is_loading(this) -> bool {
///                 this.read(|s| s.loading)
///             }
///         }
///
///         mutators {
///             // Private - can only be called from actions
///             set_count(this, value: i32) {
///                 this.mutate(|s| s.count = value);
///             }
///             set_loading(this, loading: bool) {
///                 this.mutate(|s| s.loading = loading);
///             }
///         }
///
///         actions {
///             // Public API - external code calls these
///             increment(this) {
///                 let current = this.read(|s| s.count);
///                 this.set_count(current + 1);
///             }
///             decrement(this) {
///                 let current = this.read(|s| s.count);
///                 this.set_count(current - 1);
///             }
///             reset(this) {
///                 this.set_count(0);
///                 this.set_loading(false);
///             }
///         }
///     }
/// }
///
/// // External code can only use actions
/// let store = CounterStore::new();
/// store.increment();  // OK - action is public
/// // store.set_count(5);  // ERROR - mutator is private
/// ```
///
/// # Migration from Previous Versions
///
/// If you previously had public mutators, move them to the `actions` section
/// or create actions that delegate to them:
///
/// ```rust,ignore
/// // Before (mutators were public)
/// mutators {
///     increment(this) { this.mutate(|s| s.count += 1); }
/// }
///
/// // After (mutators private, actions public)
/// mutators {
///     add_to_count(this, n: i32) { this.mutate(|s| s.count += n); }
/// }
/// actions {
///     increment(this) { this.add_to_count(1); }
/// }
/// ```
#[macro_export]
macro_rules! store {
    (
        $store_vis:vis $store_name:ident {
            state $state_name:ident {
                $(
                    $field:ident : $field_ty:ty $(= $field_default:expr)?
                ),* $(,)?
            }

            $(
                getters {
                    $(
                        $(#[$getter_meta:meta])*
                        $getter_name:ident ( $getter_self:ident ) -> $getter_ty:ty $getter_body:block
                    )*
                }
            )?

            $(
                mutators {
                    $(
                        $(#[$mutator_meta:meta])*
                        $mutator_name:ident ( $mutator_self:ident $(, $mutator_param:ident : $mutator_param_ty:ty)* ) $mutator_body:block
                    )*
                }
            )?

            $(
                actions {
                    $(
                        $(#[$action_meta:meta])*
                        $action_name:ident ( $action_self:ident $(, $action_param:ident : $action_param_ty:ty)* ) $(-> $action_ret:ty)? $action_body:block
                    )*
                }
            )?
        }
    ) => {
        // Generate state struct
        #[derive(Clone, Debug)]
        $store_vis struct $state_name {
            $(
                pub $field: $field_ty,
            )*
        }

        impl Default for $state_name {
            fn default() -> Self {
                Self {
                    $(
                        $field: $crate::store!(@default $field_ty $(, $field_default)?),
                    )*
                }
            }
        }

        // Generate store struct
        #[derive(Clone)]
        $store_vis struct $store_name {
            state: ::leptos::prelude::RwSignal<$state_name>,
        }

        impl $store_name {
            /// Create a new store with default state.
            pub fn new() -> Self {
                Self {
                    state: ::leptos::prelude::RwSignal::new($state_name::default()),
                }
            }

            /// Create a new store with custom initial state.
            #[allow(dead_code)]
            pub fn with_state(state: $state_name) -> Self {
                Self {
                    state: ::leptos::prelude::RwSignal::new(state),
                }
            }

            // ================================================================
            // Getters - PUBLIC read-only derived values
            // ================================================================
            $(
                $(
                    $(#[$getter_meta])*
                    #[allow(dead_code)]
                    pub fn $getter_name(&self) -> $getter_ty {
                        let $getter_self = self;
                        $getter_body
                    }
                )*
            )?

            // ================================================================
            // Mutators - PRIVATE internal state modification
            // Only callable from within this store (actions, other mutators)
            // ================================================================
            $(
                $(
                    $(#[$mutator_meta])*
                    #[allow(dead_code)]
                    fn $mutator_name(&self $(, $mutator_param: $mutator_param_ty)*) {
                        let $mutator_self = self;
                        $mutator_body
                    }
                )*
            )?

            // ================================================================
            // Actions - PUBLIC write API
            // The only way external code can modify state
            // ================================================================
            $(
                $(
                    $(#[$action_meta])*
                    #[allow(dead_code)]
                    pub fn $action_name(&self $(, $action_param: $action_param_ty)*) $(-> $action_ret)? {
                        let $action_self = self;
                        $action_body
                    }
                )*
            )?

            /// Read state with a closure (for getters).
            /// Uses the With trait internally.
            #[allow(dead_code)]
            #[inline]
            fn read<R>(&self, f: impl FnOnce(&$state_name) -> R) -> R {
                use ::leptos::prelude::With;
                self.state.with(f)
            }

            /// Update state with a closure (for mutators).
            /// Uses the Update trait internally.
            #[allow(dead_code)]
            #[inline]
            fn mutate<R>(&self, f: impl FnOnce(&mut $state_name) -> R) -> R {
                use ::leptos::prelude::Update;
                self.state.try_update(f).expect("signal disposed")
            }
        }

        impl Default for $store_name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::store::Store for $store_name {
            type State = $state_name;

            fn state(&self) -> ::leptos::prelude::ReadSignal<Self::State> {
                self.state.read_only()
            }
        }
    };

    // Default value helpers
    (@default $ty:ty, $default:expr) => { $default };
    (@default $ty:ty) => { <$ty as Default>::default() };
}

// ============================================================================
// Helper macros (internal use)
// ============================================================================

/// Macro to define a getter function inside an impl block.
///
/// This is a helper macro for manual store definitions when not using
/// the `store!` macro.
///
/// # Note
///
/// This macro must be used inside an impl block for a type that has
/// a `state()` method returning a signal.
#[macro_export]
macro_rules! define_getter {
    ($name:ident, $output:ty, $closure:expr) => {
        pub fn $name(&self) -> $output {
            let getter: fn(&Self::State) -> $output = $closure;
            self.state().with(getter)
        }
    };
}

/// Macro to define a **private** mutator function inside an impl block.
///
/// This is a helper macro for manual store definitions when not using
/// the `store!` macro. Mutators are generated as **private** functions
/// to enforce the Enterprise Mode pattern where only actions are public.
///
/// # Note
///
/// This macro must be used inside an impl block for a type that has
/// a `state` field of type `RwSignal<State>`.
///
/// # Example
///
/// ```rust,ignore
/// impl MyStore {
///     // Private mutator - only callable from within this impl
///     define_mutator!(set_count, value: i32, |state, v| state.count = v);
///     
///     // Public action - calls the private mutator
///     pub fn increment(&self) {
///         self.set_count(self.state.with(|s| s.count + 1));
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_mutator {
    ($name:ident, $closure:expr) => {
        #[allow(dead_code)]
        fn $name(&self) {
            self.state.update($closure);
        }
    };

    ($name:ident, $param:ident : $param_ty:ty, $closure:expr) => {
        #[allow(dead_code)]
        fn $name(&self, $param: $param_ty) {
            self.state.update(|state| {
                let mutator: fn(&mut Self::State, $param_ty) = $closure;
                mutator(state, $param);
            });
        }
    };
}

// ============================================================================
// selector! macro
// ============================================================================

/// Create multiple memoized selectors from a store in one declaration.
///
/// Each selector extracts a specific slice of state and only re-computes
/// when that slice changes.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::prelude::*;
///
/// selector! {
///     store: &my_store,
///     user_name: |s: &MyState| -> String { s.user.name.clone() },
///     is_admin: |s: &MyState| -> bool { s.user.role == Role::Admin },
///     item_count: |s: &MyState| -> usize { s.cart.items.len() },
/// }
///
/// // Now use them as Memo<T> values:
/// view! { <p>{user_name}</p> }
/// ```
#[macro_export]
macro_rules! selector {
    (
        store: $store:expr,
        $($name:ident : |$param:ident : &$state_ty:ty| -> $ret_ty:ty $body:block),+ $(,)?
    ) => {
        $(
            let $name: ::leptos::prelude::Memo<$ret_ty> = $crate::selectors::create_selector(
                $store,
                |$param: &$state_ty| -> $ret_ty { $body },
            );
        )+
    };
}

// ============================================================================
// Namespace Macro
// ============================================================================

/// Define a typed namespace that aggregates multiple domain stores.
///
/// Generates a struct with typed accessors for each store, plus
/// `provide_<snake_case>()` and `use_<snake_case>()` context helper functions.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::prelude::*;
///
/// namespace! {
///     pub AppStores {
///         user: UserStore,
///         products: ProductStore,
///         cart: CartStore,
///     }
/// }
///
/// // Create and provide:
/// let stores = AppStores::new(
///     UserStore::new(),
///     ProductStore::new(),
///     CartStore::new(),
/// );
/// provide_app_stores(stores);
///
/// // Consume in components:
/// let stores = use_app_stores();
/// let user = stores.user();
/// ```
#[macro_export]
macro_rules! namespace {
    (
        $ns_vis:vis $ns_name:ident {
            $( $store_field:ident : $store_ty:ty ),+ $(,)?
        }
    ) => {
        #[derive(Clone)]
        $ns_vis struct $ns_name {
            $( $store_field: $store_ty, )+
        }

        impl $ns_name {
            /// Create a new namespace with all stores.
            pub fn new( $( $store_field: $store_ty ),+ ) -> Self {
                Self { $( $store_field ),+ }
            }

            $(
                /// Access the store.
                pub fn $store_field(&self) -> &$store_ty {
                    &self.$store_field
                }
            )+

            /// Get the number of stores in this namespace.
            pub fn store_count(&self) -> usize {
                $crate::namespace!(@count $( $store_field )+)
            }
        }

        ::paste::paste! {
            /// Provide this namespace into the Leptos context tree.
            $ns_vis fn [<provide_ $ns_name:snake>](stores: $ns_name) {
                ::leptos::prelude::provide_context(stores);
            }

            /// Retrieve this namespace from the Leptos context tree.
            $ns_vis fn [<use_ $ns_name:snake>]() -> $ns_name {
                ::leptos::prelude::use_context::<$ns_name>()
                    .expect(concat!(stringify!($ns_name), " not found in context. Did you call provide_", stringify!([<$ns_name:snake>]), "()?"))
            }
        }
    };

    // Count helper
    (@count $first:ident $( $rest:ident )*) => {
        1 + $crate::namespace!(@count $( $rest )*)
    };
    (@count) => { 0 };
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use leptos::prelude::*;
    use std::collections::HashMap;

    #[test]
    fn test_define_state_basic() {
        define_state! {
            #[derive(Clone, Debug, PartialEq)]
            struct TestState {
                count: i32 = 10,
                name: String,
            }
        }

        let state = TestState::default();
        assert_eq!(state.count, 10);
        assert_eq!(state.name, "");
    }

    #[test]
    fn test_define_state_complex_types() {
        define_state! {
            #[derive(Clone, Debug)]
            struct ComplexState {
                items: Vec<String>,
                cache: HashMap<String, i32>,
                optional: Option<bool> = Some(true),
            }
        }

        let state = ComplexState::default();
        assert!(state.items.is_empty());
        assert!(state.cache.is_empty());
        assert_eq!(state.optional, Some(true));
    }

    #[test]
    fn test_define_action_basic() {
        define_action! {
            #[derive(Debug, Clone, PartialEq)]
            TestAction {
                user_id: String,
                amount: i32,
            }
        }

        let action = TestAction::new("user123".to_string(), 100);
        assert_eq!(action.user_id, "user123");
        assert_eq!(action.amount, 100);
    }

    #[test]
    fn test_define_action_single_field() {
        define_action! {
            #[derive(Debug)]
            SimpleAction {
                value: i32,
            }
        }

        let action = SimpleAction::new(42);
        assert_eq!(action.value, 42);
    }

    #[test]
    fn test_define_async_action() {
        use std::fmt;

        #[derive(Debug)]
        #[allow(dead_code)]
        struct TestError(String);
        impl fmt::Display for TestError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
        impl std::error::Error for TestError {}

        define_async_action! {
            #[derive(Debug, Clone)]
            FetchAction {
                id: String,
                limit: u32,
            } -> Result<Vec<String>, TestError>
        }

        let action = FetchAction::new("test_id".to_string(), 10);
        assert_eq!(action.id, "test_id");
        assert_eq!(action.limit, 10);

        // The result_type_description contains the stringified types
        let desc = FetchAction::result_type_description();
        assert!(desc.contains("Result"));
        assert!(desc.contains("Vec"));
        assert!(desc.contains("String"));
        assert!(desc.contains("TestError"));

        // Check helper methods
        assert!(FetchAction::output_type_name().contains("Vec"));
        assert_eq!(FetchAction::error_type_name(), "TestError");
    }

    #[test]
    fn test_define_async_action_simple() {
        define_async_action! {
            #[derive(Debug)]
            ComputeAction {
                input: i32,
            } -> String
        }

        let action = ComputeAction::new(42);
        assert_eq!(action.input, 42);
    }

    #[test]
    fn test_store_macro_state_generation() {
        store! {
            pub TestStore {
                state TestStoreState {
                    value: i32 = 42,
                    label: String = "test".to_string(),
                }
            }
        }

        let state = TestStoreState::default();
        assert_eq!(state.value, 42);
        assert_eq!(state.label, "test");

        let store = TestStore::new();
        assert_eq!(store.state.get().value, 42);
    }

    #[test]
    fn test_store_macro_with_state() {
        store! {
            pub CustomStore {
                state CustomState {
                    count: i32 = 0,
                }
            }
        }

        let custom_state = CustomState { count: 100 };
        let store = CustomStore::with_state(custom_state);
        assert_eq!(store.state.get().count, 100);
    }

    #[test]
    fn test_namespace_macro() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            let owner = Owner::new();
            owner.with(|| {
                use crate::store::Store;

                // Define two stores
                store! {
                    pub NsStoreA {
                        state NsStateA {
                            value_a: i32 = 10,
                        }
                        getters {}
                        mutators {}
                        actions {}
                    }
                }

                store! {
                    pub NsStoreB {
                        state NsStateB {
                            value_b: String = String::new(),
                        }
                        getters {}
                        mutators {}
                        actions {}
                    }
                }

                namespace! {
                    pub TestNamespace {
                        alpha: NsStoreA,
                        beta: NsStoreB,
                    }
                }

                let ns = TestNamespace::new(NsStoreA::new(), NsStoreB::new());
                // Test accessor methods
                let _alpha: &NsStoreA = ns.alpha();
                let _beta: &NsStoreB = ns.beta();
                // Test store count
                assert_eq!(ns.store_count(), 2);
                // Test accessor reads state correctly via Store trait
                use leptos::prelude::With;
                assert_eq!(ns.alpha().state().with(|s| s.value_a), 10);
            });
        });
    }
}
