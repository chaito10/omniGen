# Commands

## `omnigen run`

Execute a YAML pipeline workflow.

```bash
omnigen run <workflow.yaml>
```

**Arguments:**

- `workflow` - Path to the workflow YAML file

## `omnigen chat`

Start an interactive chat session.

```bash
omnigen chat [--model <MODEL>]
```

**Options:**

- `--model <MODEL>` - Model to use (default: `qwen`)

## `omnigen image`

Generate or transform an image.

```bash
omnigen image --prompt <PROMPT> [OPTIONS]
```

**Options:**

| Flag | Default | Description |
|---|---|---|
| `-p, --prompt` | (required) | Text prompt |
| `-i, --input` | - | Input image for transformation |
| `-o, --output` | `output.png` | Output file path |
| `--width` | `1024` | Image width |
| `--height` | `1024` | Image height |
| `--steps` | `4` | Inference steps |
| `--model` | `flux` | Model to use |

## `omnigen video`

Generate or transform a video.

```bash
omnigen video --prompt <PROMPT> [OPTIONS]
```

**Options:**

| Flag | Default | Description |
|---|---|---|
| `-p, --prompt` | (required) | Text prompt |
| `-i, --input` | - | Input image for transformation |
| `-o, --output` | `output.mp4` | Output path |
| `--frames` | `25` | Number of frames |
| `--width` | `512` | Frame width |
| `--height` | `320` | Frame height |
| `--steps` | `4` | Inference steps |
| `--model` | `ltx` | Model to use |

## `omnigen audio`

Transcribe audio from a file.

```bash
omnigen audio <INPUT> [OPTIONS]
```

**Arguments:**

- `input` - Path to the audio file

**Options:**

| Flag | Description |
|---|---|
| `-o, --output` | Output text file path |
| `-l, --language` | Language hint |

## `omnigen models`

List, download, or update models.

```bash
omnigen models [SUBCOMMAND]
```

**Subcommands:**

| Subcommand | Description |
|---|---|
| `list` | List all models (default) |
| `download` | Download models |
| `update` | Update models |
| `remove <MODEL>` | Remove a model |
| `info <MODEL>` | Show model information |

## `omnigen doctor`

Run system diagnostics.

```bash
omnigen doctor
```

## `omnigen benchmark`

Run performance benchmarks.

```bash
omnigen benchmark [--model <MODEL>] [--iterations <N>]
```

**Options:**

| Flag | Default | Description |
|---|---|---|
| `--model` | `qwen` | Model to benchmark |
| `--iterations` | `10` | Number of iterations |
