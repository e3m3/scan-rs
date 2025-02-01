---

#  Copyright

Copyright 2025, Giordano Salvador
SPDX-License-Identifier: BSD-3-Clause

Author/Maintainer:  Giordano Salvador <73959795+e3m3@users.noreply.github.com>


#  Description

[![Fedora 41](https://github.com/e3m3/scan-rs/actions/workflows/fedora-41.yaml/badge.svg?event=workflow_dispatch)](https://github.com/e3m3/scan-rs/actions/workflows/fedora-41.yaml)

[![MacOS 14](https://github.com/e3m3/scan-rs/actions/workflows/macos-14.yaml/badge.svg?event=workflow_dispatch)](https://github.com/e3m3/scan-rs/actions/workflows/macos-14.yaml)

Implements the exclusive scan algorithm using various sequential and SIMD techniques.

##  Prerequisites

*   rust-2024

##  Setup

*   Native build:
    
    ```shell
    cargo build
    ```

*   Run test suite:

    ```shell
    cargo test -j1 -- --nocapture
    ```

*   Run an algorithm (e.g., Sequential Scan) on a input vector:

    ```shell
    cargo run -- 0 8 3 1 7 0 4 1 6 3
    ```

*   Run an algorithm (e.g., Sequential Scan) on a input vector with verbose output:

    ```shell
    VERBOSE= cargo run -- 0 8 3 1 7 0 4 1 6 3
    ```


#  References

[1]:    https://developer.nvidia.com/gpugems/gpugems3/part-vi-gpu-computing/chapter-39-parallel-prefix-sum-scan-cuda

1.  `https://developer.nvidia.com/gpugems/gpugems3/part-vi-gpu-computing/chapter-39-parallel-prefix-sum-scan-cuda`
