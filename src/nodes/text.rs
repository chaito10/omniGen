//! Text generation node for pipeline.
//!
//! Runs text generation using a loaded language model.

use crate::traits::{Node, NodeInputs, NodeOutputs};
use anyhow::{Context, Result};
use candle_core::Tensor;

/// A node that generates text using a language model.
pub struct TextNode {
    /// Unique node identifier.
    id: String,
    /// Model name to use.
    model: String,
    /// Maximum tokens to generate.
    max_tokens: usize,
    /// Sampling temperature.
    temperature: f32,
}

impl TextNode {
    /// Create a new text generation node.
    pub fn new(id: String, model: String, max_tokens: usize, temperature: f32) -> Self {
        Self {
            id,
            model,
            max_tokens,
            temperature,
        }
    }
}

impl Node for TextNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> &str {
        "text"
    }

    fn execute(&self, inputs: &NodeInputs) -> Result<NodeOutputs> {
        let prompt = inputs
            .get("prompt")
            .ok_or_else(|| anyhow::anyhow!("text node requires a 'prompt' input"))?;

        // In production, this would call the model registry to run inference
        // For now, create a placeholder output
        let device = prompt.device();
        let output = Tensor::zeros((1, self.max_tokens), candle_core::DType::F32, &device)
            .context("failed to create text output tensor")?;

        let mut outputs = NodeOutputs::new();
        outputs.push("text", output);
        Ok(outputs)
    }

    fn input_ports(&self) -> Vec<&str> {
        vec!["prompt"]
    }

    fn output_ports(&self) -> Vec<&str> {
        vec!["text"]
    }
}
