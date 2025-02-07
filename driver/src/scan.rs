// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

use std::fmt;
use std::simd::LaneCount;
use std::simd::MaskElement;
use std::simd::Simd;
use std::simd::SimdElement;
use std::simd::SupportedLaneCount;

use crate::exit::exit;
use crate::exit::ExitCode;

use bytemuck::Pod;
use support::IAdd;
use support::ICast;
use support::IDisplay;
use support::IScan;

#[repr(i8)]
#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd)]
pub enum ImplKind {
    Sequential,
    SequentialNaive,
    SequentialNaiveDoubleBuffer,
    SimdNaiveDoubleBuffer,
    ParallelCPUNaiveDoubleBuffer,
    ParallelGPUNaiveDoubleBuffer,
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
        match self {
            ImplKind::Sequential => {
                let scan_obj = seq::Scan::new(verbose);
                scan_obj.process::<T>(identity, v_in, v_out)
            }
            ImplKind::SequentialNaive => {
                let scan_obj = seq_naive::Scan::new(verbose);
                scan_obj.process::<T>(identity, v_in, v_out)
            }
            ImplKind::SequentialNaiveDoubleBuffer => {
                let scan_obj = seq_naive_dbl::Scan::new(verbose);
                scan_obj.process::<T>(identity, v_in, v_out)
            }
            _ => {
                let scan_obj = unimplemented::Scan::new(verbose);
                scan_obj.process::<T>(identity, v_in, v_out)
            }
        }
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
        match self {
            ImplKind::SimdNaiveDoubleBuffer => {
                let scan_obj = simd_naive_dbl::Scan::new(verbose);
                scan_obj.process::<T, N>(identity, v_in, v_out)
            }
            _ => {
                let scan_obj = simd_unimplemented::Scan::new(verbose);
                scan_obj.process::<T, N>(identity, v_in, v_out)
            }
        }
    }

    pub fn dispatch_parallel<T, const N: usize>(
        &self,
        identity: T,
        v_in: &[T],
        v_out: &mut [T],
        verbose: bool,
    ) -> Result<(), String>
    where
        T: Copy + Eq + IAdd + ICast<i32> + IDisplay + Ord + Pod + Send,
        i32: ICast<T>,
    {
        match self {
            ImplKind::ParallelCPUNaiveDoubleBuffer => {
                let scan_obj = par_cpu_naive_dbl::Scan::new(verbose);
                scan_obj.process::<T, N>(identity, v_in, v_out)
            }
            ImplKind::ParallelGPUNaiveDoubleBuffer => {
                let scan_obj = par_gpu_naive_dbl::Scan::new(verbose);
                scan_obj.process::<T, N>(identity, v_in, v_out)
            }
            _ => {
                let scan_obj = par_unimplemented::Scan::new(verbose);
                scan_obj.process::<T, N>(identity, v_in, v_out)
            }
        }
    }

    pub fn get_options_string() -> String {
        format!(
            "Implementations:\n{}\n{}\n{}\n{}\n{}\n{}",
            ImplKind::Sequential.to_option_string(),
            ImplKind::SequentialNaive.to_option_string(),
            ImplKind::SequentialNaiveDoubleBuffer.to_option_string(),
            ImplKind::SimdNaiveDoubleBuffer.to_option_string(),
            ImplKind::ParallelCPUNaiveDoubleBuffer.to_option_string(),
            ImplKind::ParallelGPUNaiveDoubleBuffer.to_option_string(),
        )
    }

    pub fn is_parallel(self) -> bool {
        matches!(
            self,
            ImplKind::ParallelCPUNaiveDoubleBuffer | ImplKind::ParallelGPUNaiveDoubleBuffer
        )
    }

    pub fn is_simd(self) -> bool {
        self == ImplKind::SimdNaiveDoubleBuffer
    }

    pub fn to_option_string(self) -> String {
        format!("*  {} => {}", self as i8, self)
    }
}

impl fmt::Display for ImplKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ImplKind::Sequential => "ScanSeq",
                ImplKind::SequentialNaive => "ScanSeqNaive",
                ImplKind::SequentialNaiveDoubleBuffer => "ScanSeqNaive2x",
                ImplKind::SimdNaiveDoubleBuffer => "ScanSimdNaive2x",
                ImplKind::ParallelCPUNaiveDoubleBuffer => "ScanParCPUNaive2x",
                ImplKind::ParallelGPUNaiveDoubleBuffer => "ScanParGPUNaive2x",
            }
        )
    }
}

impl From<i8> for ImplKind {
    fn from(n: i8) -> Self {
        match n {
            0 => ImplKind::Sequential,
            1 => ImplKind::SequentialNaive,
            2 => ImplKind::SequentialNaiveDoubleBuffer,
            3 => ImplKind::SimdNaiveDoubleBuffer,
            4 => ImplKind::ParallelCPUNaiveDoubleBuffer,
            5 => ImplKind::ParallelGPUNaiveDoubleBuffer,
            _ => {
                exit(
                    ExitCode::Error,
                    Some(&format!("Invalid implementation id: {}", n)),
                );
            }
        }
    }
}
