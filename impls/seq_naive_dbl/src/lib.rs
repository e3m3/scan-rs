// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

use support::copy;
use support::DoubleBufferMode;
use support::IAdd;
use support::IDisplay;
use support::IScan;

#[derive(Clone, Copy)]
pub struct Scan {
    verbose: bool,
}

impl Scan {
    /// Implement the sequential naive parallel exclusive scan algorithm
    pub fn process<T>(&self, def: T, v_in: &[T], v_out: &mut [T]) -> Result<(), String>
    where
        T: Copy + Eq + IAdd + IDisplay,
    {
        let n_in = v_in.len();
        let n_out = v_out.len();
        let d_end = (n_out as f32).log2().ceil() as usize;
        let mut mode = DoubleBufferMode::default();
        Self::check_args(n_in, n_out)?;
        copy(&v_in[..(n_in - 1)], &mut v_out[1..n_out])?;
        v_out[0] = def;
        let mut v_out_tmp = v_out.to_vec();
        if self.verbose {
            eprintln!("tmp_a: {:?}", &v_out_tmp[0..n_out]);
            eprintln!("tmp_b: {:?}", &v_out[0..n_out]);
            eprintln!("Computing tree depth [0..{})", d_end);
        }
        for d in 0..d_end {
            if self.verbose {
                eprintln!("Depth {}:", d);
            }
            let (buf_a, buf_b) = match mode {
                DoubleBufferMode::A => (&mut v_out_tmp[0..n_out], &mut v_out[0..n_out]),
                DoubleBufferMode::B => (&mut v_out[0..n_out], &mut v_out_tmp[0..n_out]),
            };
            let offset = 1 << d; // 2^d
            for k in 1..n_out {
                if k >= offset {
                    let j = k - offset;
                    let a = buf_a[j];
                    let b = buf_a[k];
                    if self.verbose {
                        eprintln!("*   ({},{},{}): {} + {}", k, j, k, a, b);
                    }
                    buf_b[k] = a + b;
                } else {
                    let a = buf_a[k];
                    if self.verbose {
                        eprintln!("*   ({},{}): {}", k, k, a);
                    }
                    buf_b[k] = a;
                }
            }
            if self.verbose {
                eprintln!("tmp_a: {:?}", &buf_a[0..n_out]);
                eprintln!("tmp_b: {:?}", &buf_b[0..n_out]);
            }
            mode.swap();
        }
        if mode == DoubleBufferMode::A {
            copy(&v_out_tmp, v_out)?;
        }
        Ok(())
    }
}

impl IScan for Scan {
    fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}
