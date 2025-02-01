// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

use crate::scan::Scan;
use crate::support::IAdd;
use crate::support::IDisplay;

impl Scan {
    /// Implement the sequential exclusive scan algorithm
    pub fn seq<T>(&self, def: T, v_in: &[T], v_out: &mut [T]) -> Result<(), String>
    where
        T: Copy + Eq + IAdd + IDisplay,
    {
        let n_in = v_in.len();
        let n_out = v_out.len();
        Self::check_args(n_in, n_out)?;
        v_out[0] = def;
        for k in 1..n_out {
            v_out[k] = v_in[k - 1] + v_out[k - 1];
        }
        Ok(())
    }
}
