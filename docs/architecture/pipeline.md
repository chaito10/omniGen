# Pipeline Engine

The pipeline engine is a DAG (Directed Acyclic Graph) executor that processes YAML workflow definitions.

## Pipeline Flow

1. **Parse YAML** - Read pipeline steps from configuration
2. **Build Graph** - Create a dependency graph of nodes
3. **Resolve Order** - Topological sort to determine execution order
4. **Execute** - Run nodes in parallel where dependencies allow
5. **Save** - Write outputs to disk

## DAG Execution

Independent nodes execute in parallel using rayon:

```
Level 0: [Prompt]           (no dependencies)
Level 1: [Image, Text]      (depend on Prompt)
Level 2: [Save]             (depends on Image)
```

## Node Types

| Node | Inputs | Outputs | Description |
|---|---|---|---|
| `Prompt` | - | text | Hold a text prompt |
| `Text` | prompt | text | Generate text |
| `Image` | prompt | image | Generate image |
| `Video` | prompt | video | Generate video |
| `Audio` | audio | transcription | Transcribe audio |
| `Save` | any | - | Save output to disk |

## Example Pipeline

```yaml
pipeline:
  - prompt:
      text: "A futuristic city"
  - image:
      model: flux
      width: 1024
      height: 1024
  - save:
      path: output.png
```

## Features

- **Parallel execution** of independent nodes
- **Lazy model loading** on first use
- **Streaming** support for large outputs
- **Progress tracking** with progress bars
- **Cancellation** support
