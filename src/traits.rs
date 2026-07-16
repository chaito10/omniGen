//! Core trait definitions for the OmniGen runtime.
//!
//! All modalities implement the same `Model` trait, making the system
//! extensible through a plugin-based architecture.

use anyhow::Result;
use candle_core::{Device, Tensor};

/// Supported model types in the runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ModelType {
    /// Qwen3 text generation (GGUF)
    Qwen,
    /// FLUX.1 Schnell image generation (SafeTensors)
    Flux,
    /// LTX Video Distilled video generation (GGUF)
    Ltx,
    /// Whisper Large v3 Turbo audio transcription (SafeTensors)
    Whisper,
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelType::Qwen => write!(f, "qwen"),
            ModelType::Flux => write!(f, "flux"),
            ModelType::Ltx => write!(f, "ltx"),
            ModelType::Whisper => write!(f, "whisper"),
        }
    }
}

/// Input payload passed to a model during inference.
#[derive(Debug, Clone)]
pub enum ModelInput {
    /// Text prompt for language models.
    Text {
        /// The prompt string.
        prompt: String,
        /// Maximum tokens to generate.
        max_tokens: usize,
        /// Sampling temperature.
        temperature: f32,
    },
    /// Image generation request.
    Image {
        /// Text prompt describing the desired image.
        prompt: String,
        /// Optional input image for image-to-image transformation.
        input_image: Option<Tensor>,
        /// Output width.
        width: u32,
        /// Output height.
        height: u32,
        /// Number of inference steps.
        steps: usize,
    },
    /// Video generation request.
    Video {
        /// Text prompt describing the desired video.
        prompt: String,
        /// Optional input image for image-to-video transformation.
        input_image: Option<Tensor>,
        /// Number of frames to generate.
        frames: usize,
        /// Frame width.
        width: u32,
        /// Frame height.
        height: u32,
        /// Number of inference steps.
        steps: usize,
    },
    /// Audio transcription request.
    Audio {
        /// Raw audio samples (f32, mono, 16kHz).
        samples: Vec<f32>,
        /// Language hint (optional).
        language: Option<String>,
    },
}

/// Output produced by a model after inference.
#[derive(Debug, Clone)]
pub enum ModelOutput {
    /// Generated text.
    Text(String),
    /// Generated image tensor (CHW format).
    Image(Tensor),
    /// Generated video frames (list of image tensors).
    Video(Vec<Tensor>),
    /// Transcribed text from audio.
    Transcription {
        /// The transcribed text.
        text: String,
        /// Detected language.
        language: String,
    },
}

/// The core model trait. Every modality implements this same interface.
///
/// # Examples
///
/// ```ignore
/// let mut model = FluxModel::new(device)?;
/// model.load("models/flux/model.safetensors")?;
/// let output = model.run(ModelInput::Image { ... })?;
/// model.unload()?;
/// ```
pub trait Model: Send + Sync {
    /// Returns the model type.
    fn model_type(&self) -> ModelType;

    /// Returns the human-readable model name.
    fn name(&self) -> &str;

    /// Returns the expected file extension for this model's weights.
    fn file_extension(&self) -> &str;

    /// Returns the HuggingFace repository ID for downloading.
    fn huggingface_repo(&self) -> &str;

    /// Returns the filename of the model weights on HuggingFace.
    fn huggingface_filename(&self) -> &str;

    /// Load model weights from the given path into the specified device.
    fn load(&mut self, path: &std::path::Path, device: &Device) -> Result<()>;

    /// Run inference on the loaded model with the given input.
    fn run(&self, input: &ModelInput) -> Result<ModelOutput>;

    /// Unload model weights from memory, freeing VRAM/RAM.
    fn unload(&mut self) -> Result<()>;

    /// Returns true if the model is currently loaded in memory.
    fn is_loaded(&self) -> bool;

    /// Returns the estimated memory usage in bytes.
    fn memory_usage(&self) -> usize;
}

/// Backend trait for managing compute devices.
pub trait Backend: Send + Sync {
    /// Initialize the backend with automatic device selection.
    fn init() -> Result<Self>
    where
        Self: Sized;

    /// Returns the underlying Candle device.
    fn device(&self) -> &Device;

    /// Returns a human-readable device name (e.g., "Vulkan RTX 4090").
    fn device_name(&self) -> &str;

    /// Returns available VRAM in bytes.
    fn vram_available(&self) -> Result<u64>;

    /// Returns total VRAM in bytes.
    fn vram_total(&self) -> Result<u64>;

    /// Check if the backend is functional.
    fn health_check(&self) -> Result<()>;
}

/// Node trait for pipeline graph nodes.
pub trait Node: Send + Sync {
    /// Returns the node's unique identifier.
    fn id(&self) -> &str;

    /// Returns the node type name.
    fn node_type(&self) -> &str;

    /// Execute the node with the given inputs.
    fn execute(&self, inputs: &NodeInputs) -> Result<NodeOutputs>;

    /// Returns the list of input port names this node expects.
    fn input_ports(&self) -> Vec<&str>;

    /// Returns the list of output port names this node produces.
    fn output_ports(&self) -> Vec<&str>;
}

/// A collection of named inputs to a pipeline node.
#[derive(Debug, Default, Clone)]
pub struct NodeInputs {
    entries: Vec<(String, Tensor)>,
}

impl NodeInputs {
    /// Create an empty input set.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a named tensor input.
    pub fn push(&mut self, name: impl Into<String>, tensor: Tensor) {
        self.entries.push((name.into(), tensor));
    }

    /// Get a tensor by port name.
    pub fn get(&self, name: &str) -> Option<&Tensor> {
        self.entries.iter().find(|(n, _)| n == name).map(|(_, t)| t)
    }

    /// Get a tensor by port name, returning an error if not found.
    pub fn require(&self, name: &str) -> Result<&Tensor> {
        self.get(name)
            .ok_or_else(|| anyhow::anyhow!("missing required input: {}", name))
    }

    /// Returns the number of inputs.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if there are no inputs.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// A collection of named outputs from a pipeline node.
#[derive(Debug, Default, Clone)]
pub struct NodeOutputs {
    entries: Vec<(String, Tensor)>,
}

impl NodeOutputs {
    /// Create an empty output set.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add a named tensor output.
    pub fn push(&mut self, name: impl Into<String>, tensor: Tensor) {
        self.entries.push((name.into(), tensor));
    }

    /// Get a tensor by port name.
    pub fn get(&self, name: &str) -> Option<&Tensor> {
        self.entries.iter().find(|(n, _)| n == name).map(|(_, t)| t)
    }

    /// Consume self and extract the first output tensor.
    pub fn into_first(self) -> Result<Tensor> {
        self.entries
            .into_iter()
            .next()
            .map(|(_, t)| t)
            .ok_or_else(|| anyhow::anyhow!("no outputs produced"))
    }

    /// Returns the number of outputs.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if there are no outputs.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Plugin trait for extensible model registration.
pub trait Plugin: Send + Sync {
    /// Returns the plugin name.
    fn name(&self) -> &str;

    /// Returns the plugin version.
    fn version(&self) -> &str;

    /// Create a new model instance from this plugin.
    fn create_model(&self) -> Result<Box<dyn Model>>;
}
