// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

use crate::scan::Scan;

impl Scan {
    /// An unimplemented default
    pub fn unimplemented<T>(
        &self,
        _identity: T,
        _v_in: &[T],
        _v_out: &mut [T],
    ) -> Result<(), String> {
        Err("Unimplemented".to_string())
    }
}
