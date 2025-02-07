// Copyright 2025, Giordano Salvador
// SPDX-License-Identifier: BSD-3-Clause

#![allow(static_mut_refs)]
#![allow(clippy::unused_unit)]

use std::cmp;
use std::fmt;
use std::marker::Send;
use std::mem::size_of;
use std::slice;
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::sync::Barrier;
use std::sync::Mutex;
use std::thread;

use support::align;
use support::copy;
use support::DoubleBufferMode;
use support::IAdd;
use support::IDisplay;
use support::IScan;

const NUM_PAGES: usize = 10;
const PAGE_SIZE: usize = 4096;
const BUFFER_LENGTH: usize = NUM_PAGES * PAGE_SIZE;
static mut BUF_A: [u8; BUFFER_LENGTH] = [0_u8; BUFFER_LENGTH];
static mut BUF_B: [u8; BUFFER_LENGTH] = [0_u8; BUFFER_LENGTH];
static mut WORKGROUP_BARRIER: Option<Arc<Barrier>> = None;
static mut WORKGROUP_STATUS: Option<Arc<Mutex<WorkStatus>>> = None;

#[derive(Clone, Copy)]
pub struct Scan {
    verbose: bool,
}

#[derive(Copy, Clone)]
pub struct WorkGroup<const N: usize> {
    verbose: bool,
    id: usize,
    n: usize,
    offset: usize,
    mode: DoubleBufferMode,
}

#[derive(Copy, Clone, Default, Eq, PartialEq)]
pub enum WorkStatus {
    #[default]
    NoWorkPresent,
    WorkPresent {
        offset: usize,
        mode: DoubleBufferMode,
    },
    Shutdown,
}

macro_rules! thread_body {
    (
        $Verbose:ident,
        $Id:ident,
        $NIn:ident,
        $ChAckReceived:ident,
        $ChAckCompleted:ident$(,)?
    ) => {
        if $Verbose {
            eprintln!("[{}] Starting worker thread", $Id);
        }
        let (buf_a, buf_b) = Scan::get_buffers::<T>($NIn, DoubleBufferMode::default());
        let workgroup_barrier = unsafe { WORKGROUP_BARRIER.clone().unwrap() };
        let workgroup_status_lock = unsafe { WORKGROUP_STATUS.clone().unwrap() };
        loop {
            let _ = workgroup_barrier.wait();
            let workgroup_status = *workgroup_status_lock.lock().unwrap();
            if $Verbose {
                eprintln!("[{}] WorkGroupStatus: {}", $Id, workgroup_status);
            }
            let (offset, mode) = match workgroup_status {
                WorkStatus::NoWorkPresent => continue,
                WorkStatus::WorkPresent { offset, mode } => (offset, mode),
                WorkStatus::Shutdown => break,
            };
            if $Verbose {
                eprintln!("[{}] Beginning work for phase {}", $Id, offset);
            }
            if $ChAckReceived.send(()).is_err() {
                eprintln!(
                    "[{}] Failed to signal to main thread beginning of work phase",
                    $Id
                );
                break;
            }
            WorkGroup::<N> {
                $Verbose,
                $Id,
                $NIn,
                offset,
                mode,
            }
            .process::<T>(buf_a, buf_b);
            if $Verbose {
                eprintln!("[{}] Completed work for phase {}", $Id, offset);
            }
            if $ChAckCompleted.send(()).is_err() {
                eprintln!(
                    "[{}] Failed to signal to main thread end of work phase",
                    $Id
                );
                break;
            }
        }
        if $Verbose {
            eprintln!("[{}] Shutting down", $Id);
        }
    };
}

impl Scan {
    fn get_buffers<'a, T>(n: usize, mode: DoubleBufferMode) -> (&'a mut [T], &'a mut [T]) {
        let buf_a_u8 =
            align::<u8, u64>(n, unsafe { BUF_A[..(2 * n * size_of::<T>())].as_mut_ptr() });
        let buf_b_u8 =
            align::<u8, u64>(n, unsafe { BUF_B[..(2 * n * size_of::<T>())].as_mut_ptr() });
        let buf_a = unsafe { slice::from_raw_parts_mut(buf_a_u8.as_mut_ptr() as *mut T, n) };
        let buf_b = unsafe { slice::from_raw_parts_mut(buf_b_u8.as_mut_ptr() as *mut T, n) };
        match mode {
            DoubleBufferMode::A => (&mut buf_a[..n], &mut buf_b[..n]),
            DoubleBufferMode::B => (&mut buf_b[..n], &mut buf_a[..n]),
        }
    }

    /// Implement the parallel CPU exclusive scan algorithm
    pub fn process<T, const N: usize>(
        &self,
        _def: T,
        v_in: &[T],
        v_out: &mut [T],
    ) -> Result<(), String>
    where
        T: Copy + IAdd + IDisplay + Send,
    {
        let n_in = v_in.len();
        let n_out = v_out.len();
        let n_buf = BUFFER_LENGTH / size_of::<T>();
        let n_chunks = usize::div_ceil(n_out, N);
        let d_end = (n_out as f32).log2().ceil() as usize;
        Self::check_args(n_in, n_out)?;
        if n_out > n_buf {
            return Err(format!(
                "Expected internal buffer length ({}) to be at least as large as input/output size ({})",
                n_buf, n_out,
            ));
        }
        let mut mode = DoubleBufferMode::default();
        let (buf_a, buf_b) = Scan::get_buffers::<T>(n_out, mode);
        copy(&v_in[..(n_out - 1)], &mut buf_a[1..n_out])?;
        copy(buf_a, buf_b)?;
        let (ch_ack_received_send, ch_ack_received_recv) = channel::<()>();
        let (ch_ack_completed_send, ch_ack_completed_recv) = channel::<()>();
        unsafe {
            WORKGROUP_BARRIER = Some(Arc::new(Barrier::new(n_chunks + 1)));
            WORKGROUP_STATUS = Some(Arc::new(Mutex::new(WorkStatus::NoWorkPresent)));
        }
        let workgroup_barrier = unsafe { WORKGROUP_BARRIER.clone().unwrap() };
        let workgroup_status = unsafe { WORKGROUP_STATUS.clone().unwrap() };
        let worker_pool: Vec<thread::JoinHandle<()>> = (0..n_chunks)
            .map(|j| {
                let verbose = self.verbose;
                let id = j;
                let n = n_out;
                let ch_ack_received = ch_ack_received_send.clone();
                let ch_ack_completed = ch_ack_completed_send.clone();
                thread::spawn(move || {
                    thread_body!(verbose, id, n, ch_ack_received, ch_ack_completed);
                })
            })
            .collect();
        for d in 0..d_end {
            if self.verbose {
                eprintln!("[_] Depth {}:", d);
            }
            let offset = 1 << d; // 2^d
            *workgroup_status.lock().unwrap() = WorkStatus::WorkPresent { offset, mode };
            workgroup_barrier.wait();
            if self.verbose {
                eprintln!("[_] Awaiting acknowledgements of work");
            }
            if (0..n_chunks).any(|_| ch_ack_received_recv.recv().is_err()) {
                return Err(format!("Failed work received phase for depth {}", d));
            }
            *workgroup_status.lock().unwrap() = WorkStatus::NoWorkPresent;
            if self.verbose {
                eprintln!("[_] Awaiting acknowledgements work has ended");
            }
            if (0..n_chunks).any(|_| ch_ack_completed_recv.recv().is_err()) {
                return Err(format!("Failed work completed phase for depth {}", d));
            }
            mode.swap();
        }
        if self.verbose {
            eprintln!("[_] Shutting down threads");
        }
        *workgroup_status.lock().unwrap() = WorkStatus::Shutdown;
        workgroup_barrier.wait();
        worker_pool.into_iter().enumerate().for_each(|(i, j)| {
            if j.join().is_err() {
                eprintln!("[_] Failed to join thread {}", i);
            }
        });
        match mode {
            DoubleBufferMode::A => copy(buf_a, v_out)?,
            DoubleBufferMode::B => copy(buf_b, v_out)?,
        }
        Ok(())
    }
}

impl IScan for Scan {
    fn new(verbose: bool) -> Self {
        Self { verbose }
    }
}

impl<const N: usize> WorkGroup<N> {
    pub fn process<T>(&self, buf_a: &mut [T], buf_b: &mut [T]) -> ()
    where
        T: Copy + IAdd + IDisplay + Send,
    {
        let k_begin = self.id * N;
        let k_end_clamp = cmp::min(self.n, k_begin + N);
        let (buf_a, buf_b) = match self.mode {
            DoubleBufferMode::A => (&buf_a[..k_end_clamp], &mut buf_b[..k_end_clamp]),
            DoubleBufferMode::B => (&buf_b[..k_end_clamp], &mut buf_a[..k_end_clamp]),
        };
        for k in k_begin..k_end_clamp {
            if k >= self.offset {
                let j = k - self.offset;
                let a = buf_a[j];
                let b = buf_a[k];
                if self.verbose {
                    eprintln!("[{}] *   ({},{},{}): {} + {}", self.id, k, j, k, a, b);
                }
                buf_b[k] = a + b;
            } else {
                let a = buf_a[k];
                if self.verbose {
                    eprintln!("[{}] *   ({},{}): {}", self.id, k, k, a);
                }
                buf_b[k] = a;
            }
        }
    }
}

impl fmt::Display for WorkStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                WorkStatus::NoWorkPresent => "NoWorkPresent".to_string(),
                WorkStatus::WorkPresent { offset, mode } => {
                    format!("WorkPresent {{ offset = {}, mode = {} }}", offset, mode)
                }
                WorkStatus::Shutdown => "Shutdown".to_string(),
            }
        )
    }
}
