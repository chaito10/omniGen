//! Prompt node for pipeline.
//!
//! Holds a text prompt that can be fed into downstream nodes.

use crate::traits::{Node, NodeInputs, NodeOutputs};
use anyhow::Result;
use candle_core::{Device, Tensor};

/// A node that holds a text prompt.
pub struct PromptNode {
    /// Unique node identifier.
    id: String,
    /// The prompt text.
    _text: String,
}

impl PromptNode {
    /// Create a new prompt node.
    pub fn new(id: String, text: String) -> Self {
        Self { id, _text: text }
    }

    /// Get the prompt text.
    pub fn text(&self) -> &str {
        &self._text
    }
}

impl Node for PromptNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> &str {
        "prompt"
    }

    fn execute(&self, _inputs: &NodeInputs) -> Result<NodeOutputs> {
        let mut outputs = NodeOutputs::new();
        let chars: Vec<f32> = self._text.chars().map(|c| c as u32 as f32).collect();
        let device = Device::Cpu;
        let tensor = Tensor::new(chars.as_slice(), &device)?
            .unsqueeze(0)?;
        outputs.push("text", tensor);
        Ok(outputs)
    }

    fn input_ports(&self) -> Vec<&str> {
        vec![]
    }

    fn output_ports(&self) -> Vec<&str> {
        vec!["text"]
    }
}
