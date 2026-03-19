// SPDX-License-Identifier: MIT OR Apache-2.0 OR BSD-2-Clause
//! **ktrace-core** — KTRX v1 wire format types and QEMU trace transports.
//!
//! `#![no_std]` — usable from any bare-metal kernel.
//!
//! # Wire format
//!
//! A ktrace dump is a flat binary stream:
//!
//! ```text
//! DumpHeader (64 bytes)
//! CPU 0 ring (ring_size × 32 bytes)
//! CPU 1 ring ...
//! ```
//!
//! See [`DumpHeader`] and [`TraceRecord`] for field layouts.
//!
//! # Transports
//!
//! Enable the appropriate feature for your build target and call
//! [`transport::write_bytes`] to stream trace data to QEMU:
//!
//! | Feature              | Architecture | QEMU device                                    |
//! |---|---|---|
//! | `transport-x86-64`   | x86\_64      | `-device isa-debugcon,chardev=ktrace`          |
//! | `transport-arm64`    | ARM64        | `-semihosting-config enable=on,...,chardev=ktrace` |
#![no_std]

pub mod format;
pub mod transport;

pub use format::{DumpHeader, EventType, TraceRecord, KTRX_MAGIC, KTRX_VERSION};
