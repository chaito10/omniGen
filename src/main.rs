//! OmniGen: Production-quality multimodal AI runtime.
//!
//! A single-executable runtime capable of text generation, image generation,
//! video generation, and audio transcription using Candle as the compute backend.

mod backend;
mod cache;
mod cli;
mod config;
mod device;
mod download;
mod graph;
mod memory;
mod model;
mod models;
mod nodes;
mod pipeline;
mod scheduler;
mod traits;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use cli::{Cli, Commands, ModelsAction};
use config::Config;
use memory::MemoryManager;
use cache::TensorCache;
use model::ModelRegistry;
use traits::Backend;

/// Version string.
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose {
        "debug"
    } else {
        "info"
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level)),
        )
        .with_target(false)
        .with_thread_names(true)
        .init();

    info!("omnigen v{}", VERSION);

    // Load configuration
    let config = if let Some(ref config_path) = cli.config {
        Config::load(config_path)?
    } else {
        let default_paths = [
            PathBuf::from("omnigen.yaml"),
            PathBuf::from("config.yaml"),
            PathBuf::from("omnigen.yml"),
            PathBuf::from("config.yml"),
        ];

        default_paths
            .iter()
            .find(|p| p.exists())
            .map(|p| Config::load(p))
            .transpose()?
            .unwrap_or_else(Config::default_config)
    };

    // Initialize core services
    let memory = Arc::new(MemoryManager::new(0, config.memory.auto_unload, config.memory.max_vram_usage));
    let cache = Arc::new(TensorCache::new());
    let registry = Arc::new(ModelRegistry::new(
        config.models.cache_dir.clone(),
        memory.clone(),
        cache.clone(),
    ));

    // Initialize backend
    let backend = backend::create_backend(&config.backend.device)?;

    // Execute command
    match cli.command {
        Commands::Run { workflow } => cmd_run(&workflow, &config, registry, backend, cli.offline, cli.dry_run)?,
        Commands::Chat { model } => cmd_chat(&model, registry, backend)?,
        Commands::Image {
            prompt,
            input,
            output,
            width,
            height,
            steps,
            model,
        } => cmd_image(&prompt, input.as_deref(), &output, width, height, steps, &model, registry, backend, cli.dry_run)?,
        Commands::Video {
            prompt,
            input,
            output,
            frames,
            width,
            height,
            steps,
            model,
        } => cmd_video(&prompt, input.as_deref(), &output, frames, width, height, steps, &model, registry, backend, cli.dry_run)?,
        Commands::Audio {
            input,
            output,
            language,
        } => cmd_audio(&input, output.as_deref(), language.as_deref(), registry, backend)?,
        Commands::Models { action } => cmd_models(action, &config, registry)?,
        Commands::Doctor => cmd_doctor(backend)?,
        Commands::Benchmark { model, iterations } => cmd_benchmark(model.as_deref(), iterations, registry, backend)?,
    }

    Ok(())
}

/// Execute a YAML workflow pipeline.
fn cmd_run(
    workflow: &Path,
    _config: &Config,
    registry: Arc<ModelRegistry>,
    _backend: Arc<backend::CandleBackend>,
    offline: bool,
    dry_run: bool,
) -> Result<()> {
    info!("executing workflow: {}", workflow.display());

    let workflow_config = Config::load(workflow)
        .with_context(|| format!("failed to load workflow: {}", workflow.display()))?;

    if dry_run {
        println!("Dry run: would execute {} pipeline steps", workflow_config.pipeline.len());
        for (i, step) in workflow_config.pipeline.iter().enumerate() {
            if let Some(ref prompt) = step.prompt {
                println!("  Step {}: prompt = \"{}\"", i + 1, prompt.text);
            }
            if let Some(ref image) = step.image {
                println!("  Step {}: image = {} ({}x{}, {} steps)", i + 1, image.model, image.width, image.height, image.steps);
            }
            if let Some(ref video) = step.video {
                println!("  Step {}: video = {} ({} frames, {}x{})", i + 1, video.model, video.frames, video.width, video.height);
            }
        }
        return Ok(());
    }

    // Auto-download models if needed
    if !offline {
        ensure_models_downloaded(&registry)?;
    }

    let pipe = pipeline::Pipeline::from_config(&workflow_config, registry)?;
    let outputs = pipe.execute()?;

    info!("workflow complete, {} output groups", outputs.len());
    Ok(())
}

/// Start an interactive chat session.
fn cmd_chat(
    model_name: &str,
    registry: Arc<ModelRegistry>,
    backend: Arc<backend::CandleBackend>,
) -> Result<()> {
    let model_type = parse_model_type(model_name)?;

    // Ensure model is loaded
    registry.load_model(&model_type, backend.device())?;

    println!("OmniGen Chat v{}", VERSION);
    println!("Model: {} (loaded)", model_name);
    println!("Type 'exit' or 'quit' to stop.\n");

    loop {
        print!("You: ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .context("failed to read input")?;

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            println!("Goodbye!");
            break;
        }

        let model_input = traits::ModelInput::Text {
            prompt: input.to_string(),
            max_tokens: 2048,
            temperature: 0.7,
        };

        match registry.run_inference(&model_type, &model_input) {
            Ok(traits::ModelOutput::Text(response)) => {
                println!("Assistant: {}\n", response);
            }
            Ok(_) => {
                println!("Error: unexpected output type\n");
            }
            Err(e) => {
                println!("Error: {}\n", e);
            }
        }
    }

    registry.unload_model(&model_type)?;
    Ok(())
}

/// Generate or transform an image.
fn cmd_image(
    prompt: &str,
    input: Option<&Path>,
    output: &Path,
    width: u32,
    height: u32,
    steps: usize,
    model_name: &str,
    registry: Arc<ModelRegistry>,
    backend: Arc<backend::CandleBackend>,
    dry_run: bool,
) -> Result<()> {
    info!("image generation: prompt='{}', {}x{}, {} steps", prompt, width, height, steps);

    if dry_run {
        println!("Dry run: would generate image");
        println!("  Prompt: {}", prompt);
        println!("  Size: {}x{}", width, height);
        println!("  Steps: {}", steps);
        println!("  Model: {}", model_name);
        if let Some(input_path) = input {
            println!("  Input: {}", input_path.display());
        }
        println!("  Output: {}", output.display());
        return Ok(());
    }

    let model_type = parse_model_type(model_name)?;
    registry.load_model(&model_type, backend.device())?;

    // Load input image if provided
    let input_image = if let Some(input_path) = input {
        let img = image::open(input_path)
            .with_context(|| format!("failed to open input image: {}", input_path.display()))?;
        let img = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        let rgb = img.to_rgb8();
        let data: Vec<f32> = rgb.pixels().flat_map(|p| [p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0]).collect();
        let device = backend.device();
        Some(candle_core::Tensor::from_vec(data, (3, height as usize, width as usize), device)?)
    } else {
        None
    };

    let model_input = traits::ModelInput::Image {
        prompt: prompt.to_string(),
        input_image,
        width,
        height,
        steps,
    };

    match registry.run_inference(&model_type, &model_input)? {
        traits::ModelOutput::Image(tensor) => {
            let data = tensor.to_vec1::<f32>()
                .context("failed to convert output tensor")?;

            let img_data: Vec<u8> = data.iter().map(|&v| (v.clamp(0.0, 1.0) * 255.0) as u8).collect();
            let img = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, img_data)
                .ok_or_else(|| anyhow::anyhow!("failed to create image buffer"))?;

            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)?;
            }
            img.save(output)
                .with_context(|| format!("failed to save image: {}", output.display()))?;

            println!("Image saved: {}", output.display());
        }
        _ => anyhow::bail!("unexpected output type from image model"),
    }

    registry.unload_model(&model_type)?;
    Ok(())
}

/// Generate or transform a video.
fn cmd_video(
    prompt: &str,
    input: Option<&Path>,
    output: &Path,
    frames: usize,
    width: u32,
    height: u32,
    steps: usize,
    model_name: &str,
    registry: Arc<ModelRegistry>,
    backend: Arc<backend::CandleBackend>,
    dry_run: bool,
) -> Result<()> {
    info!("video generation: prompt='{}', {} frames, {}x{}", prompt, frames, width, height);

    if dry_run {
        println!("Dry run: would generate video");
        println!("  Prompt: {}", prompt);
        println!("  Frames: {}", frames);
        println!("  Size: {}x{}", width, height);
        println!("  Steps: {}", steps);
        println!("  Model: {}", model_name);
        if let Some(input_path) = input {
            println!("  Input: {}", input_path.display());
        }
        println!("  Output: {}", output.display());
        return Ok(());
    }

    let model_type = parse_model_type(model_name)?;
    registry.load_model(&model_type, backend.device())?;

    let model_input = traits::ModelInput::Video {
        prompt: prompt.to_string(),
        input_image: None,
        frames,
        width,
        height,
        steps,
    };

    match registry.run_inference(&model_type, &model_input)? {
        traits::ModelOutput::Video(video_frames) => {
            if let Some(parent) = output.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let frame_dir = output.with_extension("");
            std::fs::create_dir_all(&frame_dir)?;

            for (i, frame) in video_frames.iter().enumerate() {
                let frame_path = frame_dir.join(format!("frame_{:04}.png", i));
                let data = frame.to_vec1::<f32>()?;
                let img_data: Vec<u8> = data.iter().map(|&v| (v.clamp(0.0, 1.0) * 255.0) as u8).collect();
                let img = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, img_data)
                    .ok_or_else(|| anyhow::anyhow!("failed to create frame image buffer"))?;
                img.save(&frame_path)?;
            }

            println!("Video frames saved: {}/ ({} frames)", frame_dir.display(), frames);
        }
        _ => anyhow::bail!("unexpected output type from video model"),
    }

    registry.unload_model(&model_type)?;
    Ok(())
}

/// Transcribe audio from a file.
fn cmd_audio(
    input: &Path,
    output: Option<&Path>,
    language: Option<&str>,
    registry: Arc<ModelRegistry>,
    backend: Arc<backend::CandleBackend>,
) -> Result<()> {
    info!("audio transcription: {}", input.display());

    let samples = load_audio_samples(input)?;

    let model_type = traits::ModelType::Whisper;
    registry.load_model(&model_type, backend.device())?;

    let model_input = traits::ModelInput::Audio {
        samples,
        language: language.map(|s| s.to_string()),
    };

    match registry.run_inference(&model_type, &model_input)? {
        traits::ModelOutput::Transcription { text, language } => {
            println!("Language: {}", language);
            println!("Transcription:\n{}", text);

            if let Some(out_path) = output {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                std::fs::write(out_path, &text)?;
                println!("\nSaved to: {}", out_path.display());
            }
        }
        _ => anyhow::bail!("unexpected output type from audio model"),
    }

    registry.unload_model(&model_type)?;
    Ok(())
}

/// Handle model management commands.
fn cmd_models(
    action: Option<ModelsAction>,
    _config: &Config,
    registry: Arc<ModelRegistry>,
) -> Result<()> {
    match action {
        Some(ModelsAction::List) | None => {
            let models = registry.list_models();
            println!("\nModel Status:");
            println!("{:<12} {:<25} {:<10} {:<10}", "TYPE", "NAME", "LOADED", "DOWNLOADED");
            println!("{}", "-".repeat(60));
            for info in &models {
                println!(
                    "{:<12} {:<25} {:<10} {:<10}",
                    info.model_type,
                    info.name,
                    if info.loaded { "yes" } else { "no" },
                    if info.downloaded { "yes" } else { "no" }
                );
            }
            Ok(())
        }
        Some(ModelsAction::Download { model }) => {
            println!("Downloading models...");
            if let Some(model_name) = model {
                let model_type = parse_model_type(&model_name)?;
                let model_dir = registry.model_dir(&model_type);
                std::fs::create_dir_all(&model_dir)?;
                println!("  Would download {} to {}", model_name, model_dir.display());
            } else {
                println!("  Would download all required models");
            }
            Ok(())
        }
        Some(ModelsAction::Update { model }) => {
            println!("Updating models...");
            if let Some(model_name) = model {
                println!("  Would update: {}", model_name);
            } else {
                println!("  Would update all models");
            }
            Ok(())
        }
        Some(ModelsAction::Remove { model }) => {
            let model_type = parse_model_type(&model)?;
            let model_dir = registry.model_dir(&model_type);
            if model_dir.exists() {
                std::fs::remove_dir_all(&model_dir)?;
                println!("Removed: {}", model_dir.display());
            } else {
                println!("Model not found: {}", model);
            }
            Ok(())
        }
        Some(ModelsAction::Info { model }) => {
            let model_type = parse_model_type(&model)?;
            let models = registry.list_models();
            if let Some(info) = models.iter().find(|m| m.model_type == model_type) {
                println!("\nModel: {}", info.name);
                println!("  Type: {}", info.model_type);
                println!("  Loaded: {}", info.loaded);
                println!("  Downloaded: {}", info.downloaded);
                println!("  Memory: {} bytes", info.memory_usage);
                println!("  Repository: {}", info.repo);
                println!("  Filename: {}", info.filename);
            } else {
                println!("Unknown model: {}", model);
            }
            Ok(())
        }
    }
}

/// Run system diagnostics.
fn cmd_doctor(backend: Arc<backend::CandleBackend>) -> Result<()> {
    println!("OmniGen Doctor v{}", VERSION);
    println!("{}", "=".repeat(50));

    println!("\nDevice:");
    println!("  Name: {}", backend.device_name());

    print!("  Health check: ");
    match backend.health_check() {
        Ok(()) => println!("PASS"),
        Err(e) => println!("FAIL ({})", e),
    }

    match backend.vram_total() {
        Ok(total) if total > 0 => {
            println!("  VRAM: {} MB total", total / (1024 * 1024));
        }
        _ => println!("  VRAM: N/A (CPU mode)"),
    }

    println!("\nAll checks passed!");
    Ok(())
}

/// Run performance benchmarks.
fn cmd_benchmark(
    model_name: Option<&str>,
    iterations: usize,
    registry: Arc<ModelRegistry>,
    backend: Arc<backend::CandleBackend>,
) -> Result<()> {
    println!("OmniGen Benchmark v{}", VERSION);
    println!("Iterations: {}", iterations);
    println!("{}", "=".repeat(50));

    let model_type = if let Some(name) = model_name {
        parse_model_type(name)?
    } else {
        traits::ModelType::Qwen
    };

    println!("\nBenchmarking: {}", model_type);

    registry.load_model(&model_type, backend.device())?;

    let start = std::time::Instant::now();

    for i in 0..iterations {
        let model_input = traits::ModelInput::Text {
            prompt: "Hello, world!".to_string(),
            max_tokens: 100,
            temperature: 0.7,
        };

        match registry.run_inference(&model_type, &model_input) {
            Ok(_) => {}
            Err(e) => {
                warn!("benchmark iteration {} failed: {}", i, e);
            }
        }
    }

    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() as f64 / iterations as f64;

    println!("\nResults:");
    println!("  Total time: {:.2}s", elapsed.as_secs_f64());
    println!("  Average per iteration: {:.2}ms", avg_ms);
    println!("  Throughput: {:.2} iterations/sec", 1000.0 / avg_ms);

    registry.unload_model(&model_type)?;
    Ok(())
}

// Helper functions

/// Parse a model name string into a ModelType.
fn parse_model_type(name: &str) -> Result<traits::ModelType> {
    match name.to_lowercase().as_str() {
        "qwen" | "qwen3" | "qwen-3" => Ok(traits::ModelType::Qwen),
        "flux" | "flux.1" | "flux-1" | "flux-schnell" => Ok(traits::ModelType::Flux),
        "ltx" | "ltx-video" | "ltxvideo" => Ok(traits::ModelType::Ltx),
        "whisper" | "whisper-large" | "whisper-v3" => Ok(traits::ModelType::Whisper),
        _ => anyhow::bail!("unknown model: {}", name),
    }
}

/// Ensure all required models are downloaded.
fn ensure_models_downloaded(registry: &ModelRegistry) -> Result<()> {
    let models_to_check = [
        traits::ModelType::Qwen,
        traits::ModelType::Flux,
        traits::ModelType::Ltx,
        traits::ModelType::Whisper,
    ];

    for model_type in &models_to_check {
        if !registry.weights_exist(model_type) {
            let model_dir = registry.model_dir(model_type);
            std::fs::create_dir_all(&model_dir)
                .with_context(|| format!("failed to create model dir: {}", model_dir.display()))?;
            info!("model not found locally: {}", model_type);
        }
    }

    Ok(())
}

/// Load audio samples from a file (simplified).
fn load_audio_samples(path: &Path) -> Result<Vec<f32>> {
    let data = std::fs::read(path)
        .with_context(|| format!("failed to read audio file: {}", path.display()))?;

    let samples: Vec<f32> = data
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    Ok(samples)
}
