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

/// x86_64 transport — only compiled when targeting x86_64 *and* the feature is set.
#[cfg(all(feature = "transport-x86-64", target_arch = "x86_64"))]
pub mod x86_64;
/// ARM64 transport — only compiled when targeting AArch64 *and* the feature is set.
#[cfg(all(feature = "transport-arm64", target_arch = "aarch64"))]
pub mod arm64;

/// Write a single byte to the active transport.
///
/// Dispatches to the enabled transport at compile time.
/// Compiles to a no-op when no matching transport is available for the target.
#[inline(always)]
pub fn write_byte(_byte: u8) {
    #[cfg(all(feature = "transport-x86-64", target_arch = "x86_64"))]
    x86_64::write_byte(_byte);
    #[cfg(all(feature = "transport-arm64", target_arch = "aarch64"))]
    arm64::write_byte(_byte);
}

/// Write a byte slice to the active transport.
///
/// Batched where hardware allows (ARM64: one `SYS_WRITE` trap per slice;
/// x86_64: one `outb` per byte).
#[inline]
pub fn write_bytes(_data: &[u8]) {
    #[cfg(all(feature = "transport-x86-64", target_arch = "x86_64"))]
    x86_64::write_bytes(_data);
    #[cfg(all(feature = "transport-arm64", target_arch = "aarch64"))]
    arm64::write_bytes(_data);
}
