# KTRX Wire Format — Version 1

**Magic:** `4B 54 52 58` (`KTRX`)
**Version:** 1
**Endianness:** little-endian throughout
**Stability:** stable — decoders must handle all v1 dumps

---

## Overview

A ktrace dump is a binary byte stream written to a QEMU chardev.  A single
dump consists of:

1. One **DumpHeader** (64 bytes)
2. One **ring buffer** per CPU, each `RING_SIZE × 32` bytes

Multiple dumps may be concatenated in the output file (e.g., a header dump at
boot plus a full dump on exit).  Decoders should process the *last* complete
dump (identified by the magic at offset 0 of each header).

---

## DumpHeader (64 bytes)

| Offset | Size | Type     | Field           | Notes                                  |
|--------|------|----------|-----------------|----------------------------------------|
| 0      | 4    | `[u8;4]` | `magic`         | Always `KTRX` (0x4B545258)            |
| 4      | 4    | `u32`    | `version`       | Format version; currently 1            |
| 8      | 8    | `u64`    | `tsc_freq_hz`   | TSC/counter frequency in Hz            |
| 16     | 4    | `u32`    | `num_cpus`      | Number of per-CPU ring buffers         |
| 20     | 4    | `u32`    | `ring_size`     | Entries per ring (power of two)        |
| 24     | 4    | `u32`    | `entry_size`    | Bytes per entry; currently 32          |
| 28     | 4    | `u32`    | `flags`         | Reserved; must be 0                    |
| 32     | 32   | `[u8;32]`| `_reserved`     | Zero-padded; reserved for future use   |

Total header size: 64 bytes.

After the header, `num_cpus` ring buffers follow in order (CPU 0 first).
Each ring buffer is `ring_size × entry_size` bytes.

---

## TraceRecord (32 bytes, `align(32)`)

Each ring entry is one `TraceRecord`:

| Offset | Size | Type      | Field    | Notes                                    |
|--------|------|-----------|----------|------------------------------------------|
| 0      | 8    | `u64`     | `tsc`    | Raw architecture counter (TSC on x86_64, `CNTVCT_EL0` on ARM64) |
| 8      | 4    | `u32`     | `header` | Packed bitfield — see below              |
| 12     | 20   | `[u32;5]` | `data`   | Event-specific payload                   |

### Header bitfield (32 bits)

```
[31..24] flags     (8 bits)   — reserved; currently 0
[23..13] pid_idx   (11 bits)  — low 11 bits of the process ID
[12..10] cpu       (3 bits)   — CPU index (0–7)
[ 9.. 0] event_type (10 bits) — event class; see Event Types below
```

A `tsc` value of 0 indicates an empty (unused) ring slot.

---

## Event Types

| Value | Name              | `data[0]`     | `data[1]` | `data[2]`       | `data[3]` | `data[4]` |
|-------|-------------------|---------------|-----------|-----------------|-----------|-----------|
| 0     | `SYSCALL_ENTER`   | syscall nr    | a1 lo     | a1 hi           | a2 lo     | a2 hi     |
| 1     | `SYSCALL_EXIT`    | syscall nr    | ret lo    | ret hi          | 0         | 0         |
| 5     | `CTX_SWITCH`      | prev pid      | next pid  | 0               | 0         | 0         |
| 10    | `PAGE_FAULT`      | addr lo       | addr hi   | error code      | 0         | 0         |
| 70    | `WAITQ_SLEEP`     | queue id      | 0         | 0               | 0         | 0         |
| 71    | `WAITQ_WAKE`      | queue id      | woken pid | 0               | 0         | 0         |
| 193   | `NET_CONNECT`     | remote ip     | port      | 0               | 0         | 0         |
| 197   | `NET_SEND`        | len           | 0         | 0               | 0         | 0         |
| 198   | `NET_RECV`        | len           | 0         | 0               | 0         | 0         |
| 199   | `NET_POLL`        | events        | 0         | 0               | 0         | 0         |
| 201   | `NET_RX_PACKET`   | len           | proto     | 0               | 0         | 0         |
| 202   | `NET_TX_PACKET`   | len           | proto     | 0               | 0         | 0         |
| 203   | `NET_TCP_STATE`   | old state     | new state | 0               | 0         | 0         |
| 204   | `NET_DNS_QUERY`   | query id      | 0         | 0               | 0         | 0         |

Unknown event types should be rendered as `UNKNOWN(N)` with hex data.

---

## Timestamp conversion

```
nanoseconds = (tsc_delta × 10^9) / tsc_freq_hz
```

For x86_64, `tsc_freq_hz` is the invariant TSC frequency calibrated at boot.
For ARM64, it is the `CNTFRQ_EL0` value (typically 62.5 MHz on QEMU virt).

---

## Multiple dumps

The decoder searches the byte stream for all occurrences of the magic bytes
`KTRX` and parses each as a complete dump.  If multiple valid dumps are found,
the *last* one is used (it contains the most complete ring buffer snapshot).
Earlier dumps (e.g., a boot-time header dump) are discarded.

---

## Example layout

```
Offset 0:    DumpHeader (64 bytes)
  magic    = 4B 54 52 58
  version  = 01 00 00 00
  tsc_freq = <8 bytes, little-endian Hz>
  num_cpus = 01 00 00 00
  ring_sz  = 00 20 00 00   (8192 entries)
  entry_sz = 20 00 00 00   (32 bytes)
  flags    = 00 00 00 00
  reserved = 00 ... 00

Offset 64:   CPU 0 ring buffer (8192 × 32 = 262144 bytes)
  [slot 0]  tsc header data[0..4]
  [slot 1]  ...
  ...
  [slot 8191] ...

Total:       64 + 262144 = 262208 bytes per dump
```
