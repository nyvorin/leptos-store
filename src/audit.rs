// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Audit trail system for tracking state mutation history.
//!
//! This module provides enterprise-grade audit logging for store mutations,
//! capturing before/after state snapshots, field-level diffs, user context,
//! and correlation IDs for distributed tracing.
//!
//! # Overview
//!
//! The audit trail is designed as backend/debugging infrastructure that operates
//! outside the reactive system. It uses `Arc<RwLock<Vec<...>>>` for thread-safe
//! storage rather than reactive signals.
//!
//! # Key Types
//!
//! - [`AuditTrail`] - The main audit log that records state mutations
//! - [`AuditEntry`] - A single audit record with before/after snapshots
//! - [`StateDiff`] - Trait for computing field-level differences between states
//! - [`FieldChange`] - A single field-level change record
//! - [`AuditUserContext`] - User identity and session metadata
//!
//! # Example
//!
//! ```rust
//! use leptos_store::audit::*;
//!
//! #[derive(Clone, Debug)]
//! struct AppState {
//!     count: i32,
//!     name: String,
//! }
//!
//! let trail: AuditTrail<AppState> = AuditTrail::new()
//!     .with_max_entries(500);
//!
//! let before = AppState { count: 0, name: "Alice".into() };
//! let after = AppState { count: 1, name: "Alice".into() };
//!
//! trail.record("increment", &before, &after);
//!
//! assert_eq!(trail.len(), 1);
//! let entries = trail.entries();
//! assert_eq!(entries[0].mutation_name, "increment");
//! ```
//!
//! # With Field-Level Diffs
//!
//! Implement [`StateDiff`] on your state type to get detailed change tracking:
//!
//! ```rust
//! use leptos_store::audit::*;
//!
//! #[derive(Clone, Debug)]
//! struct AppState {
//!     count: i32,
//!     name: String,
//! }
//!
//! impl StateDiff for AppState {
//!     fn diff(&self, other: &Self) -> Vec<FieldChange> {
//!         let mut changes = Vec::new();
//!         if self.count != other.count {
//!             changes.push(FieldChange {
//!                 field_path: "count".into(),
//!                 old_value: format!("{}", self.count),
//!                 new_value: format!("{}", other.count),
//!                 change_type: ChangeType::Modified,
//!             });
//!         }
//!         if self.name != other.name {
//!             changes.push(FieldChange {
//!                 field_path: "name".into(),
//!                 old_value: self.name.clone(),
//!                 new_value: other.name.clone(),
//!                 change_type: ChangeType::Modified,
//!             });
//!         }
//!         changes
//!     }
//! }
//!
//! let trail: AuditTrail<AppState> = AuditTrail::new();
//! let before = AppState { count: 0, name: "Alice".into() };
//! let after = AppState { count: 1, name: "Alice".into() };
//!
//! trail.record_with_diff("increment", &before, &after);
//! let entries = trail.entries();
//! assert_eq!(entries[0].changes.len(), 1);
//! assert_eq!(entries[0].changes[0].field_path, "count");
//! ```

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

// ============================================================================
// Cross-platform Timestamp
// ============================================================================

/// Get current timestamp in milliseconds since the Unix epoch.
///
/// Uses `js_sys::Date::now()` on WASM targets and `std::time::SystemTime`
/// on native targets for cross-platform compatibility.
pub fn current_timestamp_ms() -> u64 {
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
// StateDiff Trait
// ============================================================================

/// Trait for computing field-level differences between two state snapshots.
///
/// Implement this trait on your state type to enable detailed change tracking
/// in [`AuditTrail::record_with_diff`]. Without this trait, audit entries will
/// still capture full before/after snapshots but won't include field-level diffs.
///
/// # Example
///
/// ```rust
/// use leptos_store::audit::{StateDiff, FieldChange, ChangeType};
///
/// #[derive(Clone, Debug)]
/// struct UserState {
///     name: String,
///     age: u32,
/// }
///
/// impl StateDiff for UserState {
///     fn diff(&self, other: &Self) -> Vec<FieldChange> {
///         let mut changes = Vec::new();
///         if self.name != other.name {
///             changes.push(FieldChange {
///                 field_path: "name".into(),
///                 old_value: self.name.clone(),
///                 new_value: other.name.clone(),
///                 change_type: ChangeType::Modified,
///             });
///         }
///         if self.age != other.age {
///             changes.push(FieldChange {
///                 field_path: "age".into(),
///                 old_value: format!("{}", self.age),
///                 new_value: format!("{}", other.age),
///                 change_type: ChangeType::Modified,
///             });
///         }
///         changes
///     }
/// }
/// ```
pub trait StateDiff {
    /// Compute the field-level differences between `self` (old state) and
    /// `other` (new state).
    ///
    /// Returns a list of [`FieldChange`] records describing what changed.
    fn diff(&self, other: &Self) -> Vec<FieldChange>;
}

// ============================================================================
// FieldChange
// ============================================================================

/// A single field-level change record within a state mutation.
///
/// Captures the path to the changed field, its old and new values as strings,
/// and the type of change that occurred.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldChange {
    /// Dot-separated path to the changed field (e.g., `"user.address.city"`).
    pub field_path: String,
    /// String representation of the old value.
    pub old_value: String,
    /// String representation of the new value.
    pub new_value: String,
    /// The type of change that occurred.
    pub change_type: ChangeType,
}

impl fmt::Display for FieldChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.change_type {
            ChangeType::Modified => {
                write!(
                    f,
                    "{}: '{}' -> '{}'",
                    self.field_path, self.old_value, self.new_value
                )
            }
            ChangeType::Added => {
                write!(f, "{}: added '{}'", self.field_path, self.new_value)
            }
            ChangeType::Removed => {
                write!(f, "{}: removed '{}'", self.field_path, self.old_value)
            }
        }
    }
}

// ============================================================================
// ChangeType
// ============================================================================

/// The type of change that occurred to a field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeType {
    /// An existing field was modified.
    Modified,
    /// A new field was added.
    Added,
    /// A field was removed.
    Removed,
}

impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeType::Modified => write!(f, "Modified"),
            ChangeType::Added => write!(f, "Added"),
            ChangeType::Removed => write!(f, "Removed"),
        }
    }
}

// ============================================================================
// AuditUserContext
// ============================================================================

/// User identity and session context attached to audit entries.
///
/// This allows audit records to be correlated with specific users,
/// sessions, or originating IP addresses for compliance and debugging.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuditUserContext {
    /// The user identifier, if known.
    pub user_id: Option<String>,
    /// The session identifier, if known.
    pub session_id: Option<String>,
    /// The originating IP address, if known.
    pub ip_address: Option<String>,
    /// Additional key-value metadata.
    pub metadata: HashMap<String, String>,
}

impl AuditUserContext {
    /// Create a new empty user context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the user ID.
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the session ID.
    pub fn with_session_id(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set the IP address.
    pub fn with_ip_address(mut self, ip_address: impl Into<String>) -> Self {
        self.ip_address = Some(ip_address.into());
        self
    }

    /// Add a metadata key-value pair.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

// ============================================================================
// AuditEntry
// ============================================================================

/// A single audit record capturing a state mutation with full context.
///
/// Each entry includes:
/// - A unique monotonically increasing ID
/// - Timestamps for chronological ordering
/// - The mutation name and optional action name
/// - Full before/after state snapshots for replay
/// - Optional field-level diffs (when [`StateDiff`] is implemented)
/// - Optional user context and correlation ID
#[derive(Debug, Clone)]
pub struct AuditEntry<State: Clone> {
    /// Unique monotonically increasing entry identifier.
    pub id: u64,
    /// Timestamp in milliseconds since the Unix epoch.
    pub timestamp: u64,
    /// The name of the mutation that was executed.
    pub mutation_name: String,
    /// The name of the action that triggered this mutation, if any.
    pub action_name: Option<String>,
    /// Complete state snapshot before the mutation.
    pub state_before: State,
    /// Complete state snapshot after the mutation.
    pub state_after: State,
    /// Field-level changes, if available.
    pub changes: Vec<FieldChange>,
    /// User context at the time of the mutation, if available.
    pub user_context: Option<AuditUserContext>,
    /// Correlation ID for distributed tracing.
    pub correlation_id: Option<String>,
}

// ============================================================================
// AuditTrail
// ============================================================================

/// Thread-safe audit trail that records state mutation history.
///
/// `AuditTrail` captures before/after state snapshots for every recorded
/// mutation, providing full state replay capability and query methods for
/// filtering entries by mutation name, timestamp, or ID.
///
/// This is designed as backend/debugging infrastructure and uses
/// `Arc<RwLock<Vec<...>>>` for thread-safe storage rather than reactive
/// signals.
///
/// # Entry Limits
///
/// By default, the trail retains up to 1000 entries. When the limit is
/// reached, the oldest entries are removed. Use [`with_max_entries`](AuditTrail::with_max_entries)
/// to customize this limit.
///
/// # User Context
///
/// A user context provider can be registered via
/// [`with_user_context`](AuditTrail::with_user_context) to automatically
/// attach user identity information to each recorded entry.
pub struct AuditTrail<State: Clone + Send + Sync + 'static> {
    entries: Arc<RwLock<Vec<AuditEntry<State>>>>,
    next_id: Arc<AtomicU64>,
    max_entries: usize,
    user_context_provider: Arc<RwLock<Option<Box<dyn Fn() -> AuditUserContext + Send + Sync>>>>,
}

impl<State: Clone + Send + Sync + 'static> Clone for AuditTrail<State> {
    fn clone(&self) -> Self {
        Self {
            entries: Arc::clone(&self.entries),
            next_id: Arc::clone(&self.next_id),
            max_entries: self.max_entries,
            user_context_provider: Arc::clone(&self.user_context_provider),
        }
    }
}

impl<State: Clone + Send + Sync + 'static> Default for AuditTrail<State> {
    fn default() -> Self {
        Self::new()
    }
}

impl<State: Clone + Send + Sync + 'static> fmt::Debug for AuditTrail<State> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self
            .entries
            .read()
            .map(|e| e.len())
            .unwrap_or(0);
        f.debug_struct("AuditTrail")
            .field("entry_count", &count)
            .field("max_entries", &self.max_entries)
            .finish()
    }
}

impl<State: Clone + Send + Sync + 'static> AuditTrail<State> {
    /// Create a new audit trail with the default maximum of 1000 entries.
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            next_id: Arc::new(AtomicU64::new(1)),
            max_entries: 1000,
            user_context_provider: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the maximum number of entries to retain.
    ///
    /// When the limit is reached, the oldest entries are removed to make room.
    pub fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// Register a user context provider that is called for each recorded entry.
    ///
    /// The provider function is invoked at record time to capture the current
    /// user identity and session information.
    pub fn with_user_context(
        self,
        provider: impl Fn() -> AuditUserContext + Send + Sync + 'static,
    ) -> Self {
        if let Ok(mut guard) = self.user_context_provider.write() {
            *guard = Some(Box::new(provider));
        }
        self
    }

    /// Record a state mutation without field-level diffs.
    ///
    /// Captures full before/after state snapshots. If a user context provider
    /// has been registered, its output is attached to the entry.
    pub fn record(&self, mutation_name: &str, state_before: &State, state_after: &State) {
        let user_context = self
            .user_context_provider
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().map(|provider| provider()));

        let entry = AuditEntry {
            id: self.next_id.fetch_add(1, Ordering::SeqCst),
            timestamp: current_timestamp_ms(),
            mutation_name: mutation_name.to_string(),
            action_name: None,
            state_before: state_before.clone(),
            state_after: state_after.clone(),
            changes: Vec::new(),
            user_context,
            correlation_id: None,
        };

        self.push_entry(entry);
    }

    /// Record a state mutation with field-level diffs.
    ///
    /// Requires `State: StateDiff`. Computes the field-level differences
    /// between the before and after states and includes them in the entry.
    pub fn record_with_diff(
        &self,
        mutation_name: &str,
        state_before: &State,
        state_after: &State,
    ) where
        State: StateDiff,
    {
        let changes = state_before.diff(state_after);
        let user_context = self
            .user_context_provider
            .read()
            .ok()
            .and_then(|guard| guard.as_ref().map(|provider| provider()));

        let entry = AuditEntry {
            id: self.next_id.fetch_add(1, Ordering::SeqCst),
            timestamp: current_timestamp_ms(),
            mutation_name: mutation_name.to_string(),
            action_name: None,
            state_before: state_before.clone(),
            state_after: state_after.clone(),
            changes,
            user_context,
            correlation_id: None,
        };

        self.push_entry(entry);
    }

    /// Push an entry, enforcing the max_entries limit by trimming oldest entries.
    fn push_entry(&self, entry: AuditEntry<State>) {
        if let Ok(mut entries) = self.entries.write() {
            entries.push(entry);
            // Trim oldest entries if over the limit
            if entries.len() > self.max_entries {
                let excess = entries.len() - self.max_entries;
                entries.drain(..excess);
            }
        }
    }

    /// Return a snapshot of all audit entries, ordered from oldest to newest.
    pub fn entries(&self) -> Vec<AuditEntry<State>> {
        self.entries
            .read()
            .map(|e| e.clone())
            .unwrap_or_default()
    }

    /// Return entries filtered by mutation name.
    pub fn entries_for_mutation(&self, name: &str) -> Vec<AuditEntry<State>> {
        self.entries
            .read()
            .map(|e| {
                e.iter()
                    .filter(|entry| entry.mutation_name == name)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Return entries recorded at or after the given timestamp (milliseconds since epoch).
    pub fn entries_since(&self, timestamp: u64) -> Vec<AuditEntry<State>> {
        self.entries
            .read()
            .map(|e| {
                e.iter()
                    .filter(|entry| entry.timestamp >= timestamp)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Look up a single entry by its unique ID.
    pub fn entry_by_id(&self, id: u64) -> Option<AuditEntry<State>> {
        self.entries
            .read()
            .ok()
            .and_then(|e| e.iter().find(|entry| entry.id == id).cloned())
    }

    /// Return the `state_after` snapshot from the entry with the given ID.
    ///
    /// This enables state replay: you can inspect what the state looked like
    /// at any recorded point in time.
    pub fn state_at(&self, entry_id: u64) -> Option<State> {
        self.entry_by_id(entry_id)
            .map(|entry| entry.state_after)
    }

    /// Return the number of entries currently stored.
    pub fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    /// Check if the audit trail is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Remove all entries from the audit trail.
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestState {
        count: i32,
        name: String,
        active: bool,
    }

    impl TestState {
        fn new(count: i32, name: &str, active: bool) -> Self {
            Self {
                count,
                name: name.to_string(),
                active,
            }
        }
    }

    impl StateDiff for TestState {
        fn diff(&self, other: &Self) -> Vec<FieldChange> {
            let mut changes = Vec::new();

            if self.count != other.count {
                changes.push(FieldChange {
                    field_path: "count".into(),
                    old_value: format!("{}", self.count),
                    new_value: format!("{}", other.count),
                    change_type: ChangeType::Modified,
                });
            }

            if self.name != other.name {
                changes.push(FieldChange {
                    field_path: "name".into(),
                    old_value: self.name.clone(),
                    new_value: other.name.clone(),
                    change_type: ChangeType::Modified,
                });
            }

            if self.active != other.active {
                changes.push(FieldChange {
                    field_path: "active".into(),
                    old_value: format!("{}", self.active),
                    new_value: format!("{}", other.active),
                    change_type: ChangeType::Modified,
                });
            }

            changes
        }
    }

    #[test]
    fn test_audit_trail_record_and_query() {
        let trail: AuditTrail<TestState> = AuditTrail::new();
        let before = TestState::new(0, "Alice", true);
        let after = TestState::new(1, "Alice", true);

        trail.record("increment", &before, &after);

        assert_eq!(trail.len(), 1);
        assert!(!trail.is_empty());

        let entries = trail.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].mutation_name, "increment");
        assert_eq!(entries[0].state_before.count, 0);
        assert_eq!(entries[0].state_after.count, 1);
        assert!(entries[0].changes.is_empty());
        assert!(entries[0].user_context.is_none());
        assert!(entries[0].correlation_id.is_none());
        assert!(entries[0].timestamp > 0);
        assert!(entries[0].id > 0);
    }

    #[test]
    fn test_audit_trail_with_diffs() {
        let trail: AuditTrail<TestState> = AuditTrail::new();
        let before = TestState::new(0, "Alice", true);
        let after = TestState::new(1, "Bob", true);

        trail.record_with_diff("update", &before, &after);

        let entries = trail.entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].changes.len(), 2);

        let count_change = entries[0]
            .changes
            .iter()
            .find(|c| c.field_path == "count")
            .expect("should have count change");
        assert_eq!(count_change.old_value, "0");
        assert_eq!(count_change.new_value, "1");
        assert_eq!(count_change.change_type, ChangeType::Modified);

        let name_change = entries[0]
            .changes
            .iter()
            .find(|c| c.field_path == "name")
            .expect("should have name change");
        assert_eq!(name_change.old_value, "Alice");
        assert_eq!(name_change.new_value, "Bob");
    }

    #[test]
    fn test_audit_trail_max_entries() {
        let trail: AuditTrail<TestState> = AuditTrail::new().with_max_entries(3);
        let state = TestState::new(0, "test", true);

        for i in 0..5 {
            let after = TestState::new(i, "test", true);
            trail.record(&format!("mutation_{}", i), &state, &after);
        }

        assert_eq!(trail.len(), 3);

        let entries = trail.entries();
        // The oldest two entries (mutation_0, mutation_1) should be trimmed
        assert_eq!(entries[0].mutation_name, "mutation_2");
        assert_eq!(entries[1].mutation_name, "mutation_3");
        assert_eq!(entries[2].mutation_name, "mutation_4");
    }

    #[test]
    fn test_audit_trail_query_by_mutation() {
        let trail: AuditTrail<TestState> = AuditTrail::new();
        let state = TestState::new(0, "test", true);

        trail.record("increment", &state, &TestState::new(1, "test", true));
        trail.record("decrement", &state, &TestState::new(-1, "test", true));
        trail.record("increment", &state, &TestState::new(2, "test", true));
        trail.record("reset", &state, &TestState::new(0, "test", true));

        let increments = trail.entries_for_mutation("increment");
        assert_eq!(increments.len(), 2);
        assert!(increments.iter().all(|e| e.mutation_name == "increment"));

        let decrements = trail.entries_for_mutation("decrement");
        assert_eq!(decrements.len(), 1);

        let nonexistent = trail.entries_for_mutation("nonexistent");
        assert!(nonexistent.is_empty());
    }

    #[test]
    fn test_audit_trail_user_context() {
        let trail: AuditTrail<TestState> = AuditTrail::new().with_user_context(|| {
            AuditUserContext::new()
                .with_user_id("user-42")
                .with_session_id("sess-abc")
                .with_ip_address("192.168.1.1")
                .with_metadata("role", "admin")
        });

        let before = TestState::new(0, "test", true);
        let after = TestState::new(1, "test", true);
        trail.record("increment", &before, &after);

        let entries = trail.entries();
        assert_eq!(entries.len(), 1);

        let ctx = entries[0]
            .user_context
            .as_ref()
            .expect("should have user context");
        assert_eq!(ctx.user_id, Some("user-42".to_string()));
        assert_eq!(ctx.session_id, Some("sess-abc".to_string()));
        assert_eq!(ctx.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(ctx.metadata.get("role"), Some(&"admin".to_string()));
    }

    #[test]
    fn test_state_at_returns_snapshot() {
        let trail: AuditTrail<TestState> = AuditTrail::new();

        let s0 = TestState::new(0, "start", true);
        let s1 = TestState::new(1, "middle", true);
        let s2 = TestState::new(2, "end", false);

        trail.record("step_1", &s0, &s1);
        trail.record("step_2", &s1, &s2);

        let entries = trail.entries();
        let id1 = entries[0].id;
        let id2 = entries[1].id;

        let snapshot1 = trail.state_at(id1).expect("should find entry 1");
        assert_eq!(snapshot1.count, 1);
        assert_eq!(snapshot1.name, "middle");

        let snapshot2 = trail.state_at(id2).expect("should find entry 2");
        assert_eq!(snapshot2.count, 2);
        assert_eq!(snapshot2.name, "end");
        assert!(!snapshot2.active);

        // Non-existent ID returns None
        assert!(trail.state_at(9999).is_none());
    }

    #[test]
    fn test_field_change_display() {
        let modified = FieldChange {
            field_path: "user.name".into(),
            old_value: "Alice".into(),
            new_value: "Bob".into(),
            change_type: ChangeType::Modified,
        };
        assert_eq!(
            format!("{}", modified),
            "user.name: 'Alice' -> 'Bob'"
        );

        let added = FieldChange {
            field_path: "user.email".into(),
            old_value: String::new(),
            new_value: "bob@example.com".into(),
            change_type: ChangeType::Added,
        };
        assert_eq!(
            format!("{}", added),
            "user.email: added 'bob@example.com'"
        );

        let removed = FieldChange {
            field_path: "user.nickname".into(),
            old_value: "Bobby".into(),
            new_value: String::new(),
            change_type: ChangeType::Removed,
        };
        assert_eq!(
            format!("{}", removed),
            "user.nickname: removed 'Bobby'"
        );

        // Also verify ChangeType Display
        assert_eq!(format!("{}", ChangeType::Modified), "Modified");
        assert_eq!(format!("{}", ChangeType::Added), "Added");
        assert_eq!(format!("{}", ChangeType::Removed), "Removed");
    }
}
