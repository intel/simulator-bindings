# Simics Rust API

This repository contains Rust bindings and utilities for Intel Simics Simulator and the
Intel Simics Simulator's C API.

These crates can be used together to build modules which can be loaded into the Intel
Simics Simulator to add or modify functionality and model devices.

## Crates

- [cargo-simics-build](cargo-simics-build): `cargo build` wrapper for packaging modules
  into `.ispm` packages.
- [ispm-wrapper](ispm-wrapper): `ispm` wrapper for running package management commands.
- [simics](simics): High level (and idiomatic) bindings for the Intel Simics Simulator C
  API.
- [simics-api-sys](simics-api-sys): Low level auto-generated bindings for the Intel
  Simics Simulator C API.
- [simics-build-utils](simics-build-utils): Build utilities for Simics modules.
- [simics-macro](simics-macro): Proc-macros for Simics modules.
- [simics-package](simics-package): Packaging tools for `.ispm` packages.
- [simics-sign](simics-sign): Module signing tools for Intel Simics Simulator modules.
- [simics-test](simics-test): Test utilities for Intel Simics Simulator modules.

## Documentation

The crate documentation can be found at
[intel.github.io/simics-rs/crates](https://intel.github.io/simics-rs/crates).

The current public Intel Simics Simulator documentation can be found at
[intel.github.io/simics-rs/simics](https://intel.github.io/simics-rs/simics).