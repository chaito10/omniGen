//! Save output node for pipeline.
//!
//! Saves pipeline outputs to files (images, videos, text).

use crate::traits::{Node, NodeInputs, NodeOutputs};
use anyhow::{Context, Result};
use candle_core::Tensor;
use std::path::PathBuf;

/// A node that saves outputs to disk.
pub struct SaveNode {
    /// Unique node identifier.
    id: String,
    /// Output file path.
    path: String,
    /// Output format hint.
    _format: Option<String>,
}

impl SaveNode {
    /// Create a new save node.
    pub fn new(id: String, path: String, format: Option<String>) -> Self {
        Self {
            id,
            path,
            _format: format,
        }
    }
}

impl Node for SaveNode {
    fn id(&self) -> &str {
        &self.id
    }

    fn node_type(&self) -> &str {
        "save"
    }

    fn execute(&self, inputs: &NodeInputs) -> Result<NodeOutputs> {
        if let Some(image) = inputs.get("image") {
            self.save_image(image)?;
        } else if let Some(video) = inputs.get("video") {
            self.save_video(video)?;
        } else if let Some(text) = inputs.get("text") {
            self.save_text(text)?;
        } else if let Some(transcription) = inputs.get("transcription") {
            self.save_text(transcription)?;
        } else {
            return Ok(NodeOutputs::new());
        }

        Ok(NodeOutputs::new())
    }

    fn input_ports(&self) -> Vec<&str> {
        vec!["image", "video", "text", "transcription"]
    }

    fn output_ports(&self) -> Vec<&str> {
        vec![]
    }
}

impl SaveNode {
    fn save_image(&self, tensor: &Tensor) -> Result<()> {
        let path = PathBuf::from(&self.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        let data = tensor.to_vec1::<f32>()
            .context("failed to convert tensor to vec")?;

        let img_data: Vec<u8> = data.iter().map(|&v| (v.clamp(0.0, 1.0) * 255.0) as u8).collect();
        let width = tensor.dim(2).unwrap_or(512) as u32;
        let height = tensor.dim(1).unwrap_or(512) as u32;

        if let Some(img) = image::ImageBuffer::<image::Rgb<u8>, _>::from_raw(width, height, img_data) {
            img.save(&path)
                .with_context(|| format!("failed to save image: {}", path.display()))?;
        } else {
            std::fs::write(&path, format!("{:?}", data))
                .with_context(|| format!("failed to write image data: {}", path.display()))?;
        }

        tracing::info!("saved image: {}", path.display());
        Ok(())
    }

    fn save_video(&self, tensor: &Tensor) -> Result<()> {
        let path = PathBuf::from(&self.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        let data = tensor.to_vec1::<f32>()
            .context("failed to convert tensor to vec")?;

        std::fs::write(&path, format!("{:?}", data))
            .with_context(|| format!("failed to write video: {}", path.display()))?;

        tracing::info!("saved video: {}", path.display());
        Ok(())
    }

    fn save_text(&self, tensor: &Tensor) -> Result<()> {
        let path = PathBuf::from(&self.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory: {}", parent.display()))?;
        }

        let data = tensor.to_vec1::<f32>()
            .context("failed to convert tensor to vec")?;

        let text: String = data.iter()
            .filter_map(|&v| {
                let c = v as u32 as u8 as char;
                if c.is_control() { None } else { Some(c) }
            })
            .collect();

        std::fs::write(&path, &text)
            .with_context(|| format!("failed to write text: {}", path.display()))?;

        tracing::info!("saved text: {}", path.display());
        Ok(())
    }
}
