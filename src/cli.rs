//! CLI argument parsing using clap.
//!
//! Defines all command-line subcommands and their arguments.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// OmniGen: Production-quality multimodal AI runtime.
#[derive(Parser, Debug)]
#[command(
    name = "omnigen",
    version,
    about = "Single-executable multimodal AI runtime powered by Candle",
    long_about = "OmniGen is a self-contained multimodal AI runtime that supports \
                  text generation, image generation, video generation, and audio \
                  transcription using Candle as the compute backend."
)]
pub struct Cli {
    /// Enable verbose logging.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Operate in offline mode (no downloads).
    #[arg(long, global = true)]
    pub offline: bool,

    /// Enable dry run mode (no actual inference).
    #[arg(long, global = true)]
    pub dry_run: bool,

    /// Path to configuration file.
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

/// Available subcommands.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Execute a YAML pipeline workflow.
    Run {
        /// Path to the workflow YAML file.
        workflow: PathBuf,
    },

    /// Start an interactive chat session.
    Chat {
        /// Model to use for chat (default: qwen).
        #[arg(long, default_value = "qwen")]
        model: String,
    },

    /// Generate or transform an image.
    Image {
        /// Text prompt for image generation.
        #[arg(short, long)]
        prompt: String,

        /// Input image path for image-to-image transformation.
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output image path.
        #[arg(short, long, default_value = "output.png")]
        output: PathBuf,

        /// Image width.
        #[arg(long, default_value = "1024")]
        width: u32,

        /// Image height.
        #[arg(long, default_value = "1024")]
        height: u32,

        /// Number of inference steps.
        #[arg(long, default_value = "4")]
        steps: usize,

        /// Model to use (default: flux).
        #[arg(long, default_value = "flux")]
        model: String,
    },

    /// Generate or transform a video.
    Video {
        /// Text prompt for video generation.
        #[arg(short, long)]
        prompt: String,

        /// Input image path for image-to-video transformation.
        #[arg(short, long)]
        input: Option<PathBuf>,

        /// Output video path.
        #[arg(short, long, default_value = "output.mp4")]
        output: PathBuf,

        /// Number of frames.
        #[arg(long, default_value = "25")]
        frames: usize,

        /// Frame width.
        #[arg(long, default_value = "512")]
        width: u32,

        /// Frame height.
        #[arg(long, default_value = "320")]
        height: u32,

        /// Number of inference steps.
        #[arg(long, default_value = "4")]
        steps: usize,

        /// Model to use (default: ltx).
        #[arg(long, default_value = "ltx")]
        model: String,
    },

    /// Transcribe audio from a file.
    Audio {
        /// Path to the audio file.
        input: PathBuf,

        /// Output text file path.
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Language hint.
        #[arg(short, long)]
        language: Option<String>,
    },

    /// List, download, or update models.
    Models {
        #[command(subcommand)]
        action: Option<ModelsAction>,
    },

    /// Run system diagnostics.
    Doctor,

    /// Run performance benchmarks.
    Benchmark {
        /// Model to benchmark.
        #[arg(long)]
        model: Option<String>,

        /// Number of benchmark iterations.
        #[arg(long, default_value = "10")]
        iterations: usize,
    },
}

/// Model management actions.
#[derive(Subcommand, Debug)]
pub enum ModelsAction {
    /// List all available and installed models.
    List,

    /// Download all required models.
    Download {
        /// Specific model to download.
        #[arg(long)]
        model: Option<String>,
    },

    /// Update models to latest versions.
    Update {
        /// Specific model to update.
        #[arg(long)]
        model: Option<String>,
    },

    /// Remove downloaded models.
    Remove {
        /// Model to remove.
        model: String,
    },

    /// Show model information.
    Info {
        /// Model to inspect.
        model: String,
    },
}

/// Device preference for backend selection.
#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum DevicePreference {
    /// Automatic device selection (preferred).
    Auto,
    /// Force CPU execution.
    Cpu,
    /// Force Vulkan GPU execution.
    Vulkan,
}

impl std::fmt::Display for DevicePreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DevicePreference::Auto => write!(f, "auto"),
            DevicePreference::Cpu => write!(f, "cpu"),
            DevicePreference::Vulkan => write!(f, "vulkan"),
        }
    }
}
