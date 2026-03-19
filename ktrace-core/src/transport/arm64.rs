// SPDX-License-Identifier: MIT OR Apache-2.0 OR BSD-2-Clause
//! ARM64 transport: ARM semihosting via `HLT #0xF000`.
//!
//! QEMU flags:
//! ```text
//! -chardev file,id=ktrace,path=ktrace.bin
//! -semihosting-config enable=on,target=native,chardev=ktrace
//! ```
//!
//! ## Protocol (AArch64 semihosting)
//!
//! ```text
//! x0 = operation number
//! x1 = parameter block pointer  (or value for SYS_WRITEC)
//! HLT #0xF000                   ← QEMU intercepts, performs op, returns
//! ```
//!
//! ## Operation mapping
//!
//! | Primitive      | Operation             | Description                         |
//! |---|---|---|
//! | `write_byte`   | `SYS_WRITEC` (0x03)   | One trap per byte, lowest latency   |
//! | `write_bytes`  | `SYS_WRITE`  (0x05)   | One trap per slice, bulk-optimised  |
//!
//! `SYS_WRITE` (used by `write_bytes`) passes a three-word parameter block:
//! `[file_handle: u64, data_ptr: u64, byte_count: u64]`.
//! File handle 2 (stderr) is routed to the ktrace chardev when
//! `-semihosting-config chardev=ktrace` is set.
//!
//! ## Performance
//!
//! On TCG, one semihosting trap is ~500 ns.  `write_bytes` issues a single
//! trap for the entire ring-buffer dump (typically 256 KB), giving comparable
//! effective throughput to ISA debugcon.

/// Semihosting op: write single byte to the debug terminal.
const SYS_WRITEC: usize = 0x03;
/// Semihosting op: write buffer to an open file handle.
const SYS_WRITE: usize = 0x05;
/// File handle for stderr — mapped to the ktrace chardev.
const STDERR: usize = 2;

/// Write one byte via `SYS_WRITEC` (one trap per call).
#[inline(always)]
#[allow(unsafe_code)]
pub fn write_byte(byte: u8) {
    unsafe {
        core::arch::asm!(
            "hlt #0xf000",
            in("x0")  SYS_WRITEC,
            in("x1")  &byte as *const u8,
            lateout("x0") _,   // semihosting may write return value into x0
            options(nostack),
        );
    }
}

/// Write a slice via `SYS_WRITE` — one trap for the entire buffer.
///
/// This is the primary path for ktrace ring-buffer dumps.  A 256 KB dump
/// completes in a single `HLT #0xF000` trap.
#[inline(never)]
#[allow(unsafe_code)]
pub fn write_bytes(data: &[u8]) {
    if data.is_empty() {
        return;
    }
    // Parameter block layout for SYS_WRITE (AArch64 = 64-bit words):
    //   [0] file handle (STDERR → ktrace chardev)
    //   [1] pointer to data
    //   [2] byte count
    let params: [usize; 3] = [STDERR, data.as_ptr() as usize, data.len()];
    unsafe {
        core::arch::asm!(
            "hlt #0xf000",
            in("x0")  SYS_WRITE,
            in("x1")  params.as_ptr(),
            lateout("x0") _,
            options(nostack, readonly),
        );
    }
}

/// Write a `u32` in little-endian byte order.
#[inline(always)]
pub fn write_u32(val: u32) {
    write_bytes(&val.to_le_bytes());
}

/// Write a `u64` in little-endian byte order.
#[inline(always)]
pub fn write_u64(val: u64) {
    write_bytes(&val.to_le_bytes());
}
