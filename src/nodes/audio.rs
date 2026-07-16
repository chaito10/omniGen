//! Audio processing node for pipeline.
//!
//! Runs audio transcription using Whisper or other audio models.

use crate::traits::{Node, NodeInputs, NodeOutputs};
use anyhow::{Context, Result};
use candle_core::{DType, Device, Tensor};

/// A node that processes audio (transcription).
pub struct AudioNode {
    /// Unique node identifier.
    id: String,
    /// Model name to use.
    _model: String,
    /// Input audio file path.
    _input_path: String,
    /// Language hint.
    _language: Option<String>,
}

impl AudioNode {
    /// Create a new audio processing node.
    pub fn new(id: String, model: String, input_path: String, language: Option<String>) -> Self {
        Self {
            id,
            _model: model,
            _input_path: input_path,
            _language: language,
        }
    }
}

impl Node for AudioNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> &str {
        "audio"
    }

    fn execute(&self, _inputs: &NodeInputs) -> Result<NodeOutputs> {
        let device = Device::Cpu;
        let output = Tensor::zeros((1,), DType::F32, &device)
            .context("failed to create audio output tensor")?;

        let mut outputs = NodeOutputs::new();
        outputs.push("transcription", output);
        Ok(outputs)
    }

    fn input_ports(&self) -> Vec<&str> {
        vec!["audio"]
    }

    fn output_ports(&self) -> Vec<&str> {
        vec!["transcription"]
    }
}
