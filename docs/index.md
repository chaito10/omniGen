# OmniGen

**Production-quality, single-executable multimodal AI runtime.**

OmniGen is a self-contained multimodal AI runtime written entirely in Rust. It supports text generation, image generation, video generation, and audio transcription using [Candle](https://github.com/huggingface/candle) as the compute backend.

## Features

- **Text Generation** - Qwen3 (GGUF format)
- **Image Generation** - FLUX.1 Schnell (SafeTensors)
- **Video Generation** - LTX Video Distilled (GGUF)
- **Audio Transcription** - Whisper Large v3 Turbo (SafeTensors)
- **Vulkan GPU acceleration** with CPU fallback
- **YAML-only pipeline engine** with DAG execution
- **Automatic model download** from HuggingFace
- **Single executable** - no Python, Docker, or external dependencies

## Quick Start

```bash
# Generate an image
omnigen image --prompt "A futuristic cyberpunk city"

# Transcribe audio
omnigen audio speech.wav

# Run a pipeline
omnigen run workflow.yaml
```

## Links

- [Installation Guide](getting-started/installation.md)
- [CLI Reference](cli/overview.md)
- [Architecture](architecture/overview.md)
