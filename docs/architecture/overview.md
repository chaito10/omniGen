# Architecture Overview

OmniGen follows a modular, plugin-based architecture with clear separation of concerns.

## High-Level Architecture

```
omnigen
│
├── Candle Backend (Vulkan / CPU fallback)
├── YAML Pipeline Engine (DAG executor)
├── Download Manager (HuggingFace direct)
├── Model Registry
├── Plugin System
│
├── Models
│   ├── Qwen3 (Text)
│   ├── FLUX (Image)
│   ├── LTX Video (Video)
│   └── Whisper (Audio)
│
└── Nodes
    ├── Prompt
    ├── Text
    ├── Image / Image2Image
    ├── Video / Image2Video
    ├── Audio
    └── Save
```

## Core Traits

Every component implements a core trait:

| Trait | Purpose |
|---|---|
| `Model` | Load, run, and unload AI models |
| `Backend` | Manage compute devices |
| `Node` | Execute pipeline nodes |
| `Plugin` | Register new models |

## Design Principles

1. **Single Executable** - Everything compiled into one binary
2. **No External Dependencies** - No Python, Docker, or Node.js
3. **Plugin Architecture** - New models are registered, not hardcoded
4. **YAML-Only Configuration** - No JSON workflows
5. **Memory Efficient** - Automatic model lifecycle management

## Pipeline Execution

```
YAML Config
    │
    ▼
Parse Pipeline Steps
    │
    ▼
Build DAG Graph
    │
    ▼
Resolve Dependencies (Topological Sort)
    │
    ▼
Execute Nodes (Parallel where possible)
    │
    ▼
Save Outputs
```

See [Pipeline Engine](pipeline.md) for details.
