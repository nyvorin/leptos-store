// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Prelude module - re-exports all commonly used types and traits.
//!
//! Import this module to get started quickly:
//!
//! ```rust
//! use leptos_store::prelude::*;
//!
//! // Now you have access to Store, Getter, Mutator, etc.
//! let state = ActionState::Idle;
//! assert!(state.is_idle());
//! ```
//!
//! ## Feature-gated Exports
//!
//! Additional types are available based on enabled features:
//!
//! - `hydrate`: HydratableStore, hydration functions
//! - `middleware`: Middleware traits, LoggingMiddleware, EventBus
//! - `devtools`: StoreInspector, devtools initialization
//! - `persist-web`: LocalStorageAdapter, SessionStorageAdapter
//! - `templates`: FeatureFlagStore, Feature component
//! - `server-actions`: ServerAction trait, action helpers

// Core store traits and types
pub use crate::store::{
    Getter, Mutator, MutatorContext, ReadonlyStore, Store, StoreBuilder, StoreError, StoreId,
    StoreRegistry,
};

// Context management
pub use crate::context::{StoreProvider, provide_store, use_store};

// Async actions
pub use crate::r#async::{
    Action, ActionError, ActionFuture, ActionResult, ActionState, AsyncAction, AsyncActionBuilder,
};

// Composition (always available)
pub use crate::composition::{
    AnyStore, CompositeStore, DerivedView, MultiStoreSelector, RootStore, RootStoreBuilder,
    StoreDependency, StoreGroup, provide_root_store, use_root_store,
};

// Hydration support (when feature is enabled)
#[cfg(feature = "hydrate")]
#[cfg_attr(docsrs, doc(cfg(feature = "hydrate")))]
pub use crate::hydration::{
    HYDRATION_SCRIPT_PREFIX, HydratableStore, HydrationBuilder, StoreHydrationError,
    has_hydration_data, hydrate_store, hydration_script_html, hydration_script_id,
    serialize_store_state,
};

#[cfg(feature = "hydrate")]
#[cfg_attr(docsrs, doc(cfg(feature = "hydrate")))]
pub use crate::context::{
    HydratableStoreContextExt, provide_hydrated_store, try_use_hydrated_store, use_hydrated_store,
};

// Middleware support (when feature is enabled)
#[cfg(feature = "middleware")]
#[cfg_attr(docsrs, doc(cfg(feature = "middleware")))]
pub use crate::middleware::{
    ActionContext, ActionResult as MiddlewareActionResult, ContextMetadata, EventBus,
    EventSubscriber, LogLevel, LoggingConfig, LoggingMiddleware, Middleware, MiddlewareChain,
    MiddlewareContext, MiddlewareError, MiddlewareResult, MiddlewareStore, MutationResult,
    StoreEvent, TimingMiddleware, ValidationMiddleware,
};

#[cfg(feature = "tracing")]
#[cfg_attr(docsrs, doc(cfg(feature = "tracing")))]
pub use crate::middleware::TracingMiddleware;

// Devtools support (when feature is enabled)
#[cfg(feature = "devtools")]
#[cfg_attr(docsrs, doc(cfg(feature = "devtools")))]
pub use crate::devtools::{
    DevtoolsConfig, DevtoolsEvent, DevtoolsEventSubscriber, StoreInfo, StoreInspector,
    TimeTravelDebugger, init_devtools, register_store, unregister_store,
};

// Persistence support (when feature is enabled)
#[cfg(any(
    feature = "persist-web",
    feature = "persist-idb",
    feature = "persist-server"
))]
#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "persist-web",
        feature = "persist-idb",
        feature = "persist-server"
    )))
)]
pub use crate::persistence::{
    MemoryAdapter, PersistConfig, PersistError, PersistResult, PersistedState, PersistenceAdapter,
    PersistentStore, StorageCapacity, StorageType,
};

#[cfg(feature = "persist-web")]
#[cfg_attr(docsrs, doc(cfg(feature = "persist-web")))]
pub use crate::persistence::{LocalStorageAdapter, SessionStorageAdapter};

#[cfg(feature = "persist-idb")]
#[cfg_attr(docsrs, doc(cfg(feature = "persist-idb")))]
pub use crate::persistence::IndexedDbAdapter;

#[cfg(feature = "persist-server")]
#[cfg_attr(docsrs, doc(cfg(feature = "persist-server")))]
pub use crate::persistence::ServerSyncAdapter;

// Server actions support (when feature is enabled)
#[cfg(feature = "server-actions")]
#[cfg_attr(docsrs, doc(cfg(feature = "server-actions")))]
pub use crate::server::{
    ActionHistory, ActionHistoryEntry, OptimisticActionHandle, OptimisticConfig, ServerAction,
    ServerActionBuilder, ServerActionError, ServerActionHandle, ServerActionResult,
    create_server_action, execute_server_action, use_optimistic_action, use_server_action,
};

// Templates (when feature is enabled)
#[cfg(feature = "templates")]
#[cfg_attr(docsrs, doc(cfg(feature = "templates")))]
pub use crate::templates::{
    Feature, FeatureFlag, FeatureFlagState, FeatureFlagStore, FeatureVariant, FlagError,
    UserContext, provide_feature_flags, use_feature, use_feature_flags,
};

// Re-export commonly used Leptos types for convenience
pub use leptos::prelude::{RwSignal, signal};

// Re-export serde when hydrate feature is enabled (for user convenience)
#[cfg(feature = "hydrate")]
pub use serde::{Deserialize, Serialize};
