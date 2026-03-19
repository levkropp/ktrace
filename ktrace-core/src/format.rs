// SPDX-License-Identifier: MIT OR Apache-2.0 OR BSD-2-Clause
//! KTRX v1 wire format types.

/// Magic bytes at offset 0 of every `DumpHeader`.
pub const KTRX_MAGIC: [u8; 4] = *b"KTRX";
/// Current wire format version.
pub const KTRX_VERSION: u32 = 1;

/// 64-byte dump header written before the ring buffer data.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct DumpHeader {
    /// Always `KTRX`.
    pub magic: [u8; 4],
    /// Wire format version; currently 1.
    pub version: u32,
    /// Architecture performance-counter frequency in Hz.
    /// x86_64: invariant TSC.  ARM64: `CNTFRQ_EL0`.
    pub tsc_freq_hz: u64,
    /// Number of per-CPU ring buffers that follow.
    pub num_cpus: u32,
    /// Entries per ring buffer (must be a power of two).
    pub ring_size: u32,
    /// Bytes per ring entry; currently 32.
    pub entry_size: u32,
    /// Reserved; must be zero.
    pub flags: u32,
    /// Zero-padded reserved region.
    pub _reserved: [u8; 32],
}

const _: () = assert!(core::mem::size_of::<DumpHeader>() == 64);

/// A single 32-byte trace event — one slot in a per-CPU ring buffer.
#[derive(Clone, Copy)]
#[repr(C, align(32))]
pub struct TraceRecord {
    /// Raw architecture counter value at event time.
    pub tsc: u64,
    /// Packed bitfield: `[flags:8 | pid_idx:11 | cpu:3 | event_type:10]`
    pub header: u32,
    /// Event-specific payload (5 × 4 bytes = 20 bytes).
    pub data: [u32; 5],
}

const _: () = assert!(core::mem::size_of::<TraceRecord>() == 32);

impl TraceRecord {
    /// An all-zero record used to mark an empty ring slot.
    pub const ZERO: Self = Self { tsc: 0, header: 0, data: [0; 5] };

    /// Pack `header` from its components.
    #[inline(always)]
    pub fn pack_header(event_type: u16, cpu: u8, pid: u16, flags: u8) -> u32 {
        ((event_type as u32) & 0x3FF)
            | (((cpu as u32) & 0x7) << 10)
            | (((pid as u32) & 0x7FF) << 13)
            | ((flags as u32) << 24)
    }

    /// Extract `event_type` from a packed header.
    #[inline(always)]
    pub fn event_type(header: u32) -> u16 { (header & 0x3FF) as u16 }

    /// Extract `cpu` from a packed header.
    #[inline(always)]
    pub fn cpu(header: u32) -> u8 { ((header >> 10) & 0x7) as u8 }

    /// Extract `pid_idx` from a packed header.
    #[inline(always)]
    pub fn pid(header: u32) -> u16 { ((header >> 13) & 0x7FF) as u16 }
}

/// Well-known event type constants.
#[allow(non_upper_case_globals)]
pub mod EventType {
    pub const SYSCALL_ENTER:  u16 = 0;
    pub const SYSCALL_EXIT:   u16 = 1;
    pub const CTX_SWITCH:     u16 = 5;
    pub const PAGE_FAULT:     u16 = 10;
    pub const WAITQ_SLEEP:    u16 = 70;
    pub const WAITQ_WAKE:     u16 = 71;
    pub const NET_CONNECT:    u16 = 193;
    pub const NET_SEND:       u16 = 197;
    pub const NET_RECV:       u16 = 198;
    pub const NET_POLL:       u16 = 199;
    pub const NET_RX_PACKET:  u16 = 201;
    pub const NET_TX_PACKET:  u16 = 202;
    pub const NET_TCP_STATE:  u16 = 203;
    pub const NET_DNS_QUERY:  u16 = 204;
}
