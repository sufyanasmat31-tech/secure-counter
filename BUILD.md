# Build Guide — secure-counter

## Standard build (online Linux machine)

```bash
# 1. Install Rust (if not present)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 2. Install supply-chain tools
cargo install cargo-auditable cargo-audit cargo-deny

# 3. Build a normal release binary (Linux)
cargo auditable build --release
# Binary: target/release/secure-counter

# 4. Run security checks
cargo audit                  # scan Cargo.lock against RustSec advisory DB
cargo deny check             # licence + ban + advisory policy (deny.toml)

# 5. Verify the audit manifest is embedded
cargo audit bin target/release/secure-counter
```

---

## Air-gapped cross-compilation — Windows x86_64 target

An air-gapped machine has **no internet access** during the build.
All dependencies must be fetched **before** disconnecting from the network.

### Step 1 — Pre-fetch everything (while online)

```bash
# Add the Windows cross-compile target
rustup target add x86_64-pc-windows-gnu

# Install the MinGW-w64 linker (Debian/Ubuntu)
sudo apt-get install -y gcc-mingw-w64-x86-64

# Download all crate sources into the local Cargo cache
cargo fetch

# Optional: vendor crates into ./vendor/ for fully offline builds
# (skips the Cargo cache entirely — useful for air-gapped Docker images)
cargo vendor
# If you vendor, append this to .cargo/config.toml:
#
# [source.crates-io]
# replace-with = "vendored-sources"
#
# [source."vendored-sources"]
# directory = "vendor"

# Pre-download the auditable/audit/deny binaries too
cargo install cargo-auditable cargo-audit cargo-deny
```

### Step 2 — Disconnect network (air-gap the machine)

### Step 3 — Cross-compile for Windows (offline)

```bash
# Tell the linker which GCC front-end to use for the Windows target
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc

cargo auditable build \
    --release \
    --target x86_64-pc-windows-gnu \
    --offline          # <-- explicit: fail instead of reaching out to network

# Output binary
ls -lh target/x86_64-pc-windows-gnu/release/secure-counter.exe
```

### Step 4 — Verify the embedded audit manifest

```bash
# Confirm dependency metadata is baked into the .exe
cargo audit bin target/x86_64-pc-windows-gnu/release/secure-counter.exe
```

---

## Reproducible builds

| Guarantee | Mechanism |
|-----------|-----------|
| Exact toolchain version | `rust-toolchain.toml` — rustup reads this automatically |
| Exact dependency versions | `Cargo.lock` committed to git |
| Licence + advisory policy | `deny.toml` + `cargo deny check` in CI |
| Supply-chain audit at deploy | `cargo audit bin <binary>` post-build |

---

## Quick usage reference

```bash
./secure-counter get               # print current value
./secure-counter increment         # +1
./secure-counter increment 10      # +10
./secure-counter decrement 5       # -5
./secure-counter reset             # back to 0

# Enable structured logs (sent to stderr)
RUST_LOG=info ./secure-counter increment
```
