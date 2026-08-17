# secure-counter

A production-grade, bounds-checked persistent counter written in Rust — built to demonstrate secure binary packaging with full supply-chain auditability.

## Features

- **Bounds-checked operations** — rejects increments/decrements that would exceed ±1,000,000
- **Atomic state writes** — counter file is written via temp-then-rename, never half-written
- **Structured logging** — timestamped INFO/WARN/ERROR output via `tracing`
- **Custom error types** — clean, typed errors via `thiserror` (no panic-on-bad-input)
- **Supply-chain auditable** — built with `cargo-auditable`; dependency manifest embedded directly in the binary
- **Reproducible builds** — toolchain pinned via `rust-toolchain.toml`, deps locked via `Cargo.lock`
- **Release hardened** — LTO, `panic=abort`, `strip=true`, `codegen-units=1`

## Quick Start

```bash
# Build (release)
cargo build --release

# Or build with embedded audit manifest
cargo install cargo-auditable
cargo auditable build --release
```

## Usage

```bash
./secure-counter get               # print current value
./secure-counter increment         # +1
./secure-counter increment 10      # +10
./secure-counter decrement 5       # -5
./secure-counter reset             # back to 0

# Enable structured logs
RUST_LOG=info ./secure-counter increment
```

## Security Tooling

```bash
# Scan Cargo.lock against RustSec advisory database
cargo install cargo-audit
cargo audit

# Scan the compiled binary directly (no source needed)
cargo audit bin target/release/secure-counter

# Full supply-chain policy check (licences + advisories + bans)
cargo install cargo-deny
cargo deny check
```

## Release Profile

```toml
[profile.release]
opt-level = 3        # maximum optimisation
lto = true           # whole-program dead-code elimination
codegen-units = 1    # required for full LTO
panic = "abort"      # no unwinding — smaller binary, reduced attack surface
strip = true         # strip debug symbols from final binary
```

## Air-Gapped Cross-Compilation (Linux → Windows)

See [BUILD.md](BUILD.md) for step-by-step instructions to build a `x86_64-pc-windows-gnu` binary from an offline Linux machine.
