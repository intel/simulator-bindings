// Copyright (C) 2024 Intel Corporation
// SPDX-License-Identifier: Apache-2.0

use crate::version::PythonVersion;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Source of the Python environment
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageSource {
    /// Found in bundled Simics base package (1000)
    Bundled,
    /// Found in separate Simics Python package (1033), used in Simics 7.28.0+
    SeparatePackage,
}

/// Complete Python environment information for Simics
#[derive(Debug, Clone)]
pub struct PythonEnvironment {
    /// Path to mini-python executable
    pub mini_python: PathBuf,
    /// Path to Python include directory (contains python3.X subdirectory)
    pub include_dir: PathBuf,
    /// Include flag for C compilation (e.g., "-I/path/to/include/python3.9")
    pub include_flag: String,
    /// Directory containing libpython*.so files
    pub lib_dir: PathBuf,
    /// Full path to the specific libpython*.so file
    pub lib_path: PathBuf,
    /// Directory containing python3.lib import library (Windows).
    /// On Unix, this mirrors `lib_dir` and is unused.
    pub import_lib_dir: PathBuf,
    /// Parsed Python version information
    pub version: PythonVersion,
    /// Source package where Python was found
    pub package_source: PackageSource,
}

impl PythonEnvironment {
    /// Create a new Python environment
    pub fn new(
        mini_python: PathBuf,
        include_dir: PathBuf,
        lib_dir: PathBuf,
        lib_path: PathBuf,
        import_lib_dir: PathBuf,
        version: PythonVersion,
        package_source: PackageSource,
    ) -> Self {
        let include_flag = format!("-I{}", include_dir.display());

        Self {
            mini_python,
            include_dir,
            include_flag,
            lib_dir,
            lib_path,
            import_lib_dir,
            version,
            package_source,
        }
    }

    /// Set the package source for this environment
    pub fn with_source(mut self, source: PackageSource) -> Self {
        self.package_source = source;
        self
    }

    /// Validate that all required files and directories exist
    pub fn validate(&self) -> Result<()> {
        if !self.mini_python.exists() {
            return Err(anyhow!(
                "Mini-python executable not found: {}",
                self.mini_python.display()
            ));
        }

        if !self.mini_python.is_file() {
            return Err(anyhow!(
                "Mini-python path is not a file: {}",
                self.mini_python.display()
            ));
        }

        if !self.include_dir.exists() {
            return Err(anyhow!(
                "Python include directory not found: {}",
                self.include_dir.display()
            ));
        }

        if !self.include_dir.is_dir() {
            return Err(anyhow!(
                "Python include path is not a directory: {}",
                self.include_dir.display()
            ));
        }

        if !self.lib_dir.exists() {
            return Err(anyhow!(
                "Python library directory not found: {}",
                self.lib_dir.display()
            ));
        }

        if !self.lib_dir.is_dir() {
            return Err(anyhow!(
                "Python library path is not a directory: {}",
                self.lib_dir.display()
            ));
        }

        if !self.lib_path.exists() {
            return Err(anyhow!(
                "Python library file not found: {}",
                self.lib_path.display()
            ));
        }

        if !self.lib_path.is_file() {
            return Err(anyhow!(
                "Python library path is not a file: {}",
                self.lib_path.display()
            ));
        }

        #[cfg(windows)]
        {
            if !self.import_lib_dir.exists() {
                return Err(anyhow!(
                    "Python import library directory not found: {}",
                    self.import_lib_dir.display()
                ));
            }

            if !self.import_lib_dir.is_dir() {
                return Err(anyhow!(
                    "Python import library path is not a directory: {}",
                    self.import_lib_dir.display()
                ));
            }
        }

        Ok(())
    }

    /// Get the Python major version as string
    pub fn major_version_str(&self) -> String {
        self.version.major.to_string()
    }

    /// Get the Python minor version as string
    pub fn minor_version_str(&self) -> String {
        self.version.minor.to_string()
    }

    /// Get the Py_LIMITED_API define for C compilation
    pub fn py_limited_api_define(&self) -> String {
        format!("-DPy_LIMITED_API={}", self.version.py_limited_api_hex())
    }

    /// Get the path to the python3.lib import library (Windows)
    pub fn import_lib_path(&self) -> PathBuf {
        self.import_lib_dir.join("python3.lib")
    }

    /// Get the library file name (without directory)
    pub fn lib_filename(&self) -> Result<String> {
        self.lib_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                anyhow!(
                    "Failed to get library filename from {}",
                    self.lib_path.display()
                )
            })
    }
}

impl std::fmt::Display for PythonEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PythonEnvironment {{ version: {}, source: {:?}, mini_python: {}, include: {}, lib: {}, import_lib: {} }}",
            self.version,
            self.package_source,
            self.mini_python.display(),
            self.include_dir.display(),
            self.lib_path.display(),
            self.import_lib_dir.display()
        )
    }
}
