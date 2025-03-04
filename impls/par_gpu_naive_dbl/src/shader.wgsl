// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

@group(0) @binding(0)
var<uniform> n: u32;
@group(0) @binding(1)
var<uniform> N: u32;
@group(0) @binding(2)
var<uniform> d_end: u32;
@group(0) @binding(3)
var<storage, read_write> mode: u32;
@group(0) @binding(4)
var<storage, read_write> input: array<i32>;
@group(0) @binding(5)
var<storage, read_write> output: array<i32>;

@compute @workgroup_size(64)
fn scan(
    @builtin(global_invocation_id) global_id: vec3<u32>
) {
    let idx = global_id.x;
    for (var d: u32 = 0; d < d_end; d++) {
        if (idx < n) {
            let offset = u32(1) << d;
            let k_begin = idx * N;
            let k_end_clamp = min(n, k_begin + N);
            for (var k: u32 = k_begin; k < k_end_clamp; k++) {
                if (mode == 0 && k >= offset) {
                    let j = k - offset;
                    let a = input[j];
                    let b = input[k];
                    output[k] = a + b;
                } else if (mode == 1 && k >= offset) {
                    let j = k - offset;
                    let a = output[j];
                    let b = output[k];
                    input[k] = a + b;
                } else if (mode == 0) {
                    let a = input[k];
                    output[k] = a;
                } else if (mode == 1) {
                    let a = output[k];
                    input[k] = a;
                }
            }
        }
        storageBarrier();
        if (idx == 0) {
            if (mode == 0) {
                mode = 1;
            } else if (mode == 1) {
                mode = 0;
            }
        }
    }
}
