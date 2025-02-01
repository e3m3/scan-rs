#!/bin/bash
# Copyright 2025, Giordano Salvador
# SPDX-License-Identifier: BSD-3-Clause

ROOT_DIR="$(dirname $0)"
source "${ROOT_DIR}/setup-macos.sh"

cargo build --verbose ${build_mode}
cargo clippy --verbose ${build_mode}
cargo fmt --all -- --check
cargo test --verbose ${build_mode} -j 1 -- --nocapture
