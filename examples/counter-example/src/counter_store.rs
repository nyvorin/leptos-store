// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Counter Store
//!
//! This module demonstrates the `store!` macro for defining
//! a complete store with the Enterprise Mode pattern:
//!
//! - **Getters**: Public, read-only derived values
//! - **Mutators**: Private, internal state modification
//! - **Actions**: Public, the only external API for writes
//!
//! External code cannot call mutators directly - they must use actions.

use leptos_store::store;

// Define the complete counter store using the store! macro
// Note: Use `this` (or any identifier) instead of `self` in bodies
// - Getters use this.read(|s| ...) for reads
// - Mutators use this.mutate(|s| ...) for writes (PRIVATE)
// - Actions are the PUBLIC API for state changes
store! {
    pub CounterStore {
        state CounterState {
            count: i32 = 0,
        }

        getters {
            doubled(this) -> i32 {
                this.read(|s| s.count * 2)
            }

            is_positive(this) -> bool {
                this.read(|s| s.count > 0)
            }

            is_negative(this) -> bool {
                this.read(|s| s.count < 0)
            }

            is_prime(this) -> bool {
                this.read(|s| s.count > 1 && (2..=s.count-1).all(|i| s.count % i != 0))
            }
        }

        // PRIVATE - only callable from actions within this store
        mutators {
            set_count(this, value: i32) {
                this.mutate(|s| s.count = value);
            }

            add_to_count(this, delta: i32) {
                this.mutate(|s| s.count += delta);
            }
        }

        // PUBLIC - the external API for state changes
        actions {
            increment(this) {
                this.add_to_count(1);
            }

            decrement(this) {
                this.add_to_count(-1);
            }

            reset(this) {
                this.set_count(0);
            }

            /// Set the counter to a specific value.
            /// This is a public action that delegates to the private mutator.
            set(this, value: i32) {
                this.set_count(value);
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use leptos::prelude::Get;
    use leptos_store::store::Store;

    #[test]
    fn test_counter_state_default() {
        let state = CounterState::default();
        assert_eq!(state.count, 0);
    }

    #[test]
    fn test_counter_store_creation() {
        let store = CounterStore::new();
        assert_eq!(store.state().get().count, 0);
    }

    #[test]
    fn test_increment_action() {
        let store = CounterStore::new();
        store.increment(); // Public action
        assert_eq!(store.state().get().count, 1);
    }

    #[test]
    fn test_decrement_action() {
        let store = CounterStore::new();
        store.decrement(); // Public action
        assert_eq!(store.state().get().count, -1);
    }

    #[test]
    fn test_doubled_getter() {
        let store = CounterStore::new();
        store.set(5); // Public action (delegates to private set_count mutator)
        assert_eq!(store.doubled(), 10);
    }

    #[test]
    fn test_is_positive_negative_getters() {
        let store = CounterStore::new();

        assert!(!store.is_positive());
        assert!(!store.is_negative());

        store.increment(); // Public action
        assert!(store.is_positive());
        assert!(!store.is_negative());

        store.set(-5); // Public action
        assert!(!store.is_positive());
        assert!(store.is_negative());
    }

    #[test]
    fn test_reset_action() {
        let store = CounterStore::new();
        store.set(100); // Public action
        store.reset(); // Public action
        assert_eq!(store.state().get().count, 0);
    }

    #[test]
    fn test_with_state() {
        let state = CounterState { count: 42 };
        let store = CounterStore::with_state(state);
        assert_eq!(store.state().get().count, 42);
    }

    // Note: The following would NOT compile because mutators are private:
    // store.set_count(5);     // ERROR: private method
    // store.add_to_count(1);  // ERROR: private method
    //
    // External code must use public actions:
    // store.set(5);           // OK: public action
    // store.increment();      // OK: public action
}
