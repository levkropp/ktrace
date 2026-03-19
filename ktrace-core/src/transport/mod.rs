// SPDX-License-Identifier: MIT OR Apache-2.0 OR BSD-2-Clause
//! Architecture-specific write transports.
//!
//! Each transport provides the same two primitives:
//! - [`write_byte`]  — write a single byte (one hardware instruction)
//! - [`write_bytes`] — write a slice (batched where possible)
//!
//! Enable one transport feature in `Cargo.toml`:
//! ```toml
//! ktrace-core = { features = ["transport-arm64"] }
//! ```

#[cfg(feature = "transport-x86-64")]
pub mod x86_64;
#[cfg(feature = "transport-arm64")]
pub mod arm64;

/// Write a single byte to the active transport.
///
/// Dispatches to the enabled transport at compile time.
/// Compiles to nothing when no transport feature is enabled.
#[inline(always)]
pub fn write_byte(_byte: u8) {
    #[cfg(feature = "transport-x86-64")]
    x86_64::write_byte(_byte);
    #[cfg(feature = "transport-arm64")]
    arm64::write_byte(_byte);
}

/// Write a byte slice to the active transport.
///
/// Batched where the hardware allows (ARM64 uses a single `SYS_WRITE` trap
/// for the whole slice; x86_64 loops `outb`).
#[inline]
pub fn write_bytes(_data: &[u8]) {
    #[cfg(feature = "transport-x86-64")]
    x86_64::write_bytes(_data);
    #[cfg(feature = "transport-arm64")]
    arm64::write_bytes(_data);
}
