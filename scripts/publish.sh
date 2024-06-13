#!/bin/bash

# Copyright (C) 2024 Intel Corporation
# SPDX-License-Identifier: Apache-2.0

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &>/dev/null && pwd)
SECRETS_FILE="${SCRIPT_DIR}/../.secrets"

if [ ! -f "${SECRETS_FILE}" ]; then
    echo "No file '${SECRETS_FILE}' found. Please create one. It must have the following keys:
            CRATES_IO_TOKEN"
    exit 1
fi

CRATES_IO_TOKEN=$(grep CRATES_IO_TOKEN "${SECRETS_FILE}" | awk -F'=' '{print $2}' | tr -d '[:space:]')

if [ -z "${CRATES_IO_TOKEN}" ]; then
    echo "No CRATES_IO_TOKEN found in '${SECRETS_FILE}'. Please add it."
    exit 1
fi

if ! command -v cargo &>/dev/null; then
    echo "cargo must be installed! Install at https://doc.rust-lang.org/cargo/getting-started/installation.html"
    exit 1
fi

<<<<<<< HEAD
cargo publish --token "${CRATES_IO_TOKEN}" --package ispm-wrapper
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-package
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-sign
cargo publish --token "${CRATES_IO_TOKEN}" --package cargo-simics-build
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-api-sys
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-build-utils
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-macro
cargo publish --token "${CRATES_IO_TOKEN}" --package simics
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-test
=======
cargo publish --token "${CRATES_IO_TOKEN}" --dry-run --package ispm-wrapper
cargo publish --token "${CRATES_IO_TOKEN}" --dry-run --package simics-package
cargo publish --token "${CRATES_IO_TOKEN}" --dry-run --package simics-sign
cargo publish --token "${CRATES_IO_TOKEN}" --dry-run --package cargo-simics-build
cargo publish --token "${CRATES_IO_TOKEN}" --dry-run --package simics-api-sys
cargo publish --token "${CRATES_IO_TOKEN}" --dry-run --package simics-build-utils
cargo publish --token "${CRATES_IO_TOKEN}" --dry-run --package simics-macro
cargo publish --token "${CRATES_IO_TOKEN}" --dry-run --package simics
cargo publish --token "${CRATES_IO_TOKEN}" --dry-run --package simics-test
>>>>>>> a609dc6 (Update crate files for publishing)
