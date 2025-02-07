// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

use std::process;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ExitCode {
    Ok = 0,
    Error = 1,
}

pub fn exit(code: ExitCode, message: Option<&str>) -> ! {
    if let Some(m) = message {
        eprintln!("{}", m);
    }
    process::exit(code as i32);
}
