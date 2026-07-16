# Backend

OmniGen uses Candle as the compute backend with Vulkan GPU acceleration and CPU fallback.

## Device Selection

```
1. Try Vulkan GPU (preferred)
2. Fall back to CPU
3. Automatic selection based on availability
```

## Vulkan Backend

- Supports NVIDIA, AMD, and Intel GPUs
- Automatic device detection
- Zero-copy tensor operations
- Memory-mapped model loading

## CPU Fallback

When no GPU is available:

- Multi-threaded inference via rayon
- Optimized for x86_64 and ARM
- Automatic thread count selection

## Backend Health Check

```bash
omnigen doctor
```

Reports:

- Device name and type
- Available VRAM/RAM
- Compute capability test
