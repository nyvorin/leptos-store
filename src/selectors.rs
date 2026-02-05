// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Fine-grained selector system for reactive state slices.
//!
//! Selectors allow components to subscribe to specific slices of store state
//! rather than the entire state object. Each selector returns a [`Memo<T>`]
//! that automatically recomputes only when the selected slice changes, providing
//! fine-grained reactivity and avoiding unnecessary re-renders.
//!
//! # Core Functions
//!
//! | Function | Purpose |
//! |----------|---------|
//! | [`create_selector`] | Extract a slice from a store's state |
//! | [`combine_selectors`] | Combine two selectors into one |
//! | [`map_selector`] | Transform a selector's output |
//! | [`filter_selector`] | Conditionally emit values from a selector |
//!
//! # Example
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_store::prelude::*;
//!
//! // Select just the count from a store
//! let count = create_selector(&counter_store, |state| state.count);
//!
//! // Transform it
//! let doubled = map_selector(count, |c| c * 2);
//!
//! // Combine two selectors
//! let summary = combine_selectors(count, doubled, |c, d| format!("{c} -> {d}"));
//! ```

use crate::store::Store;
use leptos::prelude::*;

/// Create a memoized selector that extracts a slice from a store's state.
///
/// The returned [`Memo<T>`] only recomputes when the selected value changes,
/// enabling fine-grained reactivity. Components that use this selector will
/// only re-render when the specific slice they care about is modified.
///
/// # Arguments
///
/// * `store` - A reference to any type implementing [`Store`].
/// * `selector_fn` - A closure that extracts the desired slice from the store's state.
///
/// # Returns
///
/// A [`Memo<T>`] containing the selected value.
///
/// # Example
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
///
/// let count = create_selector(&my_store, |state| state.count);
/// let name = create_selector(&my_store, |state| state.name.clone());
/// ```
pub fn create_selector<S, T>(
    store: &S,
    selector_fn: impl Fn(&S::State) -> T + Send + Sync + 'static,
) -> Memo<T>
where
    S: Store,
    T: Clone + PartialEq + Send + Sync + 'static,
{
    let state_signal = store.state();
    Memo::new(move |_prev| state_signal.with(|s| selector_fn(s)))
}

/// Combine two memoized selectors into a single selector.
///
/// The returned [`Memo<T>`] recomputes whenever either input selector changes.
/// This is useful for deriving values that depend on multiple state slices,
/// possibly from different stores.
///
/// # Arguments
///
/// * `a` - The first selector memo.
/// * `b` - The second selector memo.
/// * `combiner` - A closure that produces a combined value from both selectors.
///
/// # Returns
///
/// A [`Memo<T>`] containing the combined value.
///
/// # Example
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
///
/// let count = create_selector(&store, |s| s.count);
/// let name = create_selector(&store, |s| s.name.clone());
/// let display = combine_selectors(count, name, |c, n| format!("{n}: {c}"));
/// ```
pub fn combine_selectors<A, B, T>(
    a: Memo<A>,
    b: Memo<B>,
    combiner: impl Fn(&A, &B) -> T + Send + Sync + 'static,
) -> Memo<T>
where
    A: Clone + PartialEq + Send + Sync + 'static,
    B: Clone + PartialEq + Send + Sync + 'static,
    T: Clone + PartialEq + Send + Sync + 'static,
{
    Memo::new(move |_prev| {
        let val_a = a.get();
        let val_b = b.get();
        combiner(&val_a, &val_b)
    })
}

/// Transform a selector's output using a mapping function.
///
/// The returned [`Memo<T>`] recomputes whenever the source selector changes.
/// This is the selector equivalent of `Iterator::map`.
///
/// # Arguments
///
/// * `selector` - The source selector memo.
/// * `mapper` - A closure that transforms the selector's value.
///
/// # Returns
///
/// A [`Memo<T>`] containing the transformed value.
///
/// # Example
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
///
/// let count = create_selector(&store, |s| s.count);
/// let doubled = map_selector(count, |c| c * 2);
/// let is_positive = map_selector(count, |c| *c > 0);
/// ```
pub fn map_selector<A, T>(
    selector: Memo<A>,
    mapper: impl Fn(&A) -> T + Send + Sync + 'static,
) -> Memo<T>
where
    A: Clone + PartialEq + Send + Sync + 'static,
    T: Clone + PartialEq + Send + Sync + 'static,
{
    Memo::new(move |_prev| {
        let val = selector.get();
        mapper(&val)
    })
}

/// Filter a selector's output based on a predicate.
///
/// The returned [`Memo<Option<T>>`] emits `Some(value)` when the predicate
/// returns `true`, and `None` otherwise. This is useful for conditionally
/// displaying data based on state criteria.
///
/// # Arguments
///
/// * `selector` - The source selector memo.
/// * `predicate` - A closure that returns `true` to include the value.
///
/// # Returns
///
/// A [`Memo<Option<T>>`] that is `Some` when the predicate passes.
///
/// # Example
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use leptos_store::prelude::*;
///
/// let count = create_selector(&store, |s| s.count);
/// let positive_only = filter_selector(count, |c| *c > 0);
/// // positive_only.get() returns Some(5) when count is 5, None when count is -1
/// ```
pub fn filter_selector<T>(
    selector: Memo<T>,
    predicate: impl Fn(&T) -> bool + Send + Sync + 'static,
) -> Memo<Option<T>>
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    Memo::new(move |_prev| {
        let val = selector.get();
        if predicate(&val) { Some(val) } else { None }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    #[derive(Clone, Debug, Default, PartialEq)]
    struct TestState {
        count: i32,
        name: String,
    }

    #[derive(Clone)]
    struct TestStore {
        state: RwSignal<TestState>,
    }

    impl TestStore {
        fn new(count: i32, name: &str) -> Self {
            Self {
                state: RwSignal::new(TestState {
                    count,
                    name: name.to_string(),
                }),
            }
        }
    }

    impl Store for TestStore {
        type State = TestState;

        fn state(&self) -> ReadSignal<Self::State> {
            self.state.read_only()
        }
    }

    #[test]
    fn test_create_selector_extracts_count() {
        let store = TestStore::new(42, "Alice");
        let count = create_selector(&store, |s| s.count);
        assert_eq!(count.get(), 42);
    }

    #[test]
    fn test_create_selector_extracts_name() {
        let store = TestStore::new(0, "Bob");
        let name = create_selector(&store, |s| s.name.clone());
        assert_eq!(name.get(), "Bob");
    }

    #[test]
    fn test_combine_selectors_merges_two() {
        let store = TestStore::new(10, "Charlie");
        let count = create_selector(&store, |s| s.count);
        let name = create_selector(&store, |s| s.name.clone());

        let combined = combine_selectors(count, name, |c, n| format!("{n}: {c}"));
        assert_eq!(combined.get(), "Charlie: 10");
    }

    #[test]
    fn test_combine_selectors_from_different_stores() {
        let store_a = TestStore::new(5, "A");
        let store_b = TestStore::new(7, "B");

        let count_a = create_selector(&store_a, |s| s.count);
        let count_b = create_selector(&store_b, |s| s.count);

        let sum = combine_selectors(count_a, count_b, |a, b| a + b);
        assert_eq!(sum.get(), 12);
    }

    #[test]
    fn test_map_selector_transforms_value() {
        let store = TestStore::new(5, "test");
        let count = create_selector(&store, |s| s.count);
        let doubled = map_selector(count, |c| c * 2);
        assert_eq!(doubled.get(), 10);
    }

    #[test]
    fn test_map_selector_type_change() {
        let store = TestStore::new(42, "test");
        let count = create_selector(&store, |s| s.count);
        let as_string = map_selector(count, |c| format!("count={c}"));
        assert_eq!(as_string.get(), "count=42");
    }

    #[test]
    fn test_filter_selector_passes_when_true() {
        let store = TestStore::new(5, "test");
        let count = create_selector(&store, |s| s.count);
        let positive = filter_selector(count, |c| *c > 0);
        assert_eq!(positive.get(), Some(5));
    }

    #[test]
    fn test_filter_selector_returns_none_when_false() {
        let store = TestStore::new(-3, "test");
        let count = create_selector(&store, |s| s.count);
        let positive = filter_selector(count, |c| *c > 0);
        assert_eq!(positive.get(), None);
    }

    #[test]
    fn test_selector_chain() {
        let store = TestStore::new(8, "test");
        let count = create_selector(&store, |s| s.count);
        let doubled = map_selector(count, |c| c * 2);
        let filtered = filter_selector(doubled, |v| *v > 10);
        assert_eq!(filtered.get(), Some(16));
    }

    #[test]
    fn test_filter_selector_boundary() {
        let store = TestStore::new(0, "test");
        let count = create_selector(&store, |s| s.count);
        let non_zero = filter_selector(count, |c| *c != 0);
        assert_eq!(non_zero.get(), None);
    }

    #[test]
    fn test_selector_macro() {
        use crate::selector;

        let store = TestStore::new(7, "Eve");

        selector! {
            store: &store,
            count_val: |s: &TestState| -> i32 { s.count },
            name_val: |s: &TestState| -> String { s.name.clone() },
        }

        assert_eq!(count_val.get(), 7);
        assert_eq!(name_val.get(), "Eve");
    }
}
