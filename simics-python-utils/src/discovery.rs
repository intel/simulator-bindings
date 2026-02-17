// Copyright (C) 2024 Intel Corporation
// SPDX-License-Identifier: Apache-2.0

use crate::{environment::PackageSource, PythonEnvironment, PythonVersion};
use anyhow::{anyhow, Result};
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

// ============================================================================
// Platform Configuration - All platform differences centralized here
// ============================================================================

/// Platform-specific configuration for Python library discovery
#[derive(Debug, Clone)]
struct PlatformConfig {
    /// Subdirectory components from base_path to library directory
    /// Unix: ["sys", "lib"], Windows: ["lib"]
    lib_subdir: &'static [&'static str],
    /// Whether the lib directory contains a python3.X version subdirectory
    /// Unix: false (libs directly in sys/lib/), Windows: true (libs in lib/python3.X/)
    lib_has_version_subdir: bool,
    /// Prefix for Python library files (e.g., "libpython" or "python3")
    lib_prefix: &'static str,
    /// Extension for Python library files (e.g., ".so" or ".dll")
    lib_extension: &'static str,
    /// Generic library name to filter out in favor of versioned one
    generic_lib_name: &'static str,
}

#[cfg(unix)]
const PLATFORM_CONFIG: PlatformConfig = PlatformConfig {
    lib_subdir: &["sys", "lib"],
    lib_has_version_subdir: false,
    lib_prefix: "libpython",
    lib_extension: ".so",
    generic_lib_name: "libpython3.so",
};

#[cfg(windows)]
const PLATFORM_CONFIG: PlatformConfig = PlatformConfig {
    lib_subdir: &["lib"],
    lib_has_version_subdir: true,
    lib_prefix: "python3",
    lib_extension: ".dll",
    generic_lib_name: "python3.dll",
};

// ============================================================================
// Core Discovery Logic - Platform agnostic, testable
// ============================================================================

/// Check if a filename matches the Python library pattern for a given config
fn is_python_library_name(name: &str, config: &PlatformConfig) -> bool {
    name.starts_with(config.lib_prefix) && name.contains(config.lib_extension)
}

/// Filter library files to prefer versioned ones over generic
fn filter_python_libraries_with_config(files: &mut Vec<PathBuf>, config: &PlatformConfig) {
    // Remove generic library in favor of versioned one
    if files.len() > 1 {
        files.retain(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name != config.generic_lib_name)
                .unwrap_or(false)
        });
    }

    // On Unix, also prefer .so.X.Y over plain .so (e.g., libpython3.9.so.1.0 over libpython3.9.so)
    if config.lib_extension == ".so" && files.len() > 1 {
        files.retain(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| !name.ends_with(".so"))
                .unwrap_or(false)
        });
    }
}

/// Build library directory path from base path and config
fn get_lib_dir(base_path: &Path, config: &PlatformConfig) -> Result<PathBuf> {
    let base = config
        .lib_subdir
        .iter()
        .fold(base_path.to_path_buf(), |p, component| p.join(component));
    if config.lib_has_version_subdir {
        find_python_subdir(&base)
    } else {
        Ok(base)
    }
}

/// Find Python library in a directory using the given config
fn find_libpython_in_dir_with_config(
    lib_dir: &Path,
    config: &PlatformConfig,
) -> Result<(PathBuf, PathBuf)> {
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
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| is_python_library_name(name, config))
                .unwrap_or(false)
        })
        .collect();

    filter_python_libraries_with_config(&mut libpython_files, config);

    match libpython_files.len() {
        0 => Err(anyhow!(
            "No Python library file found in {}",
            lib_dir.display()
        )),
        1 => {
            let lib_path = libpython_files
                .into_iter()
                .next()
                .expect("exactly one element guaranteed by match arm");
            Ok((lib_dir.to_path_buf(), lib_path))
        }
        _ => Err(anyhow!(
            "Multiple Python library files found in {}, expected exactly one",
            lib_dir.display()
        )),
    }
}

// ============================================================================
// ISPM Package Discovery
// ============================================================================

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

// ============================================================================
// Public API
// ============================================================================

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
    // Unix: SIMICS_BASE/HOST_DIRNAME/sys/lib/libpython3.X.so.Y.Z
    // Windows: SIMICS_BASE/HOST_DIRNAME/bin/python3.X.dll
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
        1 => Ok(python_dirs
            .into_iter()
            .next()
            .expect("exactly one element guaranteed by match arm")),
        _ => Err(anyhow!(
            "Multiple python* subdirectories found in {}, expected exactly one",
            include_dir.display()
        )),
    }
}

/// Find Python library directory and specific library file
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

    let lib_dir = get_lib_dir(base_path, &PLATFORM_CONFIG)?;
    find_libpython_in_dir_with_config(&lib_dir, &PLATFORM_CONFIG)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // Test configs - can test BOTH platforms on ANY platform
    const UNIX_CONFIG: PlatformConfig = PlatformConfig {
        lib_subdir: &["sys", "lib"],
        lib_has_version_subdir: false,
        lib_prefix: "libpython",
        lib_extension: ".so",
        generic_lib_name: "libpython3.so",
    };

    const WINDOWS_CONFIG: PlatformConfig = PlatformConfig {
        lib_subdir: &["lib"],
        lib_has_version_subdir: true,
        lib_prefix: "python3",
        lib_extension: ".dll",
        generic_lib_name: "python3.dll",
    };

    // ========================================================================
    // Cross-platform unit tests for core logic
    // ========================================================================

    #[test]
    fn test_is_python_library_name_unix() {
        assert!(is_python_library_name("libpython3.9.so.1.0", &UNIX_CONFIG));
        assert!(is_python_library_name("libpython3.so", &UNIX_CONFIG));
        assert!(is_python_library_name("libpython3.9.so", &UNIX_CONFIG));
        assert!(!is_python_library_name("python3.9.dll", &UNIX_CONFIG));
        assert!(!is_python_library_name("libfoo.so", &UNIX_CONFIG));
    }

    #[test]
    fn test_is_python_library_name_windows() {
        assert!(is_python_library_name("python3.10.dll", &WINDOWS_CONFIG));
        assert!(is_python_library_name("python3.dll", &WINDOWS_CONFIG));
        assert!(!is_python_library_name("libpython3.9.so", &WINDOWS_CONFIG));
        assert!(!is_python_library_name("python3.10.lib", &WINDOWS_CONFIG));
        assert!(!is_python_library_name("python2.7.dll", &WINDOWS_CONFIG));
    }

    #[test]
    fn test_get_lib_dir_unix() -> Result<()> {
        let base = PathBuf::from("/opt/simics/linux64");
        let lib_dir = get_lib_dir(&base, &UNIX_CONFIG)?;
        assert_eq!(lib_dir, PathBuf::from("/opt/simics/linux64/sys/lib"));
        Ok(())
    }

    #[test]
    fn test_get_lib_dir_windows() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let lib_dir = temp_dir.path().join("lib").join("python3.10");
        fs::create_dir_all(&lib_dir)?;

        let result = get_lib_dir(temp_dir.path(), &WINDOWS_CONFIG)?;
        assert_eq!(result, lib_dir);
        Ok(())
    }

    #[test]
    fn test_filter_removes_generic_unix() {
        let mut files = vec![
            PathBuf::from("/lib/libpython3.so"),
            PathBuf::from("/lib/libpython3.9.so.1.0"),
        ];
        filter_python_libraries_with_config(&mut files, &UNIX_CONFIG);
        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("libpython3.9.so.1.0"));
    }

    #[test]
    fn test_filter_removes_generic_windows() {
        let mut files = vec![
            PathBuf::from("/bin/python3.dll"),
            PathBuf::from("/bin/python3.10.dll"),
        ];
        filter_python_libraries_with_config(&mut files, &WINDOWS_CONFIG);
        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("python3.10.dll"));
    }

    #[test]
    fn test_filter_prefers_versioned_so_unix() {
        let mut files = vec![
            PathBuf::from("/lib/libpython3.9.so"),
            PathBuf::from("/lib/libpython3.9.so.1.0"),
        ];
        filter_python_libraries_with_config(&mut files, &UNIX_CONFIG);
        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().ends_with(".so.1.0"));
    }

    #[test]
    fn test_filter_keeps_single_file() {
        let mut files = vec![PathBuf::from("/lib/libpython3.9.so.1.0")];
        filter_python_libraries_with_config(&mut files, &UNIX_CONFIG);
        assert_eq!(files.len(), 1);
    }

    // ========================================================================
    // Integration tests with mock filesystem
    // ========================================================================

    #[test]
    fn test_find_libpython_unix_structure() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let lib_dir = temp_dir.path().join("sys").join("lib");
        fs::create_dir_all(&lib_dir)?;
        fs::write(lib_dir.join("libpython3.9.so.1.0"), "")?;

        let base_path = temp_dir.path();
        let actual_lib_dir = get_lib_dir(base_path, &UNIX_CONFIG)?;
        let (found_dir, found_path) =
            find_libpython_in_dir_with_config(&actual_lib_dir, &UNIX_CONFIG)?;

        assert_eq!(found_dir, lib_dir);
        assert!(found_path.to_string_lossy().contains("libpython3.9.so.1.0"));
        Ok(())
    }

    #[test]
    fn test_find_libpython_windows_structure() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let bin_dir = temp_dir.path().join("lib").join("python3.10");
        fs::create_dir_all(&bin_dir)?;
        fs::write(bin_dir.join("python3.dll"), "")?;

        let base_path = temp_dir.path();
        let actual_lib_dir = get_lib_dir(base_path, &WINDOWS_CONFIG)?;
        let (found_dir, found_path) =
            find_libpython_in_dir_with_config(&actual_lib_dir, &WINDOWS_CONFIG)?;

        assert_eq!(found_dir, bin_dir);
        assert!(found_path.to_string_lossy().contains("python3.dll"));
        Ok(())
    }

    #[test]
    fn test_multiple_libraries_error_unix() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let lib_dir = temp_dir.path();
        fs::write(lib_dir.join("libpython3.9.so.1.0"), "")?;
        fs::write(lib_dir.join("libpython3.10.so.1.0"), "")?;

        let err = find_libpython_in_dir_with_config(lib_dir, &UNIX_CONFIG).unwrap_err();
        assert!(err.to_string().contains("Multiple Python library files"));
        Ok(())
    }

    #[test]
    fn test_multiple_libraries_error_windows() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let lib_dir = temp_dir.path();
        fs::write(lib_dir.join("python3.9.dll"), "")?;
        fs::write(lib_dir.join("python3.10.dll"), "")?;

        let err = find_libpython_in_dir_with_config(lib_dir, &WINDOWS_CONFIG).unwrap_err();
        assert!(err.to_string().contains("Multiple Python library files"));
        Ok(())
    }

    #[test]
    fn test_no_library_error() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let lib_dir = temp_dir.path();
        // Create an empty directory

        let err = find_libpython_in_dir_with_config(lib_dir, &UNIX_CONFIG).unwrap_err();
        assert!(err.to_string().contains("No Python library file found"));
        Ok(())
    }

    #[test]
    fn test_nonexistent_directory_error() {
        let lib_dir = PathBuf::from("/nonexistent/path");
        let err = find_libpython_in_dir_with_config(&lib_dir, &UNIX_CONFIG).unwrap_err();
        assert!(err.to_string().contains("does not exist"));
    }

    // ========================================================================
    // Full discovery tests (platform-specific due to PLATFORM_CONFIG)
    // ========================================================================

    fn create_mock_simics_structure(base_dir: &Path, python_version: &str) -> Result<()> {
        let host_dir = base_dir.join(HOST_DIRNAME);
        fs::create_dir_all(host_dir.join("bin"))?;
        fs::create_dir_all(
            host_dir
                .join("include")
                .join(format!("python{}", python_version)),
        )?;

        // Create mock mini-python
        let mini_python = if cfg!(windows) {
            "mini-python.exe"
        } else {
            "mini-python"
        };
        fs::write(host_dir.join("bin").join(mini_python), "")?;

        // Create platform-specific library structure
        #[cfg(unix)]
        {
            fs::create_dir_all(host_dir.join("sys").join("lib"))?;
            fs::write(
                host_dir
                    .join("sys")
                    .join("lib")
                    .join(format!("libpython{}.so", python_version)),
                "",
            )?;
        }

        #[cfg(windows)]
        {
            fs::write(
                host_dir
                    .join("bin")
                    .join(format!("python{}.dll", python_version)),
                "",
            )?;
        }

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
    fn test_detect_simics_major_version_from_base() -> Result<()> {
        let base = PathBuf::from("/opt/simics/simics-7.38.0");
        let major = detect_simics_major_version_from_base(&base)?;
        assert_eq!(major, 7);

        Ok(())
    }

    #[test]
    fn test_detect_simics_major_version_from_base_v6() -> Result<()> {
        let base = PathBuf::from("/opt/simics/simics-6.0.191");
        let major = detect_simics_major_version_from_base(&base)?;
        assert_eq!(major, 6);

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
