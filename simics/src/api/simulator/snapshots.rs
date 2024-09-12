// Copyright (C) 2024 Intel Corporation
// SPDX-License-Identifier: Apache-2.0

//! Experimental snapshot APIs

use crate::{simics_exception, AttrValue, Result};
use raw_cstr::raw_cstr;

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
    simics_version = "6.0.173",
    simics_version = "6.0.174",
    simics_version = "6.0.175",
    simics_version = "6.0.176",
    simics_version = "6.0.177",
    simics_version = "6.0.178",
    simics_version = "6.0.179",
)))]
type SnapshotError = crate::sys::snapshot_error_t;

#[cfg(simics_version = "6.0.173")]
#[simics_exception]
/// Save a snapshot with a name. This function was renamed to
/// `VT_take_snapshot` in version 6.0.180
pub fn save_snapshot<S>(name: S) -> Result<()>
where
    S: AsRef<str>,
{
    Ok(unsafe { crate::sys::VT_save_snapshot(raw_cstr(name)?) })
}

#[cfg(any(
    simics_version = "6.0.174",
    simics_version = "6.0.175",
    simics_version = "6.0.176",
    simics_version = "6.0.177",
    simics_version = "6.0.178",
    simics_version = "6.0.179",
))]
#[simics_exception]
/// Save a snapshot with a name. This function was renamed to
/// `VT_take_snapshot` in version 6.0.180
pub fn save_snapshot<S>(name: S) -> Result<bool>
where
    S: AsRef<str>,
{
    Ok(unsafe { crate::sys::VT_save_snapshot(raw_cstr(name)?) })
}

#[cfg(all(
    not(any(
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
        simics_version = "6.0.173",
        simics_version = "6.0.174",
        simics_version = "6.0.175",
        simics_version = "6.0.176",
        simics_version = "6.0.177",
        simics_version = "6.0.178",
        simics_version = "6.0.179",
    )),
    simics_version = "6"
))]
/// Save a snapshot with a name. API deprecated as of SIMICS 6.0.180
pub fn save_snapshot<S>(name: S) -> Result<SnapshotError>
where
    S: AsRef<str>,
{
    Ok(unsafe { crate::sys::VT_take_snapshot(raw_cstr(name)?) })
}

#[cfg(all(
    not(any(
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
        simics_version = "6.0.173",
        simics_version = "6.0.174",
        simics_version = "6.0.175",
        simics_version = "6.0.176",
        simics_version = "6.0.177",
        simics_version = "6.0.178",
        simics_version = "6.0.179",
    )),
    simics_version = "6"
))]
#[simics_exception]
/// Take a snapshot with a name
pub fn take_snapshot<S>(name: S) -> Result<SnapshotError>
where
    S: AsRef<str>,
{
    Ok(unsafe { crate::sys::VT_take_snapshot(raw_cstr(name)?) })
}

#[cfg(simics_version = "6.0.173")]
#[simics_exception]
/// Restore a snapshot with a name
pub fn restore_snapshot<S>(index: i32) -> Result<bool>
where
    S: AsRef<str>,
{
    Ok(unsafe { crate::sys::VT_restore_snapshot(index) })
}

#[cfg(all(
    not(any(
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
        simics_version = "6.0.173",
        simics_version = "6.0.174",
        simics_version = "6.0.175",
        simics_version = "6.0.176",
        simics_version = "6.0.177",
        simics_version = "6.0.178",
        simics_version = "6.0.179",
    )),
    simics_version = "6"
))]
#[simics_exception]
/// Restore a snapshot with a name
pub fn restore_snapshot<S>(name: S) -> Result<SnapshotError>
where
    S: AsRef<str>,
{
    Ok(unsafe { crate::sys::VT_restore_snapshot(raw_cstr(name)?) })
}

#[cfg(simics_version = "6.0.173")]
#[simics_exception]
/// Delete a snapshot with a name
pub fn delete_snapshot(index: i32) -> Result<bool> {
    Ok(unsafe { crate::sys::VT_delete_snapshot(index) })
}

#[cfg(all(
    not(any(
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
        simics_version = "6.0.173",
        simics_version = "6.0.174",
        simics_version = "6.0.175",
        simics_version = "6.0.176",
        simics_version = "6.0.177",
        simics_version = "6.0.178",
        simics_version = "6.0.179",
    )),
    simics_version = "6"
))]
#[simics_exception]
/// Delete a snapshot with a name
pub fn delete_snapshot<S>(name: S) -> Result<SnapshotError>
where
    S: AsRef<str>,
{
    Ok(unsafe { crate::sys::VT_delete_snapshot(raw_cstr(name)?) })
}

#[cfg(all(
    not(any(
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
    )),
    simics_version = "6",
))]
#[simics_exception]
/// Get the total size used by all snapshots
pub fn snapshot_size_used() -> AttrValue {
    unsafe { crate::sys::VT_snapshot_size_used() }.into()
}

#[cfg(all(
    not(any(
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
    )),
    simics_version = "6",
))]
#[simics_exception]
/// Get the list of all snapshots
pub fn list_snapshots() -> AttrValue {
    unsafe { crate::sys::VT_list_snapshots() }.into()
}

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
#[simics_exception]
/// Set snapshots to ignore a given class by name
pub fn snapshots_ignore_class<S>(class_name: S) -> Result<()>
where
    S: AsRef<str>,
{
    unsafe { crate::sys::VT_snapshots_ignore_class(raw_cstr(class_name)?) };
    Ok(())
}

#[deprecated = "Use `take_snapshot` instead`"]
#[cfg(simics_version = "7")]
#[simics_exception]
/// Take a snapshot with a name
pub fn save_snapshot<S>(name: S) -> Result<SnapshotError>
where
    S: AsRef<str>,
{
    Ok(unsafe { crate::sys::SIM_take_snapshot(raw_cstr(name)?) })
}

#[cfg(simics_version = "7")]
#[simics_exception]
/// Take a snapshot with a name
pub fn take_snapshot<S>(name: S) -> Result<SnapshotError>
where
    S: AsRef<str>,
{
    Ok(unsafe { crate::sys::SIM_take_snapshot(raw_cstr(name)?) })
}

#[cfg(simics_version = "7")]
#[simics_exception]
/// Restore a snapshot with a name
pub fn restore_snapshot<S>(name: S) -> Result<SnapshotError>
where
    S: AsRef<str>,
{
    Ok(unsafe { crate::sys::SIM_restore_snapshot(raw_cstr(name)?) })
}

#[cfg(simics_version = "7")]
#[simics_exception]
/// Delete a snapshot with a name
pub fn delete_snapshot<S>(name: S) -> Result<SnapshotError>
where
    S: AsRef<str>,
{
    Ok(unsafe { crate::sys::SIM_delete_snapshot(raw_cstr(name)?) })
}

#[cfg(simics_version = "7")]
#[simics_exception]
/// Get the list of all snapshots
pub fn list_snapshots() -> AttrValue {
    unsafe { crate::sys::SIM_list_snapshots() }.into()
}
