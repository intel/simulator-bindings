// Copyright (C) 2024 Intel Corporation
// SPDX-License-Identifier: Apache-2.0

//! Simulator control APIs

pub mod breakpoints;
pub mod callbacks;
pub mod configuration;
pub mod control;
pub mod debugger;
pub mod embed;
pub mod hap_consumer;
pub mod host_profiling;
pub mod memory;
pub mod modules;
pub mod paths;
pub mod processor;
pub mod python;
// NOTE: Reverse execution is only available in Simics 6
#[cfg(simics_version = "6")]
pub mod rev_exec;
pub mod script;
pub mod sim_caches;
pub mod sim_conf_object;
pub mod sim_get_class;
// NOTE: Snapshots are only available in Simics 6 starting after 6.0.173
#[cfg(not(any(
    simics_version = "6.0.163",
    simics_version = "6.0.164",
    simics_version = "6.0.165",
    simics_version = "6.0.166",
    simics_version = "6.0.167",
    simics_version = "6.0.168",
    simics_version = "6.0.169",
    simics_version = "6.0.170",
    simics_version = "6.0.171",
    simics_version = "6.0.172",
)))]
pub mod snapshots;

pub use breakpoints::*;
pub use callbacks::*;
pub use configuration::*;
pub use control::*;
pub use debugger::*;
pub use embed::*;
pub use hap_consumer::*;
pub use host_profiling::*;
pub use memory::*;
pub use modules::*;
pub use paths::*;
pub use processor::*;
pub use python::*;
// NOTE: Reverse execution is only available in Simics 6
#[cfg(simics_version = "6")]
pub use rev_exec::*;
pub use script::*;
pub use sim_caches::*;
pub use sim_conf_object::*;
pub use sim_get_class::*;
// NOTE: Snapshots are only available in Simics 6 starting after 6.0.173
#[cfg(not(any(
    simics_version = "6.0.163",
    simics_version = "6.0.164",
    simics_version = "6.0.165",
    simics_version = "6.0.166",
    simics_version = "6.0.167",
    simics_version = "6.0.168",
    simics_version = "6.0.169",
    simics_version = "6.0.170",
    simics_version = "6.0.171",
    simics_version = "6.0.172",
)))]
pub use snapshots::*;
