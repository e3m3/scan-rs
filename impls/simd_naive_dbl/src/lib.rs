// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

#![feature(portable_simd)]

use std::cmp;
use std::iter::FromIterator;
use std::ops::BitAnd;
use std::ops::Not;
use std::simd::cmp::SimdPartialOrd;
use std::simd::num::SimdInt;
use std::simd::LaneCount;
use std::simd::Mask;
use std::simd::MaskElement;
use std::simd::Simd;
use std::simd::SimdElement;
use std::simd::SupportedLaneCount;

use support::alloc_aligned;
use support::copy_simd;
use support::rotate_right_simd;
use support::DoubleBufferMode;
use support::IAdd;
use support::IDisplay;
use support::IScan;

#[derive(Clone, Copy)]
pub struct Scan {
    verbose: bool,
}

impl Scan {
    /// Implement the sequential Simd exclusive scan algorithm
    pub fn process<T, const N: usize>(
        &self,
        def: T,
        v_in: &[T],
        v_out: &mut [T],
    ) -> Result<(), String>
    where
        T: Copy + IAdd + IDisplay + SimdElement,
        T::Mask: IDisplay + MaskElement,
        Simd<T, N>: IAdd,
        LaneCount<N>: SupportedLaneCount,
    {
        let n_in = v_in.len();
        let n_out = v_out.len();
        let mut mode = DoubleBufferMode::default();
        Self::check_args(n_in, n_out)?;
        let (buf_a_slice, _backing_store_a) = alloc_aligned::<T, u64>(n_out, def);
        let (buf_b_slice, _backing_store_b) = alloc_aligned::<T, u64>(n_out, def);
        rotate_right_simd::<T, N>(n_out, def, v_in, buf_a_slice)?;
        buf_a_slice[0] = def;
        copy_simd::<T, N>(n_out, def, buf_a_slice, buf_b_slice)?;
        if self.verbose {
            eprintln!("tmp_a: {:?}", &buf_a_slice[..]);
            eprintln!("tmp_b: {:?}", &buf_b_slice[..]);
        }
        let n_chunks = usize::div_ceil(n_out, N);
        let d_end = (n_out as f32).log2().ceil() as usize;
        let simd_def = Simd::<T, N>::from_array([def; N]);
        for d in 0..d_end {
            if self.verbose {
                eprintln!("Depth {}:", d);
            }
            let (buf_a, buf_b) = match mode {
                DoubleBufferMode::A => (&mut buf_a_slice[..], &mut buf_b_slice[..]),
                DoubleBufferMode::B => (&mut buf_b_slice[..], &mut buf_a_slice[..]),
            };
            let offset = 1 << d; // 2^d
            let mut kk = 0;
            let mut kk_end = N;
            for _ in 0..n_chunks {
                let kk_end_clamp = cmp::min(n_out, kk_end);
                let simd_n = Simd::<isize, N>::splat(n_out as isize);
                let simd_offset = Simd::<usize, N>::splat(offset);
                let simd_k = Simd::<usize, N>::from_slice(&Vec::from_iter(kk..kk_end));
                let jj = (kk as isize) - (offset as isize);
                let jj_end = jj + N as isize;
                let simd_j = Simd::<isize, N>::from_slice(&Vec::from_iter(jj..jj_end));
                let mask_en_k =
                    Mask::<T::Mask, N>::from_array(simd_k.simd_ge(simd_offset).to_array());
                let mask_dis_k = Mask::<T::Mask, N>::from_array(mask_en_k.not().to_array());
                let mask_en_j = Mask::<T::Mask, N>::from_array(simd_j.simd_lt(simd_n).to_array());
                let mask_en_kj =
                    Mask::<T::Mask, N>::from_array(mask_en_k.bitand(mask_en_j).to_array());
                let simd_ld_k = Simd::<T, N>::load_or(&buf_a[kk..kk_end_clamp], simd_def);
                let simd_ld_j_true = Simd::<T, N>::gather_select(
                    &buf_a[0..n_out],
                    mask_en_kj.cast::<isize>(),
                    simd_j.cast::<usize>(),
                    simd_def,
                );
                let simd_add_kj_true = simd_ld_k.add(simd_ld_j_true);
                if self.verbose {
                    eprintln!("simd_n: {:?}", simd_n);
                    eprintln!("simd_offset: {:?}", simd_offset);
                    eprintln!("simd_k: {:?}", simd_k);
                    eprintln!("simd_j: {:?}", simd_j);
                    eprintln!("mask_en_k: {:?}", mask_en_k);
                    eprintln!("mask_dis_k: {:?}", mask_dis_k);
                    eprintln!("mask_en_j: {:?}", mask_en_j);
                    eprintln!("mask_en_kj: {:?}", mask_en_kj);
                    eprintln!("simd_ld_k: {:?}", simd_ld_k);
                    eprintln!("simd_ld_j_true: {:?}", simd_ld_j_true);
                    eprintln!("simd_add_kj_true: {:?}", simd_add_kj_true);
                }
                simd_ld_k.store_select(&mut buf_b[kk..kk_end_clamp], mask_dis_k);
                simd_add_kj_true.store_select(&mut buf_b[kk..kk_end_clamp], mask_en_kj);
                kk += N;
                kk_end += N;
            }
            if self.verbose {
                eprintln!("tmp_a: {:?}", &buf_a_slice[..]);
                eprintln!("tmp_b: {:?}", &buf_b_slice[..]);
            }
            mode.swap();
        }
        copy_simd::<T, N>(
            n_out,
            def,
            match mode {
                DoubleBufferMode::A => buf_a_slice,
                DoubleBufferMode::B => buf_b_slice,
            },
            v_out,
        )?;
        Ok(())
    }
}

impl IScan for Scan {
    fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}
