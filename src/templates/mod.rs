// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Store templates for common patterns.
//!
//! This module provides ready-to-use store implementations for common
//! use cases. These templates can be used directly or as starting points
//! for custom implementations.
//!
//! # Available Templates
//!
//! - `FeatureFlagStore` - Feature flag management with remote sync
//!
//! # Feature
//!
//! Requires the `templates` feature.

pub mod feature_flags;

pub use feature_flags::*;
