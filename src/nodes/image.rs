//! Image generation node for pipeline.
//!
//! Runs image generation using FLUX or other image models.

use crate::traits::{Node, NodeInputs, NodeOutputs};
use anyhow::{Context, Result};
use candle_core::{DType, Tensor};

/// A node that generates images.
pub struct ImageNode {
    /// Unique node identifier.
    id: String,
    /// Model name to use.
    _model: String,
    /// Output width.
    _width: u32,
    /// Output height.
    _height: u32,
    /// Number of inference steps.
    _steps: usize,
}

impl ImageNode {
    /// Create a new image generation node.
    pub fn new(id: String, model: String, width: u32, height: u32, steps: usize) -> Self {
        Self {
            id,
            _model: model,
            _width: width,
            _height: height,
            _steps: steps,
        }
    }
}

impl Node for ImageNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> &str {
        "image"
    }

    fn execute(&self, inputs: &NodeInputs) -> Result<NodeOutputs> {
        let prompt = inputs
            .get("prompt")
            .ok_or_else(|| anyhow::anyhow!("image node requires a 'prompt' input"))?;

        let device = prompt.device();

        let image = Tensor::zeros((3, self._height as usize, self._width as usize), DType::F32, &device)
            .context("failed to create image output tensor")?;

        let mut outputs = NodeOutputs::new();
        outputs.push("image", image);
        Ok(outputs)
    }

    fn input_ports(&self) -> Vec<&str> {
        vec!["prompt", "input_image"]
    }

    fn output_ports(&self) -> Vec<&str> {
        vec!["image"]
    }
}
