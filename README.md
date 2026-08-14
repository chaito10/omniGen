# OmniGen (Candle Studio)

![Version](https://img.shields.io/github/v/release/chaito10/OmniGen) ![License](https://img.shields.io/github/license/chaito10/OmniGen)

A production-quality, single-executable multimodal AI runtime written entirely in Rust.

## What is this?

OmniGen (Candle Studio) is a production-quality, single-executable multimodal AI runtime written in Rust. It runs text generation (Qwen3), image generation and transformation (FLUX.1 Schnell), video generation and transformation (LTX Video), and audio transcription and generation (Whisper) from one binary using Candle with Vulkan or CPU fallback. Pipelines are driven by a YAML DAG engine with automatic, resume-able model downloads from HuggingFace.

## Features

| Modality | Model | Format |
|---|---|---|
| Text Generation | Qwen3 | GGUF |
| Image Generation | FLUX.1 Schnell | SafeTensors |
| Image Transformation | FLUX.1 Schnell | SafeTensors |
| Video Generation | LTX Video Distilled | GGUF |
| Video Transformation | LTX Video Distilled | GGUF |
| Audio Transcription | Whisper Large v3 Turbo | SafeTensors |
| Audio Generation | Whisper-based | SafeTensors |

## Architecture

```
omnigen
├── Candle Backend (Vulkan / CPU fallback)
├── YAML Pipeline Engine (DAG executor)
├── Download Manager (HuggingFace direct)
├── Model Registry
└── Plugin System
```

## Installation

### Scoop (Windows)

```bash
scoop bucket add chaito10 https://github.com/chaito10/scoop-bucket
scoop install omnigen
```

### From Source

```bash
git clone https://github.com/user/OmniGen.git
cd OmniGen
cargo build --release
```

The binary will be at `target/release/omnigen`.

### Requirements

- Rust 1.85+ (2024 edition)
- Vulkan SDK (recommended) or CPU-only mode
- ~10GB disk space for models

## Usage

### Generate an Image

```bash
omnigen image --prompt "A futuristic cyberpunk city"
```

### Transform an Image

```bash
omnigen image --prompt "Anime style" --input cat.png
```

### Generate Video

```bash
omnigen video --prompt "Drone over mountains"
```

### Transcribe Audio

```bash
omnigen audio speech.wav
```

### Chat

```bash
omnigen chat
```

### Run a Pipeline

```bash
omnigen run workflow.yaml
```

### Model Management

```bash
omnigen models               # List models
omnigen models download      # Download all models
omnigen models update        # Update models
```

### Diagnostics

```bash
omnigen doctor               # System check
omnigen benchmark            # Performance benchmark
```

## Pipeline Example

```yaml
backend:
  device: auto

models:
  auto_download: true

pipeline:
  - prompt:
      text: "A futuristic cyberpunk city"

  - image:
      model: flux

output:
  image: output.png
```

## Configuration

Omnigen uses YAML-only configuration. Place a `config.yaml` in your working directory or use the defaults.

```yaml
backend:
  device: auto          # auto | cpu | cuda | vulkan

models:
  auto_download: true
  cache_dir: models/
  max_concurrent_downloads: 4

memory:
  auto_unload: true
  max_vram_usage: 0.8
```

## Model Management

Models are automatically downloaded from HuggingFace on first use. No `hf`, `git-lfs`, `git`, or Python required.

- Resume interrupted downloads
- SHA-256 checksum verification
- Progress bars for all downloads
- Offline mode support
- `HF_TOKEN` environment variable for gated models

Models are stored in the `models/` directory:

```
models/
├── qwen/
│   └── model.gguf
├── flux/
│   └── model.safetensors
├── ltx/
│   └── model.gguf
└── whisper/
    └── model.safetensors
```

## Project Structure

```
src/
├── main.rs           # Entry point
├── cli.rs            # CLI argument parsing
├── config.rs         # YAML configuration
├── backend.rs        # Candle backend management
├── device.rs         # Device selection (Vulkan/CPU)
├── model.rs          # Model registry and management
├── traits.rs         # Core trait definitions
├── graph.rs          # DAG graph for pipelines
├── pipeline.rs       # Pipeline execution engine
├── scheduler.rs      # Parallel scheduler
├── download.rs       # Download manager
├── cache.rs          # Tensor cache
├── memory.rs         # Memory management
├── models/
│   ├── qwen.rs       # Qwen3 text model
│   ├── flux.rs       # FLUX image model
│   ├── ltx.rs        # LTX Video model
│   └── whisper.rs    # Whisper audio model
├── nodes/
│   ├── prompt.rs     # Prompt node
│   ├── text.rs       # Text generation node
│   ├── image.rs      # Image generation node
│   ├── video.rs      # Video generation node
│   ├── audio.rs      # Audio processing node
│   └── save.rs       # Save output node
└── utils/
    └── mod.rs        # Utility functions
```

## Performance

- Zero-copy tensor operations where possible
- Memory-mapped GGUF loading
- Parallel DAG execution
- Lazy model loading
- Automatic VRAM management
- Streaming inference support

## License

MIT License. See [LICENSE](LICENSE) for details.
