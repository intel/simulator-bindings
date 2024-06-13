#!/bin/bash

# Copyright (C) 2024 Intel Corporation
# SPDX-License-Identifier: Apache-2.0

set -e

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

cargo publish --token "${CRATES_IO_TOKEN}" --package ispm-wrapper
sleep 120
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-package
sleep 120
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-sign
sleep 120
cargo publish --token "${CRATES_IO_TOKEN}" --package cargo-simics-build
sleep 120
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-api-sys
sleep 120
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-build-utils
sleep 120
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-macro
sleep 120
cargo publish --token "${CRATES_IO_TOKEN}" --package simics
sleep 120
cargo publish --token "${CRATES_IO_TOKEN}" --package simics-test
