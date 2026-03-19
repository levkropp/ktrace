# ktrace — Multi-Architecture Kernel Trace Protocol

[![CI](https://github.com/levkropp/ktrace/actions/workflows/ci.yml/badge.svg)](https://github.com/levkropp/ktrace/actions/workflows/ci.yml)
[![license: MIT OR Apache-2.0 OR BSD-2-Clause](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0%20OR%20BSD--2--Clause-blue)](#license)

**ktrace** is a minimal, high-bandwidth binary kernel tracing system designed
for bare-metal kernels running under QEMU.  It provides:

- A well-specified, platform-agnostic 32-byte binary event format (`KTRX v1`)
- Architecture-specific write transports that integrate with QEMU's chardev
  infrastructure — no custom QEMU patches required
- A host-side decoder that emits text timelines and
  [Perfetto](https://ui.perfetto.dev) JSON

## Why ktrace exists

Most kernel tracers (LTTng, Ftrace, perf) depend on a running Linux kernel.
ktrace was designed for kernels that are *replacing* Linux — it needs to work
from bare metal, before any userspace exists, and output through whatever
low-level channel QEMU provides.

The key insight: QEMU already has two excellent zero-overhead trace output
mechanisms that require no kernel-side driver beyond a single instruction:

| Architecture | Transport                | QEMU device                                 |
|---|---|---|
| x86\_64      | ISA debugcon `outb 0xe9` | `-device isa-debugcon,chardev=ktrace`       |
| ARM64        | ARM semihosting `SYS_WRITE` via `HLT #0xF000` | `-semihosting-config enable=on,target=native,chardev=ktrace` |

Both transports connect to the same QEMU chardev, so `ktrace.bin` has the
same format regardless of architecture.

## QEMU setup

### x86\_64
```sh
-chardev file,id=ktrace,path=ktrace.bin \
-device isa-debugcon,chardev=ktrace,iobase=0xe9
```

### ARM64
```sh
-chardev file,id=ktrace,path=ktrace.bin \
-semihosting-config enable=on,target=native,chardev=ktrace
```

Then add `debug=ktrace` to the kernel command line to enable recording.

## Decoding

```sh
python3 decode/ktrace-decode.py ktrace.bin          # text timeline
python3 decode/ktrace-decode.py ktrace.bin --summary # statistics
python3 decode/ktrace-decode.py ktrace.bin --perfetto trace.json
# open trace.json at https://ui.perfetto.dev
```

## Wire format

See [`spec/wire-format.md`](spec/wire-format.md) for the full specification.

Brief overview:
- 64-byte `DumpHeader` (magic `KTRX`, version, TSC frequency, CPU count)
- N × 32-byte `TraceRecord` (timestamp, packed header, 20-byte payload)
- Dumps can be concatenated; the decoder uses the last complete dump

## Kernel integration (`ktrace-core`)

The `ktrace-core` crate is a `no_std` Rust library that provides the wire
format types and architecture-specific write primitives.  Any bare-metal
kernel can `use ktrace_core::transport::write_bytes` to emit trace data.

```toml
[dependencies]
ktrace-core = { path = "tools/ktrace/ktrace-core", features = ["transport-arm64"] }
```

## Repository layout

```
ktrace/
├── spec/wire-format.md      # Binary protocol specification (KTRX v1)
├── ktrace-core/             # no_std Rust crate: format types + transports
│   └── src/
│       ├── lib.rs           # DumpHeader, TraceRecord, EventType
│       └── transport/
│           ├── x86_64.rs    # ISA debugcon (outb 0xe9)
│           └── arm64.rs     # ARM semihosting (HLT #0xF000, SYS_WRITE)
└── decode/
    └── ktrace-decode.py     # Host-side binary decoder
```

## Install / PATH setup

### Clone and use as a path dependency

```sh
git clone https://github.com/levkropp/ktrace.git
```

Add `ktrace-core` to your kernel's `Cargo.toml` using a relative path:

```toml
[dependencies]
# x86_64 bare-metal kernel
ktrace-core = { path = "../ktrace/ktrace-core", features = ["transport-x86-64"] }

# AArch64 bare-metal kernel
ktrace-core = { path = "../ktrace/ktrace-core", features = ["transport-arm64"] }
```

### Decoder script on your PATH

The host-side decoder is a single Python script.  To make it available system-wide:

```sh
# Option 1 — symlink into a directory already on PATH
ln -s "$PWD/decode/ktrace-decode.py" ~/.local/bin/ktrace-decode
chmod +x decode/ktrace-decode.py

# Option 2 — add decode/ to PATH in your shell profile
echo 'export PATH="$HOME/path/to/ktrace/decode:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Then run anywhere:
ktrace-decode ktrace.bin
ktrace-decode ktrace.bin --summary
ktrace-decode ktrace.bin --perfetto trace.json
```

### Cross-compile prerequisites (for bare-metal targets)

```sh
# Nightly Rust + bare-metal targets
rustup toolchain install nightly
rustup target add x86_64-unknown-none --toolchain nightly
rustup target add aarch64-unknown-none --toolchain nightly

# Build for x86_64-unknown-none
cargo +nightly build -p ktrace-core \
  --target x86_64-unknown-none \
  --features transport-x86-64 \
  -Z build-std=core
```

## License

MIT OR Apache-2.0 OR BSD-2-Clause
