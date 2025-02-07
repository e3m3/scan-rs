// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

#![allow(clippy::unused_unit)]
#![feature(adt_const_params)]
#![feature(portable_simd)]
#![feature(trait_alias)]

use std::cmp;
use std::fmt;
use std::iter::FromIterator;
use std::mem::align_of;
use std::ops;
use std::ops::Range;
use std::simd::LaneCount;
use std::simd::Mask;
use std::simd::MaskElement;
use std::simd::Simd;
use std::simd::SimdElement;
use std::simd::SupportedLaneCount;
use std::slice;

use bytemuck::Pod;
use bytemuck::Zeroable;

pub type Bitmask = u64;
pub type Predicate<T> = dyn Fn(T) -> bool;

pub trait IAdd = ops::Add<Self, Output = Self> + Sized;
pub trait IBAnd = ops::BitAnd<Self, Output = Self> + Sized;
pub trait IBOr = ops::BitOr<Self, Output = Self> + Sized;
pub trait IBXor = ops::BitXor<Self, Output = Self> + Sized;
pub trait IDisplay = fmt::Debug + fmt::Display;
pub trait IShl = ops::Shl<usize, Output = Self> + Sized;
pub trait ISlice = slice::SliceIndex<[Self], Output = Self> + Sized;

pub trait ICast<T> {
    fn cast(self) -> T;
}

pub trait ITop: PartialOrd + Sized {
    const TOP: Self;
}

pub trait IZero: Eq + PartialOrd + Sized {
    const ZERO: Self;
}

pub trait IScan {
    fn new(verbose: bool) -> Self;

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

#[repr(u8)]
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub enum DoubleBufferMode {
    #[default]
    A,
    B,
}

/// Returns an aligned slice
pub fn align<'a, T, U>(n: usize, p: *mut T) -> &'a mut [T] {
    let offset = p.align_offset(align_of::<U>());
    let buf = unsafe { slice::from_raw_parts_mut(p.wrapping_add(offset), n) };
    &mut buf[0..n]
}

/// Returns a vector of type `T` of size `n`.
pub fn alloc<T>(n: usize, def: T) -> Vec<T>
where
    T: Copy,
{
    (0..n).map(|_| def).collect::<Vec<T>>()
}

/// Returns aligned slice of type `T` of size `n` relative to sizeof type `U`,
/// and the backing vector of size `2*n`.
pub fn alloc_aligned<'a, T, U>(n: usize, def: T) -> (&'a mut [T], Vec<T>)
where
    T: Copy,
{
    let mut v = alloc(2 * n, def);
    let buf = align::<T, U>(n, v[..].as_mut_ptr().cast::<T>());
    (buf, v)
}

pub fn clamp<T>(n: usize, v_in: &[T], v_out: &mut [T], begin: T, end: T) -> Result<(), String>
where
    T: Copy + Ord,
{
    for (i, &x) in v_in[0..n].iter().enumerate() {
        v_out[i] = cmp::min(cmp::max(x, begin), end);
    }
    Ok(())
}

pub fn concat<T>(v_a: &[T], v_b: &[T], v_dst: &mut [T]) -> Result<(), String>
where
    T: Copy,
{
    let n_a = v_a.len();
    let n_b = v_b.len();
    let n_dst = v_dst.len();
    if n_a + n_b < n_dst {
        return Err(format!(
            "Expected destination buffer at least as large ({}) as source buffers ({} + {})",
            n_dst, n_a, n_b
        ));
    }
    v_dst[..n_a].copy_from_slice(&v_a[..n_a]);
    v_dst[n_a..(n_a + n_b)].copy_from_slice(&v_b[..n_b]);
    Ok(())
}

pub fn copy<T>(v_src: &[T], v_dst: &mut [T]) -> Result<(), String>
where
    T: Copy,
{
    let n_src = v_src.len();
    let n_dst = v_dst.len();
    v_dst[..n_dst].copy_from_slice(&v_src[..n_src]);
    Ok(())
}

pub fn copy_casted<T, U>(v_src: &[T], v_dst: &mut [U]) -> Result<(), String>
where
    T: Copy + ICast<U>,
    U: Copy,
{
    v_src
        .iter()
        .enumerate()
        .for_each(|(i, &x)| v_dst[i] = x.cast());
    Ok(())
}

/// Copy data elements from the source vector to the destination vector using SIMD parallelism.
/// The incoming data pointers `v_src` and `v_dst` are expected to be aligned.
/// Only elements lying at indices within the bounds of the source (`range_src`) and
/// destination (`range_dst`) ranges are copied from the source and to the destination buffers,
/// respectively.
pub fn copy_simd<T, const N: usize>(
    n: usize,
    def: T,
    v_src: &[T],
    v_dst: &mut [T],
) -> Result<(), String>
where
    T: Copy + SimdElement,
    LaneCount<N>: SupportedLaneCount,
{
    let n_chunks = n / N;
    let n_coalesced = n_chunks * N;
    let n_rem = n - n_coalesced;
    let simd_def = Simd::<T, N>::from_array([def; N]);
    let mut i = 0;
    let mut j = N;
    for _ in 0..n_chunks {
        let simd_ld = Simd::load_or(&v_src[i..j], simd_def);
        simd_ld.store_select(&mut v_dst[i..j], new_mask_all_on::<T::Mask, N>());
        i += N;
        j += N;
    }
    if n_rem > 0 {
        let simd_ld = Simd::load_or(&v_src[n_coalesced..n], simd_def);
        simd_ld.store_select(
            &mut v_dst[n_coalesced..n],
            new_mask::<T::Mask, N>(&vec![true; n_rem]),
        );
    }
    Ok(())
}

/// Copy data elements from the source vector to the destination vector using SIMD parallelism.
/// The incoming data pointers `v_src` and `v_dst` are expected to be aligned.
/// Only elements lying at indices within the bounds of the source (`range_src`) and
/// destination (`range_dst`) ranges are copied from the source and to the destination buffers,
/// respectively.
pub fn copy_in_range_simd_masked<T, const N: usize>(
    n: usize,
    def: T,
    v_src: &[T],
    v_dst: &mut [T],
    range_src: Range<usize>,
    range_dst: Range<usize>,
) -> Result<(), String>
where
    T: Copy + SimdElement,
    LaneCount<N>: SupportedLaneCount,
{
    let n_chunks = usize::div_ceil(n, N);
    let simd_def = Simd::<T, N>::from_array([def; N]);
    let step = Simd::<usize, N>::from_array([N; N]);
    let in_range_src = move |k| range_src.contains(&k);
    let in_range_dst = move |k| range_dst.contains(&k);
    let mut i = 0;
    let mut j = N;
    let mut v_ij = Simd::<usize, N>::from_slice(&Vec::from_iter(i..j));
    for _ in 0..n_chunks {
        let mask_src = new_mask_pred_simd::<usize, T::Mask, N>(&v_ij, &in_range_src);
        let mask_dst = new_mask_pred_simd::<usize, T::Mask, N>(&v_ij, &in_range_dst);
        let simd_ld = Simd::load_select(&v_src[i..j], mask_src, simd_def);
        simd_ld.store_select(&mut v_dst[i..j], mask_dst);
        v_ij += step;
        i += N;
        j += N;
    }
    Ok(())
}

/// Copy data elements from the source vector to the destination vector using SIMD parallelism.
/// The incoming data pointers `v_src` and `v_dst` are expected to be aligned.
/// Elements to be copied to the destination buffer at index `j` are specified by the indices
/// buffer.
/// The element at index `k` in the index buffer with value `i` corresponds to
/// a copy of the `i`-th value from the source buffer to the destination buffer at index `j`.
pub fn copy_swizzle_simd<T, const N: usize>(
    n: usize,
    def: T,
    v_src: &[T],
    v_dst: &mut [T],
    indices: &[usize],
) -> Result<(), String>
where
    T: Copy + SimdElement,
    LaneCount<N>: SupportedLaneCount,
{
    let n_chunks = n / N;
    let n_coalesced = n_chunks * N;
    let n_rem = n - n_coalesced;
    let mut i = 0;
    let mut j = N;
    for _ in 0..n_chunks {
        let simd_swz = swizzle_simd::<T, N>(def, &v_src[i..j], &indices[i..j]);
        simd_swz.store_select(&mut v_dst[i..j], new_mask_all_on::<T::Mask, N>());
        i += N;
        j += N;
    }
    if n_rem > 0 {
        let simd_swz = swizzle_simd::<T, N>(def, &v_src[n_coalesced..n], &indices[n_coalesced..n]);
        simd_swz.store_select(
            &mut v_dst[n_coalesced..n],
            new_mask::<T::Mask, N>(&vec![true; n_rem]),
        );
    }
    Ok(())
}

pub fn rotate_right<T>(n: usize, v_src: &[T], v_dst: &mut [T]) -> Result<(), String>
where
    T: Copy,
{
    v_dst[0] = v_src[n];
    v_dst[1..n].copy_from_slice(&v_src[0..(n - 1)]);
    Ok(())
}

pub fn rotate_right_simd<T, const N: usize>(
    n: usize,
    def: T,
    v_src: &[T],
    v_dst: &mut [T],
) -> Result<(), String>
where
    T: Copy + SimdElement,
    LaneCount<N>: SupportedLaneCount,
{
    let n_rem = n - n / N;
    let last = v_src[n - 1]; // TODO: Per depth?
    let simd_def = Simd::<T, N>::from_array([def; N]);
    let simd_def_idx = Simd::<usize, N>::from_array([usize::MAX; N]);
    copy_simd::<T, N>(n, def, v_src, v_dst)?;
    // Rotate phase: [a,b,c,d, w,x,y,z, i,j,k,0] -> [d,a,b,c, z,w,x,y, 0,i,j,0]
    let n_chunks = usize::div_ceil(n, N);
    let indices = Vec::from_iter(0..n);
    let mut i = 0;
    let mut j = N;
    for _ in 0..n_chunks {
        let j_ = cmp::min(n, j);
        let simd_idx = Simd::<usize, N>::load_or(&indices[i..j_], simd_def_idx);
        let simd_ld = Simd::<T, N>::gather_or(&v_dst[0..n], simd_idx, simd_def);
        let simd_rot = simd_ld.rotate_elements_right::<1>();
        simd_rot.store_select(&mut v_dst[i..j_], new_mask_all_on::<T::Mask, N>());
        i += N;
        j += N;
    }
    // Strided offset phase: [d,z,0] -> [0,a,b,c, d,w,x,y, z,i,j,0]
    let d_end_ = f32::ceil((n as f32).log(N as f32));
    let d_end = unsafe { d_end_.to_int_unchecked::<usize>() };
    for d in 1..d_end {
        let stride: usize = N.pow(d as u32);
        let indices = Vec::from_iter((0..n).step_by(stride));
        let n_idx = usize::div_ceil(n, stride);
        let n_chunks = usize::div_ceil(n_idx, N);
        let mut i = 0;
        let mut j = N;
        for _ in 0..n_chunks {
            let j_ = cmp::min(n_idx, j);
            let simd_idx = Simd::<usize, N>::load_or(&indices[i..j_], simd_def_idx);
            let simd_ld = Simd::<T, N>::gather_or(&v_dst[0..n], simd_idx, simd_def);
            let simd_rot = simd_ld.rotate_elements_right::<1>();
            simd_rot.scatter(&mut v_dst[0..n], simd_idx);
            i += N;
            j += N;
        }
    }
    // Cleanup phase: last -> [k,a,b,c, d,w,x,y, z,i,j,0]
    if n_rem > 0 {
        v_dst[0] = last;
    }
    Ok(())
}

pub fn swizzle<T, const N: usize>(def: T, v_src: &[T], indices: &[usize]) -> [T; N]
where
    T: Copy,
{
    let mut res = [def; N];
    for i in 0..N {
        let index = indices[i];
        res[i] = if (0..N).contains(&index) {
            v_src[index]
        } else {
            def
        };
    }
    res
}

pub fn swizzle_simd<T, const N: usize>(def: T, v_src: &[T], indices: &[usize]) -> Simd<T, N>
where
    T: Copy + SimdElement,
    LaneCount<N>: SupportedLaneCount,
{
    let res = [def; N];
    let simd_idx = Simd::load_or(&indices[0..N], Simd::<usize, N>::from_array([0_usize; N]));
    Simd::gather_or(&v_src[0..N], simd_idx, Simd::<T, N>::from_slice(&res))
}

macro_rules! SwizzleConst {
    () => {};
    (($N:literal, $FnName:ident)) => {
        pub fn $FnName<T, const IDX: [usize; $N]>(def: T, v_src: &[T]) -> [T; $N]
        where
            T: Copy + Default,
        {
            let mut res = [def; $N];
            for i in 0..$N {
                let index = IDX[i];
                res[i] = v_src[index];
            }
            res
        }
    };
    (($N:literal, $FnName:ident), $($tail:tt)*) => {
        SwizzleConst!(($N, $FnName));
        SwizzleConst!($($tail)*);
    };
}
SwizzleConst!(
    (2, swizzle_const_2),
    (4, swizzle_const_4),
    (8, swizzle_const_8),
    (16, swizzle_const_16),
    (32, swizzle_const_32),
    (64, swizzle_const_64),
);

macro_rules! SwizzleConstSimd {
    () => {};
    (($N:literal, $FnName:ident)) => {
        pub fn $FnName<T, const IDX: [usize; $N]>(def: T, v_src: &[T]) -> Simd::<T, $N>
        where
            T: Copy + SimdElement,
        {
            let res = [def; $N];
            let simd_idx = Simd::<usize, $N>::from_array(IDX);
            Simd::gather_or(&v_src[0..$N], simd_idx, Simd::<T, $N>::from_slice(&res))
        }
    };
    (($N:literal, $FnName:ident), $($tail:tt)*) => {
        SwizzleConstSimd!(($N, $FnName));
        SwizzleConstSimd!($($tail)*);
    };
}
SwizzleConstSimd!(
    (2, swizzle_const_simd_2),
    (4, swizzle_const_simd_4),
    (8, swizzle_const_simd_8),
    (16, swizzle_const_simd_16),
    (32, swizzle_const_simd_32),
    (64, swizzle_const_simd_64),
);

/// Generate a make from elements in `v`.
pub fn new_mask<U, const N: usize>(v: &[bool]) -> Mask<U, N>
where
    U: MaskElement,
    LaneCount<N>: SupportedLaneCount,
{
    let n = cmp::min(v.len(), N);
    let mut mask_array = [false; N];
    mask_array[0..n].copy_from_slice(&v[0..n]);
    Mask::<U, N>::from_array(mask_array)
}

/// Generate a make from elements in `v` using `pred`.
pub fn new_mask_pred<T, U, const N: usize>(v: &[T], pred: &Predicate<T>) -> Mask<U, N>
where
    T: Copy,
    U: MaskElement,
    LaneCount<N>: SupportedLaneCount,
    Bitmask: From<bool> + IBOr + IShl + IZero,
{
    let n = cmp::min(v.len(), N);
    let bitmask: Bitmask = v[0..n]
        .iter()
        .enumerate()
        .map(|(i, &x)| Bitmask::from(pred(x)) << i)
        .fold(Bitmask::ZERO, |acc, m| acc | m);
    Mask::<U, N>::from_bitmask(bitmask)
}

/// Generate a make from elements in `v` using `pred`.
pub fn new_mask_pred_simd<T, U, const N: usize>(v: &Simd<T, N>, pred: &Predicate<T>) -> Mask<U, N>
where
    T: Copy + SimdElement,
    U: MaskElement,
    LaneCount<N>: SupportedLaneCount,
    Bitmask: From<bool> + IBOr + IShl + IZero,
{
    let mut bitmask = Bitmask::ZERO;
    for i in 0..N {
        let x = v[i];
        bitmask |= Bitmask::from(pred(x)) << i;
    }
    Mask::<U, N>::from_bitmask(bitmask)
}

pub fn new_mask_all_off<U, const N: usize>() -> Mask<U, N>
where
    U: MaskElement,
    LaneCount<N>: SupportedLaneCount,
{
    Mask::<U, N>::splat(false)
}

pub fn new_mask_all_on<U, const N: usize>() -> Mask<U, N>
where
    U: MaskElement,
    LaneCount<N>: SupportedLaneCount,
{
    Mask::<U, N>::splat(true)
}

impl ICast<i32> for i64 {
    fn cast(self) -> i32 {
        self as i32
    }
}

impl ICast<i64> for i32 {
    fn cast(self) -> i64 {
        self as i64
    }
}

macro_rules! ImplTopInt {
    () => {};
    ($T:ty) => {
        impl ITop for $T {
            const TOP: $T = <$T>::MAX;
        }
    };
    ($T:ty, $($tail:tt)*) => {
        ImplTopInt!($T);
        ImplTopInt!($($tail)*);
    };
}
ImplTopInt!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

macro_rules! ImplZeroInt {
    () => {};
    ($T:ty) => {
        impl IZero for $T {
            const ZERO: $T = 0 as $T;
        }
    };
    ($T:ty, $($tail:tt)*) => {
        ImplZeroInt!($T);
        ImplZeroInt!($($tail)*);
    };
}
ImplZeroInt!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

impl DoubleBufferMode {
    pub fn get_alternate(self) -> Self {
        match self {
            DoubleBufferMode::A => DoubleBufferMode::B,
            DoubleBufferMode::B => DoubleBufferMode::A,
        }
    }

    pub fn swap(&mut self) -> () {
        *self = self.get_alternate();
    }
}

impl fmt::Display for DoubleBufferMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                DoubleBufferMode::A => "ModeA",
                DoubleBufferMode::B => "ModeB",
            }
        )
    }
}

unsafe impl Pod for DoubleBufferMode {}

unsafe impl Zeroable for DoubleBufferMode {}
