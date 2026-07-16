# Project Structure

## Directory Layout

```
OmniGen/
├── Cargo.toml            # Project manifest
├── README.md             # Project documentation
├── LICENSE               # MIT License
├── .gitignore            # Git ignore rules
├── mkdocs.yml            # Documentation configuration
├── docs/                 # MkDocs documentation
│   ├── index.md
│   ├── getting-started/
│   ├── cli/
│   ├── models/
│   ├── architecture/
│   └── development/
├── .github/
│   └── workflows/
│       ├── ci.yml        # CI pipeline
│       ├── pages.yml     # GitHub Pages deployment
│       └── release.yml   # Release automation
├── src/
│   ├── main.rs           # Entry point
│   ├── cli.rs            # CLI argument parsing
│   ├── config.rs         # YAML configuration
│   ├── backend.rs        # Candle backend
│   ├── device.rs         # Device detection
│   ├── model.rs          # Model registry
│   ├── traits.rs         # Core trait definitions
│   ├── graph.rs          # DAG graph
│   ├── pipeline.rs       # Pipeline execution
│   ├── scheduler.rs      # Parallel scheduler
│   ├── download.rs       # Download manager
│   ├── cache.rs          # Tensor cache
│   ├── memory.rs         # Memory management
│   ├── models/
│   │   ├── mod.rs
│   │   ├── qwen.rs       # Qwen3 text model
│   │   ├── flux.rs       # FLUX image model
│   │   ├── ltx.rs        # LTX Video model
│   │   └── whisper.rs    # Whisper audio model
│   ├── nodes/
│   │   ├── mod.rs
│   │   ├── prompt.rs     # Prompt node
│   │   ├── text.rs       # Text generation node
│   │   ├── image.rs      # Image generation node
│   │   ├── video.rs      # Video generation node
│   │   ├── audio.rs      # Audio processing node
│   │   └── save.rs       # Save output node
│   └── utils/
│       └── mod.rs        # Utility functions
└── release/              # Release artifacts
```

## Module Dependencies

```
main.rs
├── cli.rs
├── config.rs
├── backend.rs
│   └── device.rs
├── model.rs
│   ├── traits.rs
│   ├── cache.rs
│   └── memory.rs
├── pipeline.rs
│   ├── graph.rs
│   ├── scheduler.rs
│   └── nodes/
├── download.rs
├── models/
│   ├── qwen.rs
│   ├── flux.rs
│   ├── ltx.rs
│   └── whisper.rs
└── utils/
```
