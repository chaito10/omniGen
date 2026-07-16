# Installation

## From Source

### Prerequisites

- Rust 1.85+ (2024 edition)
- Vulkan SDK (recommended)
- Git

### Build

```bash
git clone https://github.com/user/OmniGen.git
cd OmniGen
cargo build --release
```

The binary will be at `target/release/omnigen`.

### Install Locally

```bash
cargo install --path .
```

## Requirements

| Requirement | Details |
|---|---|
| **Rust** | 1.85+ (2024 edition) |
| **Vulkan** | Recommended, CPU fallback available |
| **Disk Space** | ~10GB for all models |
| **RAM** | 8GB minimum, 16GB recommended |

## Platform Support

| Platform | Status |
|---|---|
| Windows | Supported |
| Linux | Supported |
| macOS | Supported (Metal backend) |

## Post-Installation

Verify your installation:

```bash
omnigen --version
omnigen doctor
```

Models are automatically downloaded on first use. No manual setup required.
