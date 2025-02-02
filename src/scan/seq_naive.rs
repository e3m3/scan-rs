// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

use crate::scan::Scan;
use crate::support;
use crate::support::IAdd;
use crate::support::IDisplay;

impl Scan {
    /// Implement the sequential naive parallel exclusive scan algorithm
    pub fn seq_naive<T>(&self, def: T, v_in: &[T], v_out: &mut [T]) -> Result<(), String>
    where
        T: Copy + Eq + IAdd + IDisplay,
    {
        let n_in = v_in.len();
        let n_out = v_out.len();
        let d_end = (n_out as f32).log2().ceil() as usize;
        Self::check_args(n_in, n_out)?;
        support::copy(&v_in[..(n_in - 1)], &mut v_out[1..n_out])?;
        v_out[0] = def;
        if self.verbose {
            eprintln!("Computing tree depth [0..{})", d_end);
            eprintln!("tmp: {:?}", v_out);
        }
        for d in 0..d_end {
            if self.verbose {
                eprintln!("Depth {}:", d);
            }
            let offset = 1 << d; // 2^d
            // NOTE: Loop in reverse due loop-carried dependencies
            for k in (1..n_out).rev() {
                if k >= offset {
                    let j = k - offset;
                    let a = v_out[j];
                    let b = v_out[k];
                    if self.verbose {
                        eprintln!("*   ({},{},{}): {} + {}", k, j, k, a, b);
                    }
                    v_out[k] = a + b;
                }
            }
            if self.verbose {
                eprintln!("tmp: {:?}", v_out);
            }
        }
        Ok(())
    }
}
