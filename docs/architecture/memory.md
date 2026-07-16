# Memory Management

OmniGen automatically manages memory to ensure efficient resource usage.

## Key Features

- **Auto-unload** inactive models from VRAM
- **Tensor caching** to avoid redundant computation
- **Memory pooling** for efficient allocation
- **Configurable limits** for VRAM usage

## Memory Lifecycle

```
Model Request
    │
    ▼
Check Cache (tensor reuse)
    │
    ▼
Check Memory Budget
    │
    ▼
Unload Inactive Models (if needed)
    │
    ▼
Load Model Weights
    │
    ▼
Run Inference
    │
    ▼
Cache Results
```

## Configuration

```yaml
memory:
  auto_unload: true        # Auto-unload when VRAM is full
  max_vram_usage: 0.8      # Use at most 80% of VRAM
  tensor_cache: true       # Cache intermediate tensors
```

## Model Memory Usage

| Model | Disk Size | VRAM (inference) |
|---|---|---|
| Qwen3-8B | ~5GB | ~4GB |
| FLUX.1 Schnell | ~10GB | ~12GB |
| LTX Video | ~8GB | ~16GB |
| Whisper Large v3 Turbo | ~3GB | ~4GB |
