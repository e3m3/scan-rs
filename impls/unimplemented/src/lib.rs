// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

#![allow(dead_code)]

use support::IScan;

#[derive(Clone, Copy)]
pub struct Scan {
    verbose: bool,
}

impl Scan {
    /// An unimplemented default
    pub fn process<T>(&self, _identity: T, _v_in: &[T], _v_out: &mut [T]) -> Result<(), String> {
        Err("Unimplemented".to_string())
    }
}

impl IScan for Scan {
    fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}
