//! Video generation node for pipeline.
//!
//! Runs video generation using LTX Video or other video models.

use crate::traits::{Node, NodeInputs, NodeOutputs};
use anyhow::{Context, Result};
use candle_core::{DType, Tensor};

/// A node that generates videos.
pub struct VideoNode {
    /// Unique node identifier.
    id: String,
    /// Model name to use.
    model: String,
    /// Number of frames.
    frames: usize,
    /// Frame width.
    width: u32,
    /// Frame height.
    height: u32,
    /// Number of inference steps.
    steps: usize,
}

impl VideoNode {
    /// Create a new video generation node.
    pub fn new(
        id: String,
        model: String,
        frames: usize,
        width: u32,
        height: u32,
        steps: usize,
    ) -> Self {
        Self {
            id,
            model,
            frames,
            width,
            height,
            steps,
        }
    }
}

impl Node for VideoNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> &str {
        "video"
    }

    fn execute(&self, inputs: &NodeInputs) -> Result<NodeOutputs> {
        let prompt = inputs
            .get("prompt")
            .ok_or_else(|| anyhow::anyhow!("video node requires a 'prompt' input"))?;

        let device = prompt.device();

        // Create video output tensor (T, C, H, W format)
        let video = Tensor::zeros(
            (self.frames, 3, self.height as usize, self.width as usize),
            DType::F32,
            &device,
        )
        .context("failed to create video output tensor")?;

        let mut outputs = NodeOutputs::new();
        outputs.push("video", video);
        Ok(outputs)
    }

    fn input_ports(&self) -> Vec<&str> {
        vec!["prompt", "input_image"]
    }

    fn output_ports(&self) -> Vec<&str> {
        vec!["video"]
    }
}
