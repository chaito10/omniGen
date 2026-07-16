# Building from Source

## Prerequisites

- Rust 1.85+ (install via [rustup](https://rustup.rs/))
- Vulkan SDK (for GPU support)
- Git

## Build Commands

### Debug Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

### Optimized Release

```bash
RUSTFLAGS="-C target-cpu=native" cargo build --release
```

## Build Profiles

The release profile in `Cargo.toml` is optimized for:

- **LTO** - Link-time optimization (fat)
- **Single codegen unit** - Maximum optimization
- **Abort on panic** - Smaller binary
- **Strip symbols** - Smaller binary

## Platform-Specific Notes

### Windows

```powershell
cargo build --release
# Binary: target\release\omnigen.exe
```

### Linux

```bash
cargo build --release
# Binary: target/release/omnigen
```

### macOS

```bash
cargo build --release
# Binary: target/release/omnigen
```

## Cross-Compilation

```bash
# Linux target from Windows
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```
