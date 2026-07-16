//! YAML configuration parsing and management.
//!
//! All configuration is YAML-only. No JSON workflows.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Top-level configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Backend configuration.
    #[serde(default)]
    pub backend: BackendConfig,

    /// Model management configuration.
    #[serde(default)]
    pub models: ModelsConfig,

    /// Memory management configuration.
    #[serde(default)]
    pub memory: MemoryConfig,

    /// Pipeline definition (optional, for `omnigen run`).
    #[serde(default)]
    pub pipeline: Vec<PipelineStep>,

    /// Output configuration.
    #[serde(default)]
    pub output: OutputConfig,
}

/// Backend device configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Device preference: "auto", "cpu", "vulkan".
    #[serde(default = "default_device")]
    pub device: String,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            device: default_device(),
        }
    }
}

fn default_device() -> String {
    "auto".to_string()
}

/// Model management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsConfig {
    /// Whether to auto-download missing models.
    #[serde(default = "default_true")]
    pub auto_download: bool,

    /// Directory to store downloaded models.
    #[serde(default = "default_models_dir")]
    pub cache_dir: PathBuf,

    /// Maximum concurrent downloads.
    #[serde(default = "default_max_concurrent_downloads")]
    pub max_concurrent_downloads: usize,

    /// HuggingFace mirror URL (optional).
    #[serde(default)]
    pub mirror_url: Option<String>,
}

impl Default for ModelsConfig {
    fn default() -> Self {
        Self {
            auto_download: default_true(),
            cache_dir: default_models_dir(),
            max_concurrent_downloads: default_max_concurrent_downloads(),
            mirror_url: None,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_models_dir() -> PathBuf {
    PathBuf::from("models")
}

fn default_max_concurrent_downloads() -> usize {
    4
}

/// Memory management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Whether to automatically unload inactive models.
    #[serde(default = "default_true")]
    pub auto_unload: bool,

    /// Maximum fraction of VRAM to use (0.0 - 1.0).
    #[serde(default = "default_max_vram")]
    pub max_vram_usage: f32,

    /// Whether to enable tensor caching.
    #[serde(default = "default_true")]
    pub tensor_cache: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            auto_unload: default_true(),
            max_vram_usage: default_max_vram(),
            tensor_cache: default_true(),
        }
    }
}

fn default_max_vram() -> f32 {
    0.8
}

/// A single step in a pipeline definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    /// Prompt step.
    #[serde(default)]
    pub prompt: Option<PromptStep>,

    /// Text generation step.
    #[serde(default)]
    pub text: Option<TextStep>,

    /// Image generation step.
    #[serde(default)]
    pub image: Option<ImageStep>,

    /// Video generation step.
    #[serde(default)]
    pub video: Option<VideoStep>,

    /// Audio processing step.
    #[serde(default)]
    pub audio: Option<AudioStep>,

    /// Save output step.
    #[serde(default)]
    pub save: Option<SaveStep>,

    /// Load input step.
    #[serde(default)]
    pub load: Option<LoadStep>,

    /// Loop step.
    #[serde(default)]
    pub r#loop: Option<LoopStep>,

    /// Condition step.
    #[serde(default)]
    pub condition: Option<ConditionStep>,
}

/// Prompt configuration in a pipeline step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptStep {
    /// The prompt text.
    pub text: String,
}

/// Text generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStep {
    /// Model to use (e.g., "qwen").
    pub model: String,
    /// Maximum tokens.
    #[serde(default = "default_max_tokens")]
    pub max_tokens: usize,
    /// Temperature.
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

fn default_max_tokens() -> usize {
    2048
}

fn default_temperature() -> f32 {
    0.7
}

/// Image generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageStep {
    /// Model to use (e.g., "flux").
    pub model: String,
    /// Output width.
    #[serde(default = "default_image_width")]
    pub width: u32,
    /// Output height.
    #[serde(default = "default_image_height")]
    pub height: u32,
    /// Number of inference steps.
    #[serde(default = "default_steps")]
    pub steps: usize,
    /// Optional input image path for image-to-image.
    #[serde(default)]
    pub input: Option<String>,
}

fn default_image_width() -> u32 {
    1024
}

fn default_image_height() -> u32 {
    1024
}

fn default_steps() -> usize {
    4
}

/// Video generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStep {
    /// Model to use (e.g., "ltx").
    pub model: String,
    /// Number of frames.
    #[serde(default = "default_frames")]
    pub frames: usize,
    /// Frame width.
    #[serde(default = "default_video_width")]
    pub width: u32,
    /// Frame height.
    #[serde(default = "default_video_height")]
    pub height: u32,
    /// Number of inference steps.
    #[serde(default = "default_steps")]
    pub steps: usize,
    /// Optional input image path for image-to-video.
    #[serde(default)]
    pub input: Option<String>,
}

fn default_frames() -> usize {
    25
}

fn default_video_width() -> u32 {
    512
}

fn default_video_height() -> u32 {
    320
}

/// Audio processing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStep {
    /// Model to use (e.g., "whisper").
    pub model: String,
    /// Input audio file path.
    pub input: String,
    /// Language hint.
    #[serde(default)]
    pub language: Option<String>,
}

/// Save output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveStep {
    /// Output file path.
    pub path: String,
    /// Output format hint.
    #[serde(default)]
    pub format: Option<String>,
}

/// Load input configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadStep {
    /// Input file path.
    pub path: String,
}

/// Loop configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStep {
    /// Number of iterations.
    pub count: usize,
    /// Steps to repeat.
    pub steps: Vec<PipelineStep>,
}

/// Conditional execution configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionStep {
    /// Condition expression (e.g., "model_loaded:flux").
    pub r#if: String,
    /// Steps to execute if condition is true.
    #[serde(default)]
    pub then: Vec<PipelineStep>,
    /// Steps to execute if condition is false.
    #[serde(default)]
    pub r#else: Vec<PipelineStep>,
}

/// Output configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OutputConfig {
    /// Default image output path.
    #[serde(default)]
    pub image: Option<String>,
    /// Default video output path.
    #[serde(default)]
    pub video: Option<String>,
    /// Default text output path.
    #[serde(default)]
    pub text: Option<String>,
    /// Default audio output path.
    #[serde(default)]
    pub audio: Option<String>,
}

impl Config {
    /// Load configuration from a YAML file.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("failed to read config file: {}", path.display()))?;
        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("failed to parse config file: {}", path.display()))?;
        Ok(config)
    }

    /// Load configuration from a YAML string.
    pub fn from_str(s: &str) -> Result<Self> {
        let config: Config =
            serde_yaml::from_str(s).with_context(|| "failed to parse config YAML")?;
        Ok(config)
    }

    /// Save configuration to a YAML file.
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self)
            .with_context(|| "failed to serialize config to YAML")?;
        std::fs::write(path, content)
            .with_context(|| format!("failed to write config file: {}", path.display()))?;
        Ok(())
    }

    /// Create a default configuration.
    pub fn default_config() -> Self {
        Self {
            backend: BackendConfig::default(),
            models: ModelsConfig::default(),
            memory: MemoryConfig::default(),
            pipeline: Vec::new(),
            output: OutputConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default_config();
        assert_eq!(config.backend.device, "auto");
        assert!(config.models.auto_download);
        assert_eq!(config.models.max_concurrent_downloads, 4);
        assert!(config.memory.auto_unload);
        assert!((config.memory.max_vram_usage - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_yaml() {
        let yaml = r#"
backend:
  device: vulkan
models:
  auto_download: true
  cache_dir: /tmp/models
"#;
        let config = Config::from_str(yaml).unwrap();
        assert_eq!(config.backend.device, "vulkan");
        assert_eq!(config.models.cache_dir, PathBuf::from("/tmp/models"));
    }
}
