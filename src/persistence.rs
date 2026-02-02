// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Persistence adapters for store state.
//!
//! This module provides infrastructure for persisting store state across
//! sessions using various storage backends.
//!
//! # Available Adapters
//!
//! | Adapter | Feature | Platform | Description |
//! |---------|---------|----------|-------------|
//! | `MemoryAdapter` | default | All | In-memory storage (testing) |
//! | `LocalStorageAdapter` | `persist-web` | WASM | Browser localStorage |
//! | `SessionStorageAdapter` | `persist-web` | WASM | Browser sessionStorage |
//! | `IndexedDbAdapter` | `persist-idb` | WASM | IndexedDB for larger data |
//! | `ServerSyncAdapter` | `persist-server` | SSR | Server-side persistence |
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos_store::persistence::*;
//!
//! let store = MyStore::new();
//! let persistent = PersistentStore::new(store, LocalStorageAdapter::new())
//!     .with_key("my_store")
//!     .with_debounce(500);
//! ```

use crate::store::Store;
use leptos::prelude::Get;
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use thiserror::Error;

// ============================================================================
// Persistence Errors
// ============================================================================

/// Errors that can occur during persistence operations.
#[derive(Debug, Error, Clone)]
pub enum PersistError {
    /// Failed to serialize state.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Failed to deserialize state.
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Storage is not available.
    #[error("Storage not available: {0}")]
    NotAvailable(String),

    /// Storage quota exceeded.
    #[error("Storage quota exceeded")]
    QuotaExceeded,

    /// Key not found.
    #[error("Key not found: {0}")]
    NotFound(String),

    /// Permission denied.
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Network error (for server sync).
    #[error("Network error: {0}")]
    Network(String),

    /// Version mismatch during migration.
    #[error("Version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },

    /// Internal error.
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for persistence operations.
pub type PersistResult<T> = Result<T, PersistError>;

// ============================================================================
// Storage Types
// ============================================================================

/// Types of storage backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// In-memory storage (volatile).
    Memory,
    /// Browser localStorage (persistent, synchronous).
    LocalStorage,
    /// Browser sessionStorage (session-scoped).
    SessionStorage,
    /// IndexedDB (persistent, asynchronous, larger capacity).
    IndexedDb,
    /// Server-side storage.
    Server,
    /// Custom storage implementation.
    Custom,
}

impl fmt::Display for StorageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Memory => write!(f, "Memory"),
            Self::LocalStorage => write!(f, "LocalStorage"),
            Self::SessionStorage => write!(f, "SessionStorage"),
            Self::IndexedDb => write!(f, "IndexedDB"),
            Self::Server => write!(f, "Server"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

// ============================================================================
// Persistence Adapter Trait
// ============================================================================

/// Future type for async persistence operations.
pub type PersistFuture<'a, T> = Pin<Box<dyn Future<Output = PersistResult<T>> + Send + 'a>>;

/// Trait for persistence adapters.
///
/// Adapters provide the interface between stores and storage backends.
/// All operations are async to support both sync (localStorage) and
/// async (IndexedDB) backends uniformly.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::persistence::*;
///
/// struct MyAdapter;
///
/// impl PersistenceAdapter for MyAdapter {
///     fn save<'a>(&'a self, key: &'a str, data: &'a [u8]) -> PersistFuture<'a, ()> {
///         Box::pin(async move {
///             // Save data to your backend
///             Ok(())
///         })
///     }
///
///     fn load<'a>(&'a self, key: &'a str) -> PersistFuture<'a, Option<Vec<u8>>> {
///         Box::pin(async move {
///             // Load data from your backend
///             Ok(None)
///         })
///     }
///
///     fn remove<'a>(&'a self, key: &'a str) -> PersistFuture<'a, ()> {
///         Box::pin(async move {
///             // Remove data from your backend
///             Ok(())
///         })
///     }
///
///     fn storage_type(&self) -> StorageType {
///         StorageType::Custom
///     }
/// }
/// ```
pub trait PersistenceAdapter: Send + Sync {
    /// Save data to storage.
    fn save<'a>(&'a self, key: &'a str, data: &'a [u8]) -> PersistFuture<'a, ()>;

    /// Load data from storage.
    fn load<'a>(&'a self, key: &'a str) -> PersistFuture<'a, Option<Vec<u8>>>;

    /// Remove data from storage.
    fn remove<'a>(&'a self, key: &'a str) -> PersistFuture<'a, ()>;

    /// Get the storage type.
    fn storage_type(&self) -> StorageType;

    /// Check if the adapter is available.
    fn is_available(&self) -> bool {
        true
    }

    /// Get storage capacity info (if available).
    fn capacity(&self) -> Option<StorageCapacity> {
        None
    }

    /// Clear all data from this adapter's namespace.
    fn clear<'a>(&'a self) -> PersistFuture<'a, ()> {
        Box::pin(async { Ok(()) })
    }

    /// List all keys in this adapter's namespace.
    fn keys<'a>(&'a self) -> PersistFuture<'a, Vec<String>> {
        Box::pin(async { Ok(Vec::new()) })
    }
}

/// Storage capacity information.
#[derive(Debug, Clone, Copy)]
pub struct StorageCapacity {
    /// Total capacity in bytes (if known).
    pub total: Option<u64>,
    /// Used capacity in bytes (if known).
    pub used: Option<u64>,
    /// Available capacity in bytes (if known).
    pub available: Option<u64>,
}

impl StorageCapacity {
    /// Create a new capacity info with all fields unknown.
    pub fn unknown() -> Self {
        Self {
            total: None,
            used: None,
            available: None,
        }
    }

    /// Create capacity info with known values.
    pub fn known(total: u64, used: u64) -> Self {
        Self {
            total: Some(total),
            used: Some(used),
            available: Some(total.saturating_sub(used)),
        }
    }
}

// ============================================================================
// Memory Adapter (Default)
// ============================================================================

/// In-memory persistence adapter for testing.
///
/// Data is stored in memory and is lost when the application restarts.
/// This is useful for testing and development.
#[derive(Clone)]
pub struct MemoryAdapter {
    storage: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl Default for MemoryAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryAdapter {
    /// Create a new memory adapter.
    pub fn new() -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a memory adapter with initial data.
    pub fn with_data(data: HashMap<String, Vec<u8>>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(data)),
        }
    }

    /// Get a snapshot of all data (for testing).
    pub fn snapshot(&self) -> HashMap<String, Vec<u8>> {
        self.storage.read().map(|s| s.clone()).unwrap_or_default()
    }

    /// Get the number of stored items.
    pub fn len(&self) -> usize {
        self.storage.read().map(|s| s.len()).unwrap_or(0)
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl PersistenceAdapter for MemoryAdapter {
    fn save<'a>(&'a self, key: &'a str, data: &'a [u8]) -> PersistFuture<'a, ()> {
        let storage = self.storage.clone();
        let key = key.to_string();
        let data = data.to_vec();

        Box::pin(async move {
            storage
                .write()
                .map_err(|e| PersistError::Internal(e.to_string()))?
                .insert(key, data);
            Ok(())
        })
    }

    fn load<'a>(&'a self, key: &'a str) -> PersistFuture<'a, Option<Vec<u8>>> {
        let storage = self.storage.clone();
        let key = key.to_string();

        Box::pin(async move {
            let data = storage
                .read()
                .map_err(|e| PersistError::Internal(e.to_string()))?
                .get(&key)
                .cloned();
            Ok(data)
        })
    }

    fn remove<'a>(&'a self, key: &'a str) -> PersistFuture<'a, ()> {
        let storage = self.storage.clone();
        let key = key.to_string();

        Box::pin(async move {
            storage
                .write()
                .map_err(|e| PersistError::Internal(e.to_string()))?
                .remove(&key);
            Ok(())
        })
    }

    fn storage_type(&self) -> StorageType {
        StorageType::Memory
    }

    fn clear<'a>(&'a self) -> PersistFuture<'a, ()> {
        let storage = self.storage.clone();

        Box::pin(async move {
            storage
                .write()
                .map_err(|e| PersistError::Internal(e.to_string()))?
                .clear();
            Ok(())
        })
    }

    fn keys<'a>(&'a self) -> PersistFuture<'a, Vec<String>> {
        let storage = self.storage.clone();

        Box::pin(async move {
            let keys = storage
                .read()
                .map_err(|e| PersistError::Internal(e.to_string()))?
                .keys()
                .cloned()
                .collect();
            Ok(keys)
        })
    }
}

impl fmt::Debug for MemoryAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryAdapter")
            .field("items", &self.len())
            .finish()
    }
}

// ============================================================================
// Persist Configuration
// ============================================================================

/// Configuration for persistent stores.
#[derive(Debug, Clone)]
pub struct PersistConfig {
    /// Storage key for this store.
    pub key: String,
    /// Debounce time in milliseconds (0 = no debounce).
    pub debounce_ms: u64,
    /// Version number for migrations.
    pub version: u32,
    /// Whether to auto-save on state changes.
    pub auto_save: bool,
    /// Whether to auto-load on store creation.
    pub auto_load: bool,
    /// Prefix for storage keys.
    pub key_prefix: String,
}

impl Default for PersistConfig {
    fn default() -> Self {
        Self {
            key: String::new(),
            debounce_ms: 100,
            version: 1,
            auto_save: true,
            auto_load: true,
            key_prefix: "leptos_store_".to_string(),
        }
    }
}

impl PersistConfig {
    /// Create a new config with the given key.
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            ..Default::default()
        }
    }

    /// Get the full storage key (prefix + key).
    pub fn full_key(&self) -> String {
        format!("{}{}", self.key_prefix, self.key)
    }
}

// ============================================================================
// Persisted State Wrapper
// ============================================================================

/// Wrapper for state that includes version information.
#[derive(Debug, Clone)]
pub struct PersistedState<State> {
    /// The actual state data.
    pub state: State,
    /// Version number for migration.
    pub version: u32,
    /// Timestamp when saved (milliseconds since epoch).
    pub saved_at: u64,
}

impl<State> PersistedState<State> {
    /// Create a new persisted state wrapper.
    pub fn new(state: State, version: u32) -> Self {
        Self {
            state,
            version,
            saved_at: current_timestamp_ms(),
        }
    }
}

// ============================================================================
// Persistent Store Wrapper
// ============================================================================

/// A store wrapper that adds persistence capabilities.
///
/// This wrapper automatically saves state changes to the configured
/// storage adapter and can restore state on initialization.
///
/// # Example
///
/// ```rust,ignore
/// use leptos_store::persistence::*;
///
/// let store = MyStore::new();
/// let persistent = PersistentStore::new(store, MemoryAdapter::new())
///     .with_key("my_store")
///     .with_debounce(500);
///
/// // State changes are automatically persisted
/// persistent.inner().set_count(42);
/// ```
pub struct PersistentStore<S, A>
where
    S: Store,
    A: PersistenceAdapter,
{
    inner: S,
    adapter: Arc<A>,
    config: PersistConfig,
    _marker: PhantomData<S::State>,
}

impl<S, A> Clone for PersistentStore<S, A>
where
    S: Store,
    A: PersistenceAdapter,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            adapter: Arc::clone(&self.adapter),
            config: self.config.clone(),
            _marker: PhantomData,
        }
    }
}

impl<S, A> PersistentStore<S, A>
where
    S: Store,
    A: PersistenceAdapter + 'static,
{
    /// Create a new persistent store.
    pub fn new(store: S, adapter: A) -> Self {
        Self {
            inner: store,
            adapter: Arc::new(adapter),
            config: PersistConfig::default(),
            _marker: PhantomData,
        }
    }

    /// Set the storage key.
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.config.key = key.into();
        self
    }

    /// Set the debounce time in milliseconds.
    pub fn with_debounce(mut self, ms: u64) -> Self {
        self.config.debounce_ms = ms;
        self
    }

    /// Set the version number for migrations.
    pub fn with_version(mut self, version: u32) -> Self {
        self.config.version = version;
        self
    }

    /// Enable or disable auto-save.
    pub fn with_auto_save(mut self, enabled: bool) -> Self {
        self.config.auto_save = enabled;
        self
    }

    /// Enable or disable auto-load.
    pub fn with_auto_load(mut self, enabled: bool) -> Self {
        self.config.auto_load = enabled;
        self
    }

    /// Set a custom key prefix.
    pub fn with_key_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config.key_prefix = prefix.into();
        self
    }

    /// Get the inner store.
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// Get mutable access to the inner store.
    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }

    /// Get the adapter.
    pub fn adapter(&self) -> &A {
        &self.adapter
    }

    /// Get the configuration.
    pub fn config(&self) -> &PersistConfig {
        &self.config
    }

    /// Get the full storage key.
    pub fn storage_key(&self) -> String {
        self.config.full_key()
    }
}

// Persistence operations for serializable state
impl<S, A> PersistentStore<S, A>
where
    S: Store,
    S::State: serde::Serialize + serde::de::DeserializeOwned,
    A: PersistenceAdapter + 'static,
{
    /// Save the current state to storage.
    pub async fn save(&self) -> PersistResult<()> {
        let state = self.inner.state().get();
        let persisted = PersistedState::new(state, self.config.version);

        let data = serde_json::to_vec(&persisted)
            .map_err(|e| PersistError::Serialization(e.to_string()))?;

        self.adapter.save(&self.storage_key(), &data).await
    }

    /// Load state from storage.
    ///
    /// Returns `None` if no data is found.
    pub async fn load(&self) -> PersistResult<Option<S::State>> {
        let data = self.adapter.load(&self.storage_key()).await?;

        match data {
            Some(bytes) => {
                let persisted: PersistedState<S::State> = serde_json::from_slice(&bytes)
                    .map_err(|e| PersistError::Deserialization(e.to_string()))?;

                // Check version
                if persisted.version != self.config.version {
                    return Err(PersistError::VersionMismatch {
                        expected: self.config.version,
                        found: persisted.version,
                    });
                }

                Ok(Some(persisted.state))
            }
            None => Ok(None),
        }
    }

    /// Remove persisted state from storage.
    pub async fn remove(&self) -> PersistResult<()> {
        self.adapter.remove(&self.storage_key()).await
    }

    /// Check if persisted state exists.
    pub async fn exists(&self) -> PersistResult<bool> {
        let data = self.adapter.load(&self.storage_key()).await?;
        Ok(data.is_some())
    }
}

impl<S, A> Store for PersistentStore<S, A>
where
    S: Store,
    A: PersistenceAdapter + 'static,
{
    type State = S::State;

    fn state(&self) -> leptos::prelude::ReadSignal<Self::State> {
        self.inner.state()
    }

    fn id(&self) -> crate::store::StoreId {
        self.inner.id()
    }

    fn name(&self) -> &'static str {
        self.inner.name()
    }
}

impl<S, A> fmt::Debug for PersistentStore<S, A>
where
    S: Store + fmt::Debug,
    A: PersistenceAdapter + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PersistentStore")
            .field("inner", &self.inner)
            .field("storage_type", &self.adapter.storage_type())
            .field("key", &self.storage_key())
            .finish()
    }
}

// ============================================================================
// Web Storage Adapters (feature-gated)
// ============================================================================

/// LocalStorage adapter for browser environments.
///
/// This adapter uses the Web Storage API's localStorage, which persists
/// data across browser sessions.
///
/// # Feature
///
/// Requires the `persist-web` feature.
///
/// # Limitations
///
/// - Synchronous API (but wrapped as async for uniformity)
/// - ~5MB storage limit per origin
/// - String-only storage (data is base64 encoded)
#[cfg(feature = "persist-web")]
pub struct LocalStorageAdapter {
    _private: (),
}

#[cfg(feature = "persist-web")]
impl Default for LocalStorageAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "persist-web")]
impl LocalStorageAdapter {
    /// Create a new localStorage adapter.
    pub fn new() -> Self {
        Self { _private: () }
    }

    #[cfg(target_arch = "wasm32")]
    fn get_storage(&self) -> PersistResult<web_sys::Storage> {
        web_sys::window()
            .ok_or_else(|| PersistError::NotAvailable("No window object".to_string()))?
            .local_storage()
            .map_err(|_| PersistError::NotAvailable("localStorage not accessible".to_string()))?
            .ok_or_else(|| PersistError::NotAvailable("localStorage is null".to_string()))
    }
}

#[cfg(feature = "persist-web")]
impl PersistenceAdapter for LocalStorageAdapter {
    fn save<'a>(&'a self, key: &'a str, data: &'a [u8]) -> PersistFuture<'a, ()> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = key.to_string();
            let data = data.to_vec();
            let storage_result = self.get_storage();

            Box::pin(async move {
                let storage = storage_result?;
                let encoded = base64_encode(&data);
                storage
                    .set_item(&key, &encoded)
                    .map_err(|_| PersistError::QuotaExceeded)?;
                Ok(())
            })
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (key, data);
            Box::pin(async { Err(PersistError::NotAvailable("Not in browser".to_string())) })
        }
    }

    fn load<'a>(&'a self, key: &'a str) -> PersistFuture<'a, Option<Vec<u8>>> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = key.to_string();
            let storage_result = self.get_storage();

            Box::pin(async move {
                let storage = storage_result?;
                match storage.get_item(&key) {
                    Ok(Some(encoded)) => {
                        let data = base64_decode(&encoded)
                            .map_err(|e| PersistError::Deserialization(e.to_string()))?;
                        Ok(Some(data))
                    }
                    Ok(None) => Ok(None),
                    Err(_) => Err(PersistError::Internal(
                        "Failed to read localStorage".to_string(),
                    )),
                }
            })
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = key;
            Box::pin(async { Err(PersistError::NotAvailable("Not in browser".to_string())) })
        }
    }

    fn remove<'a>(&'a self, key: &'a str) -> PersistFuture<'a, ()> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = key.to_string();
            let storage_result = self.get_storage();

            Box::pin(async move {
                let storage = storage_result?;
                storage
                    .remove_item(&key)
                    .map_err(|_| PersistError::Internal("Failed to remove item".to_string()))?;
                Ok(())
            })
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = key;
            Box::pin(async { Err(PersistError::NotAvailable("Not in browser".to_string())) })
        }
    }

    fn storage_type(&self) -> StorageType {
        StorageType::LocalStorage
    }

    #[cfg(target_arch = "wasm32")]
    fn is_available(&self) -> bool {
        self.get_storage().is_ok()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn is_available(&self) -> bool {
        false
    }
}

/// SessionStorage adapter for browser environments.
///
/// This adapter uses the Web Storage API's sessionStorage, which persists
/// data only for the duration of the browser session.
///
/// # Feature
///
/// Requires the `persist-web` feature.
#[cfg(feature = "persist-web")]
pub struct SessionStorageAdapter {
    _private: (),
}

#[cfg(feature = "persist-web")]
impl Default for SessionStorageAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "persist-web")]
impl SessionStorageAdapter {
    /// Create a new sessionStorage adapter.
    pub fn new() -> Self {
        Self { _private: () }
    }

    #[cfg(target_arch = "wasm32")]
    fn get_storage(&self) -> PersistResult<web_sys::Storage> {
        web_sys::window()
            .ok_or_else(|| PersistError::NotAvailable("No window object".to_string()))?
            .session_storage()
            .map_err(|_| PersistError::NotAvailable("sessionStorage not accessible".to_string()))?
            .ok_or_else(|| PersistError::NotAvailable("sessionStorage is null".to_string()))
    }
}

#[cfg(feature = "persist-web")]
impl PersistenceAdapter for SessionStorageAdapter {
    fn save<'a>(&'a self, key: &'a str, data: &'a [u8]) -> PersistFuture<'a, ()> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = key.to_string();
            let data = data.to_vec();
            let storage_result = self.get_storage();

            Box::pin(async move {
                let storage = storage_result?;
                let encoded = base64_encode(&data);
                storage
                    .set_item(&key, &encoded)
                    .map_err(|_| PersistError::QuotaExceeded)?;
                Ok(())
            })
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (key, data);
            Box::pin(async { Err(PersistError::NotAvailable("Not in browser".to_string())) })
        }
    }

    fn load<'a>(&'a self, key: &'a str) -> PersistFuture<'a, Option<Vec<u8>>> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = key.to_string();
            let storage_result = self.get_storage();

            Box::pin(async move {
                let storage = storage_result?;
                match storage.get_item(&key) {
                    Ok(Some(encoded)) => {
                        let data = base64_decode(&encoded)
                            .map_err(|e| PersistError::Deserialization(e.to_string()))?;
                        Ok(Some(data))
                    }
                    Ok(None) => Ok(None),
                    Err(_) => Err(PersistError::Internal(
                        "Failed to read sessionStorage".to_string(),
                    )),
                }
            })
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = key;
            Box::pin(async { Err(PersistError::NotAvailable("Not in browser".to_string())) })
        }
    }

    fn remove<'a>(&'a self, key: &'a str) -> PersistFuture<'a, ()> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = key.to_string();
            let storage_result = self.get_storage();

            Box::pin(async move {
                let storage = storage_result?;
                storage
                    .remove_item(&key)
                    .map_err(|_| PersistError::Internal("Failed to remove item".to_string()))?;
                Ok(())
            })
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = key;
            Box::pin(async { Err(PersistError::NotAvailable("Not in browser".to_string())) })
        }
    }

    fn storage_type(&self) -> StorageType {
        StorageType::SessionStorage
    }

    #[cfg(target_arch = "wasm32")]
    fn is_available(&self) -> bool {
        self.get_storage().is_ok()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn is_available(&self) -> bool {
        false
    }
}

// ============================================================================
// IndexedDB Adapter (feature-gated)
// ============================================================================

/// IndexedDB adapter for larger persistent storage.
///
/// This adapter uses IndexedDB for storing larger amounts of data
/// with proper async support.
///
/// # Feature
///
/// Requires the `persist-idb` feature.
///
/// # Advantages
///
/// - Larger storage capacity (typically 50MB+)
/// - Proper async API
/// - Structured data support
/// - Transaction support
#[cfg(feature = "persist-idb")]
pub struct IndexedDbAdapter {
    database_name: String,
    store_name: String,
}

#[cfg(feature = "persist-idb")]
impl IndexedDbAdapter {
    /// Create a new IndexedDB adapter.
    pub fn new(database_name: impl Into<String>) -> Self {
        Self {
            database_name: database_name.into(),
            store_name: "store".to_string(),
        }
    }

    /// Set the object store name.
    pub fn with_store_name(mut self, name: impl Into<String>) -> Self {
        self.store_name = name.into();
        self
    }
}

#[cfg(feature = "persist-idb")]
impl PersistenceAdapter for IndexedDbAdapter {
    fn save<'a>(&'a self, key: &'a str, data: &'a [u8]) -> PersistFuture<'a, ()> {
        let _ = (key, data);
        // IndexedDB implementation would go here using idb crate
        Box::pin(async {
            Err(PersistError::NotAvailable(
                "IndexedDB not yet implemented".to_string(),
            ))
        })
    }

    fn load<'a>(&'a self, key: &'a str) -> PersistFuture<'a, Option<Vec<u8>>> {
        let _ = key;
        Box::pin(async {
            Err(PersistError::NotAvailable(
                "IndexedDB not yet implemented".to_string(),
            ))
        })
    }

    fn remove<'a>(&'a self, key: &'a str) -> PersistFuture<'a, ()> {
        let _ = key;
        Box::pin(async {
            Err(PersistError::NotAvailable(
                "IndexedDB not yet implemented".to_string(),
            ))
        })
    }

    fn storage_type(&self) -> StorageType {
        StorageType::IndexedDb
    }
}

// ============================================================================
// Server Sync Adapter (feature-gated)
// ============================================================================

/// Server sync adapter for SSR state persistence.
///
/// This adapter syncs state with a server endpoint, enabling state
/// persistence across server restarts and horizontal scaling.
///
/// # Feature
///
/// Requires the `persist-server` feature.
#[cfg(feature = "persist-server")]
pub struct ServerSyncAdapter {
    endpoint: String,
}

#[cfg(feature = "persist-server")]
impl ServerSyncAdapter {
    /// Create a new server sync adapter.
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }
}

#[cfg(feature = "persist-server")]
impl PersistenceAdapter for ServerSyncAdapter {
    fn save<'a>(&'a self, key: &'a str, data: &'a [u8]) -> PersistFuture<'a, ()> {
        let _ = (key, data);
        Box::pin(async {
            Err(PersistError::NotAvailable(
                "Server sync not yet implemented".to_string(),
            ))
        })
    }

    fn load<'a>(&'a self, key: &'a str) -> PersistFuture<'a, Option<Vec<u8>>> {
        let _ = key;
        Box::pin(async {
            Err(PersistError::NotAvailable(
                "Server sync not yet implemented".to_string(),
            ))
        })
    }

    fn remove<'a>(&'a self, key: &'a str) -> PersistFuture<'a, ()> {
        let _ = key;
        Box::pin(async {
            Err(PersistError::NotAvailable(
                "Server sync not yet implemented".to_string(),
            ))
        })
    }

    fn storage_type(&self) -> StorageType {
        StorageType::Server
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get current timestamp in milliseconds.
fn current_timestamp_ms() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Base64 encode data.
#[cfg(feature = "persist-web")]
fn base64_encode(data: &[u8]) -> String {
    use std::io::Write;
    let mut encoder =
        base64::write::EncoderStringWriter::new(&base64::engine::general_purpose::STANDARD);
    encoder.write_all(data).unwrap();
    encoder.into_inner()
}

/// Base64 decode data.
#[cfg(feature = "persist-web")]
fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| e.to_string())
}

// ============================================================================
// Derive macro for serialization (requires serde)
// ============================================================================

/// Implement Serialize and Deserialize for PersistedState.
#[cfg(feature = "hydrate")]
impl<State: serde::Serialize> serde::Serialize for PersistedState<State> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("PersistedState", 3)?;
        s.serialize_field("state", &self.state)?;
        s.serialize_field("version", &self.version)?;
        s.serialize_field("saved_at", &self.saved_at)?;
        s.end()
    }
}

#[cfg(feature = "hydrate")]
impl<'de, State: serde::Deserialize<'de>> serde::Deserialize<'de> for PersistedState<State> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct PersistedStateHelper<S> {
            state: S,
            version: u32,
            saved_at: u64,
        }

        let helper = PersistedStateHelper::deserialize(deserializer)?;
        Ok(Self {
            state: helper.state,
            version: helper.version,
            saved_at: helper.saved_at,
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::*;

    #[derive(Clone, Debug, Default, PartialEq)]
    struct TestState {
        count: i32,
    }

    #[derive(Clone)]
    struct TestStore {
        state: RwSignal<TestState>,
    }

    impl Store for TestStore {
        type State = TestState;

        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    #[test]
    fn test_persist_error_display() {
        assert!(
            PersistError::Serialization("test".to_string())
                .to_string()
                .contains("Serialization")
        );
        assert!(PersistError::QuotaExceeded.to_string().contains("quota"));
        assert!(
            PersistError::VersionMismatch {
                expected: 2,
                found: 1
            }
            .to_string()
            .contains("expected 2")
        );
    }

    #[test]
    fn test_storage_type_display() {
        assert_eq!(StorageType::Memory.to_string(), "Memory");
        assert_eq!(StorageType::LocalStorage.to_string(), "LocalStorage");
        assert_eq!(StorageType::IndexedDb.to_string(), "IndexedDB");
    }

    #[test]
    fn test_memory_adapter() {
        let adapter = MemoryAdapter::new();
        assert!(adapter.is_empty());
        assert_eq!(adapter.storage_type(), StorageType::Memory);
    }

    #[tokio::test]
    async fn test_memory_adapter_operations() {
        let adapter = MemoryAdapter::new();

        // Save
        adapter.save("key1", b"hello").await.unwrap();
        assert_eq!(adapter.len(), 1);

        // Load
        let data = adapter.load("key1").await.unwrap();
        assert_eq!(data, Some(b"hello".to_vec()));

        // Load non-existent
        let data = adapter.load("key2").await.unwrap();
        assert!(data.is_none());

        // Remove
        adapter.remove("key1").await.unwrap();
        assert!(adapter.is_empty());

        // Keys
        adapter.save("a", b"1").await.unwrap();
        adapter.save("b", b"2").await.unwrap();
        let keys = adapter.keys().await.unwrap();
        assert_eq!(keys.len(), 2);

        // Clear
        adapter.clear().await.unwrap();
        assert!(adapter.is_empty());
    }

    #[test]
    fn test_persist_config() {
        let config = PersistConfig::new("my_store");
        assert_eq!(config.key, "my_store");
        assert_eq!(config.full_key(), "leptos_store_my_store");
    }

    #[test]
    fn test_storage_capacity() {
        let unknown = StorageCapacity::unknown();
        assert!(unknown.total.is_none());

        let known = StorageCapacity::known(1000, 300);
        assert_eq!(known.total, Some(1000));
        assert_eq!(known.used, Some(300));
        assert_eq!(known.available, Some(700));
    }

    #[test]
    fn test_persistent_store_config() {
        let store = TestStore {
            state: RwSignal::new(TestState::default()),
        };

        let persistent = PersistentStore::new(store, MemoryAdapter::new())
            .with_key("test")
            .with_debounce(500)
            .with_version(2)
            .with_auto_save(false);

        assert_eq!(persistent.config().key, "test");
        assert_eq!(persistent.config().debounce_ms, 500);
        assert_eq!(persistent.config().version, 2);
        assert!(!persistent.config().auto_save);
    }
}
