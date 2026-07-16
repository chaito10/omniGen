# Quick Start

## Generate Your First Image

```bash
omnigen image --prompt "A beautiful sunset over mountains"
```

On first run, OmniGen will:

1. Detect your compute device (Vulkan GPU or CPU)
2. Download the FLUX.1 Schnell model (~10GB)
3. Generate and save the image to `output.png`

## Transform an Image

```bash
omnigen image --prompt "Anime style" --input photo.jpg --output anime.png
```

## Generate a Video

```bash
omnigen video --prompt "Drone shot over mountains" --frames 25
```

## Transcribe Audio

```bash
omnigen audio recording.wav
```

## Interactive Chat

```bash
omnigen chat
```

## Run a Pipeline

Create a `workflow.yaml`:

```yaml
backend:
  device: auto

models:
  auto_download: true

pipeline:
  - prompt:
      text: "A futuristic cyberpunk city at night"

  - image:
      model: flux
      width: 1024
      height: 1024
      steps: 4

  - save:
      path: output/cyberpunk.png
```

```bash
omnigen run workflow.yaml
```

## Check System Status

```bash
omnigen doctor       # System diagnostics
omnigen benchmark    # Performance test
omnigen models       # List model status
```
