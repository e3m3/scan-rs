// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

use std::fmt;

use std::simd::LaneCount;
use std::simd::MaskElement;
use std::simd::Simd;
use std::simd::SimdElement;
use std::simd::SupportedLaneCount;

use crate::exit::ExitCode;
use crate::exit::exit;
use crate::support::IAdd;
use crate::support::IDisplay;

mod seq;
mod seq_naive;
mod seq_naive_dbl;
mod simd_naive_dbl;
mod simd_unimplemented;
mod unimplemented;

#[repr(i8)]
#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd)]
pub enum ImplKind {
    Sequential,
    SequentialNaive,
    SequentialNaiveDoubleBuffer,
    SimdNaiveDoubleBuffer,
    ParallelCPUNaive,
    ParallelCPUNaiveDoubleBuffer,
    ParallelGPUNaive,
    ParallelGPUNaiveDoubleBuffer,
}

#[derive(Clone, Copy)]
pub struct Scan {
    verbose: bool,
}

impl ImplKind {
    pub fn dispatch<T>(
        &self,
        identity: T,
        v_in: &[T],
        v_out: &mut [T],
        verbose: bool,
    ) -> Result<(), String>
    where
        T: Copy + Eq + IDisplay + IAdd,
    {
        let scan_obj = Scan::new(verbose);
        let impl_fn = match self {
            ImplKind::Sequential => Scan::seq::<T>,
            ImplKind::SequentialNaive => Scan::seq_naive::<T>,
            ImplKind::SequentialNaiveDoubleBuffer => Scan::seq_naive_dbl::<T>,
            ImplKind::ParallelCPUNaive => Scan::unimplemented::<T>,
            ImplKind::ParallelCPUNaiveDoubleBuffer => Scan::unimplemented::<T>,
            ImplKind::ParallelGPUNaive => Scan::unimplemented::<T>,
            ImplKind::ParallelGPUNaiveDoubleBuffer => Scan::unimplemented::<T>,
            _ => Scan::unimplemented::<T>,
        };
        impl_fn(&scan_obj, identity, v_in, v_out)
    }

    pub fn dispatch_simd<T, const N: usize>(
        &self,
        identity: T,
        v_in: &[T],
        v_out: &mut [T],
        verbose: bool,
    ) -> Result<(), String>
    where
        T: Copy + Eq + IAdd + IDisplay + SimdElement,
        T::Mask: IDisplay + MaskElement,
        Simd<T, N>: IAdd,
        LaneCount<N>: SupportedLaneCount,
    {
        let scan_obj = Scan::new(verbose);
        let impl_fn = match self {
            ImplKind::SimdNaiveDoubleBuffer => Scan::simd_naive_dbl::<T, N>,
            _ => Scan::simd_unimplemented::<T, N>,
        };
        impl_fn(&scan_obj, identity, v_in, v_out)
    }

    pub fn get_options_string() -> String {
        format!(
            "Implementations:\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            ImplKind::Sequential.to_option_string(),
            ImplKind::SequentialNaive.to_option_string(),
            ImplKind::SequentialNaiveDoubleBuffer.to_option_string(),
            ImplKind::SimdNaiveDoubleBuffer.to_option_string(),
            ImplKind::ParallelCPUNaive.to_option_string(),
            ImplKind::ParallelCPUNaiveDoubleBuffer.to_option_string(),
            ImplKind::ParallelGPUNaive.to_option_string(),
            ImplKind::ParallelGPUNaiveDoubleBuffer.to_option_string(),
        )
    }

    pub fn is_simd(self) -> bool {
        self == ImplKind::SimdNaiveDoubleBuffer
    }

    pub fn to_option_string(self) -> String {
        format!("*  {} => {}", self as i8, self)
    }
}

/// NOTE: See `src/scan/` for implementations.
impl Scan {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    fn check_args(n_in: usize, n_out: usize) -> Result<(), String> {
        if n_in != n_out {
            Err(format!(
                "Expected output vector of length {} for input length {}",
                n_out, n_in
            ))
        } else {
            Ok(())
        }
    }
}

impl fmt::Display for ImplKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            ImplKind::Sequential => "ScanSeq",
            ImplKind::SequentialNaive => "ScanSeqNaive",
            ImplKind::SequentialNaiveDoubleBuffer => "ScanSeqNaive2x",
            ImplKind::SimdNaiveDoubleBuffer => "ScanSimdNaive2x",
            ImplKind::ParallelCPUNaive => "ScanParCPUNaive",
            ImplKind::ParallelCPUNaiveDoubleBuffer => "ScanParCPUNaive2x",
            ImplKind::ParallelGPUNaive => "ScanParGPUNaive",
            ImplKind::ParallelGPUNaiveDoubleBuffer => "ScanParGPUNaive2x",
        })
    }
}

impl From<i8> for ImplKind {
    fn from(n: i8) -> Self {
        match n {
            0 => ImplKind::Sequential,
            1 => ImplKind::SequentialNaive,
            2 => ImplKind::SequentialNaiveDoubleBuffer,
            3 => ImplKind::SimdNaiveDoubleBuffer,
            4 => ImplKind::ParallelCPUNaive,
            5 => ImplKind::ParallelCPUNaiveDoubleBuffer,
            6 => ImplKind::ParallelGPUNaive,
            7 => ImplKind::ParallelGPUNaiveDoubleBuffer,
            _ => {
                exit(
                    ExitCode::Error,
                    Some(&format!("Invalid implementation id: {}", n)),
                );
            }
        }
    }
}

impl Default for Scan {
    fn default() -> Self {
        Self::new(false)
    }
}
