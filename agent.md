You are an expert Rust systems programmer.

Build a production-quality, single executable multimodal AI runtime.

Project name:
Candle Studio

Goal

Create ONE executable capable of

• Text Generation
• Image Generation
• Image Transformation
• Video Generation
• Video Transformation
• Audio Transcription
• Audio Generation

The project must be completely self-contained.

No Python.
No Node.js.
No Java.
No Docker.
No external scripts.

Everything must be implemented in Rust.

====================================================
Core requirements
====================================================

Backend

• Candle
• Vulkan backend (preferred)
• CPU fallback
• Automatic device selection

Executable

omnigen.exe

or

omnigen

Only one executable should be produced.

====================================================
Supported models
====================================================

Text

Qwen3 GGUF

Image

FLUX.1 Schnell

Video

LTX Video Distilled GGUF

Audio

Whisper Large v3 Turbo

====================================================
Model management
====================================================

The executable must automatically download models.

Do NOT require

hf
git-lfs
git
python

Use reqwest only.

Download directly from HuggingFace

https://huggingface.co/<repo>/resolve/main/<file>

Support

• resume downloads
• progress bars
• retries
• checksum verification
• caching
• offline mode
• update checking

Models should be stored under

models/

Example

models/
    qwen.gguf
    flux.safetensors
    ltx.gguf
    whisper.safetensors

If model exists

skip download.

====================================================
Configuration
====================================================

Use YAML only.

Example

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

No JSON workflows.

====================================================
CLI
====================================================

omnigen run workflow.yaml

omnigen chat

omnigen image \
    --prompt "A castle"

omnigen image \
    --prompt "Anime style" \
    --input cat.png

omnigen video \
    --prompt "Drone over mountains"

omnigen video \
    --input image.png \
    --prompt "Animate"

omnigen audio speech.wav

omnigen models

omnigen models update

omnigen models download

omnigen doctor

omnigen benchmark

====================================================
Architecture
====================================================

Everything should be modular.

Use traits.

Example

Backend

Model

Node

Pipeline

Plugin

Every modality should implement the same interface.

trait Model {

load()

run()

unload()

}

====================================================
Pipeline
====================================================

Implement a DAG executor.

Read YAML

↓

Build graph

↓

Resolve dependencies

↓

Execute nodes

↓

Save outputs

Support

parallel execution

lazy loading

streaming

progress

cancellation

====================================================
Nodes
====================================================

Prompt

Text

Image

Image2Image

Video

Image2Video

Audio

Save

Load

Merge

Condition

Repeat

Loop

====================================================
Memory
====================================================

Automatically unload inactive models.

Keep only one large model in VRAM.

Cache tensors.

Use memory pooling.

====================================================
Downloads
====================================================

Implement a DownloadManager.

Features

parallel downloads

resume

progress

retry

sha256 verification

cache

authentication

HF_TOKEN support

Mirror support

====================================================
Libraries
====================================================

candle

tokio

reqwest

serde

serde_yaml

clap

indicatif

sha2

directories

anyhow

thiserror

petgraph

rayon

parking_lot

====================================================
No unnecessary dependencies.

====================================================
Project structure
====================================================

src/

main.rs

cli.rs

config.rs

download.rs

backend.rs

graph.rs

pipeline.rs

scheduler.rs

cache.rs

memory.rs

model.rs

device.rs

traits.rs

models/

qwen.rs

flux.rs

ltx.rs

whisper.rs

nodes/

prompt.rs

text.rs

image.rs

video.rs

audio.rs

save.rs

utils/

====================================================
Features
====================================================

Automatic Vulkan detection

CPU fallback

Colored logs

Progress bars

Benchmark mode

Dry run

Verbose mode

Offline mode

Portable mode

Portable cache

Cross platform

Windows

Linux

macOS

====================================================
Performance
====================================================

Zero-copy where possible

Streaming tensors

Memory mapped GGUF

Parallel scheduler

Lazy model loading

Minimal allocations

====================================================
Quality
====================================================

Rust 2024 edition

cargo fmt

cargo clippy clean

cargo test

Document every module.

Avoid unwrap().

Proper error handling.

====================================================
Output
====================================================

Produce a complete Cargo project.

The project should compile.

No placeholders.

No TODOs.

No pseudocode.

Every file should be fully implemented.

The final result must be a single executable capable of automatically downloading the required models and performing text, image, video, and audio inference using Candle.


### One recommendation

I would slightly change the architecture to make it future-proof by separating the **engine** from the **models**.

```
omnigen.exe
        │
        ├── Candle Backend
        ├── Vulkan
        ├── YAML Engine
        ├── Download Manager
        ├── Model Registry
        └── Plugin Loader
                 │
      ┌──────────┼───────────┐
      │          │           │
   Qwen      FLUX       LTX Video
      │          │           │
    Whisper   (future SDXL) (future Wan)
```

With this design, adding a new model is just registering another plugin rather than modifying the core runtime. It keeps the executable as a **single binary** while making the system much easier to extend.
