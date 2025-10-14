// Copyright (C) 2024 Intel Corporation
// SPDX-License-Identifier: Apache-2.0

//! Python environment detection utilities for Intel® Simics® Simulator
//!
//! This crate provides unified Python detection functionality that works with both:
//! - Traditional Simics base package (1000) that includes Python
//! - Separate Simics Python package (1033) for Simics 7.28.0+

#![deny(clippy::unwrap_used)]
#![deny(missing_docs)]
#![forbid(unsafe_code)]

mod discovery;
mod environment;
mod version;

pub use discovery::*;
pub use environment::*;
pub use version::*;
