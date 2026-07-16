# Configuration

OmniGen uses YAML-only configuration. No JSON workflows.

## Config File Locations

OmniGen searches for configuration in the following order:

1. Path specified with `--config` flag
2. `omnigen.yaml` in current directory
3. `config.yaml` in current directory
4. `omnigen.yml` in current directory
5. `config.yml` in current directory
6. Built-in defaults

## Configuration Structure

```yaml
backend:
  device: auto          # auto | cpu | vulkan

models:
  auto_download: true
  cache_dir: models/
  max_concurrent_downloads: 4
  mirror_url: null      # Optional HuggingFace mirror

memory:
  auto_unload: true
  max_vram_usage: 0.8
  tensor_cache: true

pipeline:
  - prompt:
      text: "A futuristic city"
  - image:
      model: flux
      width: 1024
      height: 1024
      steps: 4
  - save:
      path: output.png

output:
  image: output.png
  video: output.mp4
  text: output.txt
```

## Backend Configuration

| Key | Type | Default | Description |
|---|---|---|---|
| `device` | string | `auto` | Compute device preference |

Device options:

- `auto` - Try Vulkan, fall back to CPU
- `cpu` - Force CPU execution
- `vulkan` - Force Vulkan GPU

## Models Configuration

| Key | Type | Default | Description |
|---|---|---|---|
| `auto_download` | bool | `true` | Auto-download missing models |
| `cache_dir` | string | `models/` | Model storage directory |
| `max_concurrent_downloads` | int | `4` | Parallel download limit |
| `mirror_url` | string | `null` | HuggingFace mirror URL |

## Memory Configuration

| Key | Type | Default | Description |
|---|---|---|---|
| `auto_unload` | bool | `true` | Auto-unload inactive models |
| `max_vram_usage` | float | `0.8` | Max VRAM fraction (0.0-1.0) |
| `tensor_cache` | bool | `true` | Enable tensor caching |

## Environment Variables

| Variable | Description |
|---|---|
| `HF_TOKEN` | HuggingFace authentication token |
| `RUST_LOG` | Log level (debug, info, warn, error) |
