//! Model registry and lifecycle management.
//!
//! Manages model loading, unloading, and lifecycle through a central registry
//! that supports the plugin architecture.

use crate::traits::{Model, ModelInput, ModelOutput, ModelType};
use crate::cache::TensorCache;
use crate::memory::MemoryManager;
use anyhow::Result;
use candle_core::Device;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// A registered model entry with metadata.
struct ModelEntry {
    /// The model instance (behind a RwLock for interior mutability).
    model: Box<dyn Model>,
    /// Path to the model weights file.
    _weight_path: PathBuf,
    /// Whether this model is currently loaded.
    loaded: bool,
}

/// Central model registry that manages all models.
pub struct ModelRegistry {
    /// Registered model instances keyed by model type.
    models: RwLock<HashMap<ModelType, ModelEntry>>,
    /// Base directory for model storage.
    models_dir: PathBuf,
    /// Reference to the memory manager.
    memory: Arc<MemoryManager>,
    /// Reference to the tensor cache.
    _cache: Arc<TensorCache>,
}

impl ModelRegistry {
    /// Create a new model registry with the given base directory.
    pub fn new(models_dir: PathBuf, memory: Arc<MemoryManager>, cache: Arc<TensorCache>) -> Self {
        Self {
            models: RwLock::new(HashMap::new()),
            models_dir,
            memory,
            _cache: cache,
        }
    }

    /// Register a model with the registry.
    pub fn register(&self, model: Box<dyn Model>, model_type: ModelType) -> Result<()> {
        let model_type_str = model_type.to_string();
        let weight_path = self.models_dir.join(&model_type_str);

        let entry = ModelEntry {
            model,
            _weight_path: weight_path,
            loaded: false,
        };

        self.models.write().insert(model_type, entry);
        debug!("registered model: {}", model_type_str);
        Ok(())
    }

    /// Load a model's weights from disk.
    pub fn load_model(&self, model_type: &ModelType, device: &Device) -> Result<()> {
        let mut models = self.models.write();
        let entry = models
            .get_mut(model_type)
            .ok_or_else(|| anyhow::anyhow!("model not registered: {}", model_type))?;

        if entry.loaded {
            debug!("model already loaded: {}", model_type);
            return Ok(());
        }

        // Check memory before loading
        let estimated_size = entry.model.memory_usage();
        self.memory.reserve(estimated_size)?;

        // Determine weight file path
        let filename = entry.model.huggingface_filename();
        let path = entry._weight_path.join(filename);

        if !path.exists() {
            let ext = entry.model.file_extension();
            let path_with_ext = entry._weight_path.join(format!("model{}", ext));
            if path_with_ext.exists() {
                entry.model.load(&path_with_ext, device)?;
            } else {
                self.memory.release(estimated_size);
                anyhow::bail!(
                    "model weights not found at {} or {}",
                    path.display(),
                    path_with_ext.display()
                );
            }
        } else {
            entry.model.load(&path, device)?;
        }

        entry.loaded = true;
        info!("loaded model: {} ({} bytes)", model_type, estimated_size);
        Ok(())
    }

    /// Unload a model from memory.
    pub fn unload_model(&self, model_type: &ModelType) -> Result<()> {
        let mut models = self.models.write();
        let entry = models
            .get_mut(model_type)
            .ok_or_else(|| anyhow::anyhow!("model not registered: {}", model_type))?;

        if !entry.loaded {
            return Ok(());
        }

        let mem_usage = entry.model.memory_usage();
        entry.model.unload()?;
        entry.loaded = false;
        self.memory.release(mem_usage);

        info!("unloaded model: {}", model_type);
        Ok(())
    }

    /// Run inference on a loaded model.
    pub fn run_inference(
        &self,
        model_type: &ModelType,
        input: &ModelInput,
    ) -> Result<ModelOutput> {
        let models = self.models.read();
        let entry = models
            .get(model_type)
            .ok_or_else(|| anyhow::anyhow!("model not registered: {}", model_type))?;

        if !entry.loaded {
            anyhow::bail!("model not loaded: {}", model_type);
        }

        entry.model.run(input)
    }

    /// Get the weight directory for a model.
    pub fn model_dir(&self, model_type: &ModelType) -> PathBuf {
        self.models_dir.join(model_type.to_string())
    }

    /// Check if a model's weights exist on disk.
    pub fn weights_exist(&self, model_type: &ModelType) -> bool {
        let models = self.models.read();
        if let Some(entry) = models.get(model_type) {
            let filename = entry.model.huggingface_filename();
            let path = entry._weight_path.join(filename);
            let path_ext = entry._weight_path.join(format!("model{}", entry.model.file_extension()));
            path.exists() || path_ext.exists()
        } else {
            false
        }
    }

    /// Get information about all registered models.
    pub fn list_models(&self) -> Vec<ModelInfo> {
        let models = self.models.read();
        models
            .iter()
            .map(|(model_type, entry)| ModelInfo {
                model_type: *model_type,
                name: entry.model.name().to_string(),
                loaded: entry.loaded,
                memory_usage: entry.model.memory_usage(),
                downloaded: self.weights_exist(model_type),
                repo: entry.model.huggingface_repo().to_string(),
                filename: entry.model.huggingface_filename().to_string(),
            })
            .collect()
    }

    /// Auto-unload least recently used models if memory pressure is high.
    pub fn auto_unload(&self) -> Result<()> {
        if !self.memory.should_unload() {
            return Ok(());
        }

        let models_to_unload: Vec<ModelType> = {
            let models = self.models.read();
            models
                .iter()
                .filter(|(_, entry)| entry.loaded)
                .map(|(model_type, _)| *model_type)
                .collect()
        };

        for model_type in models_to_unload {
            if let Err(e) = self.unload_model(&model_type) {
                warn!("failed to auto-unload model {}: {}", model_type, e);
            }
        }

        Ok(())
    }
}

/// Information about a registered model.
#[derive(Debug, Clone)]
pub struct ModelInfo {
    /// Model type identifier.
    pub model_type: ModelType,
    /// Human-readable name.
    pub name: String,
    /// Whether the model is currently loaded.
    pub loaded: bool,
    /// Memory usage in bytes.
    pub memory_usage: usize,
    /// Whether weights are downloaded.
    pub downloaded: bool,
    /// HuggingFace repository.
    pub repo: String,
    /// Model filename.
    pub filename: String,
}
