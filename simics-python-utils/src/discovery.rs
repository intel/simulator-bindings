// Copyright (C) 2024 Intel Corporation
// SPDX-License-Identifier: Apache-2.0

use crate::{environment::PackageSource, PythonEnvironment, PythonVersion};
use anyhow::{anyhow, Context, Result};
use ispm_wrapper::{
    data::InstalledPackage,
    ispm::{self, GlobalOptions},
};
use std::{
    env,
    fs::read_dir,
    path::{Path, PathBuf},
};
use versions::Versioning;

#[cfg(not(windows))]
/// The name of the binary/library/object subdirectory on linux systems
pub const HOST_DIRNAME: &str = "linux64";

#[cfg(windows)]
/// The name of the binary/library/object subdirectory on Windows systems  
pub const HOST_DIRNAME: &str = "win64";

/// Environment variable name for Python include path override
pub const PYTHON3_INCLUDE_ENV: &str = "PYTHON3_INCLUDE";
/// Environment variable name for Python library path override  
pub const PYTHON3_LDFLAGS_ENV: &str = "PYTHON3_LDFLAGS";

/// Find the latest Python package (1033) for a given Simics major version using ISPM API
fn find_latest_python_package(simics_major_version: u32) -> Result<InstalledPackage> {
    let packages = ispm::packages::list(&GlobalOptions::default())
        .map_err(|e| anyhow!("Failed to query ISPM for installed packages: {}", e))?;

    let installed = packages
        .installed_packages
        .as_ref()
        .ok_or_else(|| anyhow!("No installed packages found"))?;

    // Filter for Python packages (1033) matching the major version
    let python_packages: Vec<_> = installed
        .iter()
        .filter(|pkg| pkg.package_number == 1033)
        .filter(|pkg| pkg.name == "Python")
        .filter(|pkg| {
            // Check if the version starts with the expected major version
            pkg.version
                .starts_with(&format!("{}.", simics_major_version))
        })
        .collect();

    if python_packages.is_empty() {
        return Err(anyhow!(
            "No Python packages found for Simics major version {}",
            simics_major_version
        ));
    }

    // Find the package with the highest version
    let latest_package = python_packages
        .iter()
        .max_by(|a, b| {
            let version_a = Versioning::new(&a.version).unwrap_or_default();
            let version_b = Versioning::new(&b.version).unwrap_or_default();
            version_a.cmp(&version_b)
        })
        .ok_or_else(|| anyhow!("Failed to find latest Python package"))?;

    Ok((*latest_package).clone())
}

/// Extract Simics major version using hints and fallbacks
fn detect_simics_major_version_from_base(simics_base: &Path) -> Result<u32> {
    if let Some(dir_name) = simics_base.file_name().and_then(|n| n.to_str()) {
        // Look for patterns like "simics-7.57.0" or "simics-6.0.191"
        if let Some(version_part) = dir_name.strip_prefix("simics-") {
            if let Some(major_str) = version_part.split('.').next() {
                if let Ok(major) = major_str.parse::<u32>() {
                    return Ok(major);
                }
            }
        }
    }

    Err(anyhow!(
        "Unable to determine Simics major version from SIMICS base path {}; expected directory name like simics-x.x.x",
        simics_base.display()
    ))
}

/// Auto-discover Python environment from SIMICS_BASE environment variable
pub fn discover_python_environment() -> Result<PythonEnvironment> {
    let simics_base =
        env::var("SIMICS_BASE").map_err(|_| anyhow!("SIMICS_BASE environment variable not set"))?;

    discover_python_environment_from_base(simics_base)
}

/// Discover Python environment from a specific Simics base path
pub fn discover_python_environment_from_base<P: AsRef<Path>>(
    simics_base: P,
) -> Result<PythonEnvironment> {
    let simics_base = simics_base.as_ref();

    // Try traditional paths first (Simics 1000)
    let traditional_err = match try_traditional_paths(simics_base) {
        Ok(env) => return Ok(env.with_source(PackageSource::Traditional)),
        Err(err) => err.context("Traditional discovery failed"),
    };

    // Try dynamic discovery of separate Python package (Simics 1033)
    let dynamic_err = match try_dynamic_python_package_discovery(simics_base) {
        Ok(env) => return Ok(env.with_source(PackageSource::SeparatePackage)),
        Err(err) => err.context("Dynamic discovery failed"),
    };

    Err(anyhow!(
        "Python environment not found in traditional location ({}) or through dynamic package discovery\nTraditional error: {:#}\nDynamic error: {:#}",
        simics_base.join(HOST_DIRNAME).display(),
        traditional_err,
        dynamic_err
    ))
}

/// Try to discover Python environment from traditional Simics base package paths
fn try_traditional_paths(simics_base: &Path) -> Result<PythonEnvironment> {
    let base_path = simics_base.join(HOST_DIRNAME);
    discover_from_base_path(base_path)
}

/// Try to discover Python environment using dynamic ISPM-based package discovery
fn try_dynamic_python_package_discovery(simics_base: &Path) -> Result<PythonEnvironment> {
    // Detect the Simics major version using multiple strategies
    let major_version = detect_simics_major_version_from_base(simics_base)?;

    // Find the latest Python package for this specific major version
    let python_package = find_latest_python_package(major_version)?;

    println!(
        "cargo:warning=Using dynamically discovered Python package: {} (version {})",
        python_package.name, python_package.version
    );

    // Get the first path from the installed package
    let package_path = python_package.paths.first().ok_or_else(|| {
        anyhow!(
            "No installation paths found for Python package {}",
            python_package.name
        )
    })?;

    // Construct the path to the host-specific directory
    let python_package_path = package_path.join(HOST_DIRNAME);

    discover_from_base_path(python_package_path)
}

/// Common discovery logic for both package types
fn discover_from_base_path(base_path: PathBuf) -> Result<PythonEnvironment> {
    // SIMICS_BASE/HOST_DIRNAME/bin/mini-python
    let mini_python = find_mini_python(&base_path)?;
    // SIMICS_BASE/HOST_DIRNAME/include/python3.X
    let include_dir = find_python_include(&base_path)?;
    // SIMICS_BASE/HOST_DIRNAME/sys/lib/libpython3.X.so.Y.Z
    let (lib_dir, lib_path) = find_python_library(&base_path)?;
    let version = PythonVersion::parse_from_include_dir(&include_dir)?;

    let env = PythonEnvironment::new(
        mini_python,
        include_dir,
        lib_dir,
        lib_path,
        version,
        PackageSource::Traditional, // Will be updated by caller
    );

    // Validate the environment before returning
    env.validate()?;

    Ok(env)
}

/// Find mini-python executable in the given base path
fn find_mini_python(base_path: &Path) -> Result<PathBuf> {
    #[cfg(unix)]
    let executable_name = "mini-python";

    #[cfg(windows)]
    let executable_name = "mini-python.exe";

    let mini_python_path = base_path.join("bin").join(executable_name);

    if mini_python_path.exists() {
        Ok(mini_python_path)
    } else {
        Err(anyhow!(
            "Mini-python executable not found at {}",
            mini_python_path.display()
        ))
    }
}

/// Find Python include directory with python3.X subdirectory
fn find_python_include(base_path: &Path) -> Result<PathBuf> {
    // Check environment variable first for compatibility
    if let Ok(include_env) = env::var(PYTHON3_INCLUDE_ENV) {
        // Extract path from -I flag if present
        let include_path = if let Some(path) = include_env.strip_prefix("-I") {
            PathBuf::from(path)
        } else {
            PathBuf::from(include_env)
        };

        if include_path.exists() {
            return Ok(include_path);
        }
    }

    let include_base = base_path.join("include");
    find_python_subdir(&include_base)
}

/// Find the python3.X subdirectory in the include directory
fn find_python_subdir(include_dir: &Path) -> Result<PathBuf> {
    if !include_dir.exists() {
        return Err(anyhow!(
            "Include directory does not exist: {}",
            include_dir.display()
        ));
    }

    let entries = read_dir(include_dir).map_err(|e| {
        anyhow!(
            "Failed to read include directory {}: {}",
            include_dir.display(),
            e
        )
    })?;

    let python_dirs: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("python"))
                .unwrap_or(false)
        })
        .collect();

    match python_dirs.len() {
        0 => Err(anyhow!(
            "No python* subdirectory found in {}",
            include_dir.display()
        )),
        1 => Ok(python_dirs.into_iter().next().unwrap()),
        _ => Err(anyhow!(
            "Multiple python* subdirectories found in {}, expected exactly one",
            include_dir.display()
        )),
    }
}

/// Find Python library directory and specific libpython file
fn find_python_library(base_path: &Path) -> Result<(PathBuf, PathBuf)> {
    // Check environment variable first for compatibility
    if let Ok(lib_env) = env::var(PYTHON3_LDFLAGS_ENV) {
        let lib_path = PathBuf::from(lib_env);
        if lib_path.exists() {
            let lib_dir = lib_path
                .parent()
                .ok_or_else(|| anyhow!("Library path has no parent directory"))?
                .to_path_buf();
            return Ok((lib_dir, lib_path));
        }
    }

    let sys_lib_dir = base_path.join("sys").join("lib");
    find_libpython_in_dir(&sys_lib_dir)
}

/// Find a libpython*.so file in the given directory
fn find_libpython_in_dir(lib_dir: &Path) -> Result<(PathBuf, PathBuf)> {
    if !lib_dir.exists() {
        return Err(anyhow!(
            "Library directory does not exist: {}",
            lib_dir.display()
        ));
    }

    let entries = read_dir(lib_dir).map_err(|e| {
        anyhow!(
            "Failed to read library directory {}: {}",
            lib_dir.display(),
            e
        )
    })?;

    let mut libpython_files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .filter(|path| {
            if let Some(file_name) = path.file_name().and_then(|name| name.to_str()) {
                file_name.starts_with("libpython")
            } else {
                false
            }
        })
        .collect();

    if libpython_files.len() > 1 {
        libpython_files.retain(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name != "libpython3.so")
                .unwrap_or(false)
        });
    }

    if libpython_files.len() > 1 {
        libpython_files.retain(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| !name.ends_with(".so"))
                .unwrap_or(false)
        });
    }

    match libpython_files.len() {
        0 => Err(anyhow!(
            "No libpython3.x.so file found in {}",
            lib_dir.display()
        )),
        1 => {
            let lib_path = libpython_files.into_iter().next().unwrap();
            Ok((lib_dir.to_path_buf(), lib_path))
        }
        _ => Err(anyhow!(
            "Multiple libpython files found in {}, expected exactly one",
            lib_dir.display()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_mock_simics_structure(base_dir: &Path, python_version: &str) -> Result<()> {
        // Create traditional structure
        let host_dir = base_dir.join(HOST_DIRNAME);
        fs::create_dir_all(host_dir.join("bin"))?;
        fs::create_dir_all(
            host_dir
                .join("include")
                .join(format!("python{}", python_version)),
        )?;
        fs::create_dir_all(host_dir.join("sys").join("lib"))?;

        // Create mock files
        let mini_python = if cfg!(windows) {
            "mini-python.exe"
        } else {
            "mini-python"
        };
        fs::write(host_dir.join("bin").join(mini_python), "")?;
        fs::write(
            host_dir
                .join("sys")
                .join("lib")
                .join(format!("libpython{}.so", python_version)),
            "",
        )?;

        Ok(())
    }

    #[test]
    fn test_discover_traditional_structure() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let base_path = temp_dir.path();

        create_mock_simics_structure(base_path, "3.9")?;

        let env = discover_python_environment_from_base(base_path)?;

        assert_eq!(env.package_source, PackageSource::Traditional);
        assert_eq!(env.version.major, 3);
        assert_eq!(env.version.minor, 9);
        assert!(env.mini_python.exists());
        assert!(env.include_dir.exists());
        assert!(env.lib_path.exists());

        Ok(())
    }

    #[test]
    fn test_version_parsing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let include_dir = temp_dir.path().join("python3.9.10");
        fs::create_dir_all(&include_dir)?;

        let version = PythonVersion::parse_from_include_dir(&include_dir)?;
        assert_eq!(version.major, 3);
        assert_eq!(version.minor, 9);
        assert_eq!(version.patch, 10);

        Ok(())
    }

    #[test]
    fn test_multiple_python_include_dirs_error() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let include_dir = temp_dir.path().join("include");
        fs::create_dir_all(include_dir.join("python3.9"))?;
        fs::create_dir_all(include_dir.join("python3.10"))?;

        let err = find_python_subdir(&include_dir).unwrap_err();
        assert!(err.to_string().contains("Multiple python* subdirectories"));

        Ok(())
    }

    #[test]
    fn test_multiple_libpython_files_error() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let lib_dir = temp_dir.path();
        fs::write(lib_dir.join("libpython3.9.so.1.0"), "")?;
        fs::write(lib_dir.join("libpython3.10.so.1.0"), "")?;

        let err = find_libpython_in_dir(lib_dir).unwrap_err();
        assert!(err.to_string().contains("Multiple libpython files"));

        Ok(())
    }

    #[test]
    fn test_detect_simics_major_version_from_base() -> Result<()> {
        let base = PathBuf::from("/opt/simics/simics-7.38.0");
        let major = detect_simics_major_version_from_base(&base)?;
        assert_eq!(major, 7);

        Ok(())
    }

    #[test]
    fn test_detect_simics_major_version_from_base_invalid() {
        let base = PathBuf::from("/opt/simics/current");
        let err = detect_simics_major_version_from_base(&base).unwrap_err();
        assert!(err
            .to_string()
            .contains("expected directory name like simics-x.x.x"));
    }
}
