// Copyright (C) 2024 Intel Corporation
// SPDX-License-Identifier: Apache-2.0

use anyhow::{anyhow, Result};
use std::path::Path;

/// Python version information parsed from include directory
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PythonVersion {
    /// Major version number (e.g., 3)
    pub major: u32,
    /// Minor version number (e.g., 9)
    pub minor: u32,
    /// Patch version number (e.g., 10)
    pub patch: u32,
}

impl PythonVersion {
    /// Create a new Python version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// Generate Py_LIMITED_API hex value for this Python version
    ///
    /// Format: 0xMMmmpppp where MM=major, mm=minor, pppp=patch*1000
    /// Example: Python 3.9.10 -> 0x03090000 (patch is truncated to 0 for API compatibility)
    pub fn py_limited_api_hex(&self) -> String {
        format!("0x{:02x}{:02x}0000", self.major, self.minor)
    }

    /// Parse Python version from include directory name
    ///
    /// Expects directory names like "python3.9" or "python3.9.10"
    pub fn parse_from_include_dir<P: AsRef<Path>>(include_dir: P) -> Result<Self> {
        let dir_name = include_dir
            .as_ref()
            .file_name()
            .ok_or_else(|| anyhow!("Failed to get include directory filename"))?
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert include directory name to string"))?;

        // Remove "python" prefix and parse version components
        let version_str = dir_name.strip_prefix("python").ok_or_else(|| {
            anyhow!(
                "Include directory name does not start with 'python': {}",
                dir_name
            )
        })?;

        let components: Result<Vec<u32>> = version_str
            .split('.')
            .map(|s| {
                s.parse::<u32>()
                    .map_err(|e| anyhow!("Failed to parse version component '{}': {}", s, e))
            })
            .collect();

        let components = components?;

        match components.len() {
            2 => Ok(Self::new(components[0], components[1], 0)),
            3 => Ok(Self::new(components[0], components[1], components[2])),
            _ => Err(anyhow!(
                "Invalid Python version format '{}', expected 'X.Y' or 'X.Y.Z'",
                version_str
            )),
        }
    }
}

impl std::fmt::Display for PythonVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.patch == 0 {
            write!(f, "{}.{}", self.major, self.minor)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_py_limited_api_hex() {
        let version = PythonVersion::new(3, 9, 10);
        assert_eq!(version.py_limited_api_hex(), "0x03090000");

        let version = PythonVersion::new(3, 8, 0);
        assert_eq!(version.py_limited_api_hex(), "0x03080000");
    }

    #[test]
    fn test_parse_from_include_dir() {
        // Test 2-component version
        let path = PathBuf::from("/some/path/python3.9");
        let version = PythonVersion::parse_from_include_dir(&path).unwrap();
        assert_eq!(version, PythonVersion::new(3, 9, 0));

        // Test 3-component version
        let path = PathBuf::from("/some/path/python3.9.10");
        let version = PythonVersion::parse_from_include_dir(&path).unwrap();
        assert_eq!(version, PythonVersion::new(3, 9, 10));
    }

    #[test]
    fn test_display() {
        let version = PythonVersion::new(3, 9, 0);
        assert_eq!(version.to_string(), "3.9");

        let version = PythonVersion::new(3, 9, 10);
        assert_eq!(version.to_string(), "3.9.10");
    }
}
