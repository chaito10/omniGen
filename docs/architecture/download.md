# Download Manager

The download manager handles automatic model retrieval from HuggingFace.

## Features

- **Direct HTTP downloads** (no git-lfs, no Python)
- **Resume interrupted downloads**
- **SHA-256 checksum verification**
- **Parallel downloads** with configurable concurrency
- **Progress bars** for all downloads
- **Retry with exponential backoff**
- **HF_TOKEN authentication** for gated models
- **Mirror URL support**

## Download Flow

```
Check if model exists locally
    │
    ▼
Verify checksum (if exists)
    │
    ▼
Download from HuggingFace
    │
    ▼
Support resume (Range header)
    │
    ▼
Verify SHA-256 checksum
    │
    ▼
Cache locally
```

## Configuration

```yaml
models:
  auto_download: true
  cache_dir: models/
  max_concurrent_downloads: 4
  mirror_url: null  # Optional mirror
```

## Authentication

Set `HF_TOKEN` for gated models:

```bash
export HF_TOKEN=hf_your_token_here
omnigen models download
```

## URL Format

```
https://huggingface.co/<repo>/resolve/main/<file>
```

Example:

```
https://huggingface.co/Qwen/Qwen3-8B-GGUF/resolve/main/qwen3-8b.gguf
```
