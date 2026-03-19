// SPDX-License-Identifier: MIT OR Apache-2.0 OR BSD-2-Clause
//! x86\_64 transport: ISA debugcon via `outb` to I/O port 0xe9.
//!
//! QEMU flags:
//! ```text
//! -chardev file,id=ktrace,path=ktrace.bin
//! -device isa-debugcon,chardev=ktrace,iobase=0xe9
//! ```
//!
//! On KVM, each `outb` completes in ~200 ns → ~5 MB/s sustained throughput.
//! The ISA debugcon chardev buffers writes asynchronously; there is no
//! flow-control stall.

/// I/O port used by ISA debugcon.
const PORT: u16 = 0xe9;

/// Write one byte to the ISA debugcon port.
#[inline(always)]
#[allow(unsafe_code)]
pub fn write_byte(byte: u8) {
    unsafe {
        core::arch::asm!(
            "outb %al, %dx",
            in("al") byte,
            in("dx") PORT,
            options(nomem, nostack, preserves_flags, att_syntax),
        );
    }
}

/// Write a slice to the ISA debugcon port — one `outb` per byte.
#[inline(never)]
pub fn write_bytes(data: &[u8]) {
    for &b in data {
        write_byte(b);
    }
}

/// Write a `u32` in little-endian byte order.
#[inline(always)]
pub fn write_u32(val: u32) { write_bytes(&val.to_le_bytes()); }

/// Write a `u64` in little-endian byte order.
#[inline(always)]
pub fn write_u64(val: u64) { write_bytes(&val.to_le_bytes()); }
