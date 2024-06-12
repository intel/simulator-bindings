# Environment Variables

There are several environment variables which can be used for various purposes.

## Build Flags

- SIMICS_BASE: Specify the directory containing the Simics base package to build against
e.g. `SIMICS_BASE=/home/me/simics/simics-6.0.195`
- SIMICS_BINDINGS_NOCLEAN: Specify that the bindings should not have unknown items stripped. This results in messier and larger bindings files, but can be used if targeting a version of Simics which the bindings have not yet been explicitly updated to target. Also speeds up compilation, so this can be used in `.cargo/config.toml` to speed up rust-analyzer runs.

```toml
[env]
SIMICS_BINDINGS_NOCLEAN = "1"
```

## Metadata Overrides

- SIMICS_PACKAGE_PACKAGE_NAME: Override the `package-name` field of the
`package.metadata.simics` `Cargo.toml` table
- SIMICS_PACKAGE_PACKAGE_NUMBER: Override the `package-number` field of the
`package.metadata.simics` `Cargo.toml` table
- SIMICS_PACKAGE_NAME: Override the `name` field of the 
`package.metadata.simics` `Cargo.toml` table
- SIMICS_PACKAGE_DESCRIPTION: Override the `description` field of the 
`package.metadata.simics` `Cargo.toml` table
- SIMICS_PACKAGE_HOST: Override the `host` field of the 
`package.metadata.simics` `Cargo.toml` table
- SIMICS_PACKAGE_VERSION: Override the `version` field of the 
`package.metadata.simics` `Cargo.toml` table
- SIMICS_PACKAGE_BUILD_ID: Override the `build-id` field of the 
`package.metadata.simics` `Cargo.toml` table
- SIMICS_PACKAGE_BUILD_ID_NAMESPACE: Override the `build-id-namespace` field of the 
`package.metadata.simics` `Cargo.toml` table
- SIMICS_PACKAGE_CONFIDENTIALITY: Override the `confidentiality` field of the 
`package.metadata.simics` `Cargo.toml` table
- SIMICS_PACKAGE_TYPE: Override the `type` field of the 
`package.metadata.simics` `Cargo.toml` table
- SIMICS_PACKAGE_DOC_TITLE: Override the `doc-title` field of the 
`package.metadata.simics` `Cargo.toml` table

## Testing Flags

- SIMICS_TEST_LOCAL_PACKAGES_ONLY: Do not download new package versions for test
environments and only copy locally installed packages instead. Tests which explicitly
depend on non-installed packages will fail.
- SIMICS_TEST_CLEANUP_EACH: Clean up after each test instead of leaving the package
installation directory. Use if the package versions used in the test change, but there
is no need to use if only the code under test has changed (and not the test environment
configuration).