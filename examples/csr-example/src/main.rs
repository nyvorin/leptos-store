// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 nyvorin

//! Binary entry point for the CSR Todo Example
//!
//! This example is designed for CSR mode. Run with:
//!   trunk serve --features csr

#[cfg(feature = "csr")]
fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(csr_example::App);
}

#[cfg(not(feature = "csr"))]
fn main() {
    println!("This example is designed for CSR mode. Run with: trunk serve --features csr");
}
