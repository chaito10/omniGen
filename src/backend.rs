//! Candle backend management.

use crate::device;
use crate::traits::Backend;
use anyhow::Result;
use candle_core::Device;
use std::sync::Arc;
use tracing::info;

/// The Candle compute backend.
pub struct CandleBackend {
    device: Device,
    device_name: String,
}

impl CandleBackend {
    pub fn auto() -> Result<Self> {
        let info = device::select_device("auto")?;
        Ok(Self {
            device: info.device,
            device_name: info.name,
        })
    }

    pub fn with_preference(preference: &str) -> Result<Self> {
        let info = device::select_device(preference)?;
        Ok(Self {
            device: info.device,
            device_name: info.name,
        })
    }

    pub fn health_check(&self) -> Result<()> {
        device::health_check(&self.device)
    }
}

impl Backend for CandleBackend {
    fn init() -> Result<Self> {
        Self::auto()
    }

    fn device(&self) -> &Device {
        &self.device
    }

    fn device_name(&self) -> &str {
        &self.device_name
    }

    fn vram_available(&self) -> Result<u64> {
        Ok(0)
    }

    fn vram_total(&self) -> Result<u64> {
        Ok(0)
    }

    fn health_check(&self) -> Result<()> {
        device::health_check(&self.device)
    }
}

/// Shared backend reference type.
pub type SharedBackend = Arc<CandleBackend>;

/// Create a shared backend with the given device preference.
pub fn create_backend(preference: &str) -> Result<SharedBackend> {
    let backend = CandleBackend::with_preference(preference)?;
    info!("backend initialized: {}", backend.device_name());
    Ok(Arc::new(backend))
}
