// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

#![allow(dead_code)]
#![allow(clippy::unused_unit)]
#![feature(adt_const_params)]
#![feature(integer_sign_cast)]
#![feature(portable_simd)]
#![feature(step_trait)]
#![feature(trait_alias)]

use std::cmp;
use std::env;
use std::str::FromStr;

mod exit;
use exit::ExitCode;
use exit::exit;

mod scan;
mod support;

const USAGE: &str = "usage: scan <Impl:i8> <N:i64> [<x_0:i64> .. <x_{N-1}:i64>]";

type TInt = i64;

fn parse_vec<T>(str_vec: &[String], default: T) -> Result<Vec<T>, &str>
where
    T: Copy + Eq + FromStr,
{
    let res = str_vec
        .iter()
        .map(|s| s.parse::<T>().unwrap_or(default))
        .collect::<Vec<T>>();
    if res.contains(&default) {
        Err("Failed to parse integer(s) from arguments")
    } else {
        Ok(res)
    }
}

fn main() -> ! {
    let verbose = env::var("VERBOSE").is_ok();
    let args: Vec<String> = env::args().collect();
    let n_args = args.len() as isize;
    if n_args < 3 {
        exit(
            ExitCode::Error,
            Some(&format!(
                "{}\n{}",
                USAGE,
                scan::ImplKind::get_options_string()
            )),
        );
    }

    let impl_id: i8 = match args.get(1).unwrap_or(&"-1".to_string()).parse::<i8>() {
        Ok(n) => n,
        Err(m) => exit(ExitCode::Error, Some(&m.to_string())),
    };
    let impl_kind = scan::ImplKind::from(impl_id);

    if verbose {
        eprintln!("Selected implementation:\n{}", impl_kind.to_option_string());
    }

    let n: isize = match args.get(2).unwrap_or(&"-1".to_string()).parse::<isize>() {
        Ok(n) => n,
        Err(m) => exit(ExitCode::Error, Some(&m.to_string())),
    };

    if verbose {
        eprintln!("Found array length N = {}", n);
    }

    match n.cmp(&0) {
        cmp::Ordering::Less => exit(
            ExitCode::Error,
            Some(&format!("Expected positive array length ({})", n)),
        ),
        cmp::Ordering::Equal => exit(ExitCode::Ok, Some("Empty array (N=0)")),
        _ => (),
    }

    if n_args != (n + 3) {
        exit(
            ExitCode::Error,
            Some(&format!("Expected {} arguments after N: {}", n, USAGE)),
        );
    }
    let v: Vec<TInt> = match parse_vec(&args[3..(3 + (n as usize))], TInt::MIN) {
        Ok(v) => v,
        Err(m) => exit(ExitCode::Error, Some(m)),
    };

    if verbose {
        eprintln!("Found input vector: {:?}", v);
    }

    let (v_in, _backing_store_in) = support::alloc_aligned::<TInt, u64>(n as usize, 0);
    let (v_out, _backing_store_out) = support::alloc_aligned::<TInt, u64>(n as usize, 0);

    if let Err(m) = support::copy(&v, v_in) {
        exit(ExitCode::Error, Some(&m));
    };

    let result = if impl_kind.is_simd() {
        impl_kind.dispatch_simd::<TInt, 4>(0, v_in, v_out, verbose)
    } else {
        impl_kind.dispatch::<TInt>(0, v_in, v_out, verbose)
    };

    match result {
        Ok(()) => (),
        Err(m) => exit(ExitCode::Error, Some(&m)),
    };

    println!("in  : {:?}", v_in);
    println!("out : {:?}", v_out);

    exit(ExitCode::Ok, None);
}
