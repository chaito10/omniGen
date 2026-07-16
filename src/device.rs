//! Device detection and management.
//!
//! Handles automatic device selection with CUDA preferred,
//! falling back to CPU when no GPU is available.

use anyhow::{Context, Result};
use candle_core::{Device, DeviceLocation};
use tracing::{info, warn};

/// Detected compute device information.
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    /// The Candle device handle.
    pub device: Device,
    /// Human-readable device name.
    pub name: String,
    /// Available VRAM in bytes (0 for CPU).
    pub vram_bytes: u64,
    /// Whether this is a GPU device.
    pub is_gpu: bool,
}

/// Try to initialize GPU device, falling back to CPU.
pub fn select_device(preference: &str) -> Result<DeviceInfo> {
    match preference {
        "vulkan" | "gpu" | "auto" => {
            // Try CUDA first
            if let Ok(device) = Device::cuda_if_available(0) {
                let name = get_device_name(&device);
                let vram = get_vram_size(&device).unwrap_or(0);
                info!("selected device: {} (GPU)", name);
                return Ok(DeviceInfo {
                    device,
                    name,
                    vram_bytes: vram,
                    is_gpu: true,
                });
            }
            // Try Metal on macOS
            #[cfg(target_os = "macos")]
            if let Ok(device) = Device::new_metal(0) {
                let name = get_device_name(&device);
                info!("selected device: {} (Metal)", name);
                return Ok(DeviceInfo {
                    device,
                    name,
                    vram_bytes: 0,
                    is_gpu: true,
                });
            }
            if preference == "vulkan" || preference == "gpu" {
                warn!("GPU requested but unavailable, falling back to CPU");
            }
            init_cpu()
        }
        "cpu" => init_cpu(),
        _ => {
            warn!("unknown device preference '{}', trying auto", preference);
            select_device("auto")
        }
    }
}

fn init_cpu() -> Result<DeviceInfo> {
    let device = Device::Cpu;
    let name = "CPU".to_string();
    info!("selected device: CPU");
    Ok(DeviceInfo {
        device,
        name,
        vram_bytes: 0,
        is_gpu: false,
    })
}

fn get_device_name(device: &Device) -> String {
    match device.location() {
        DeviceLocation::Cpu => "CPU".to_string(),
        DeviceLocation::Cuda { gpu_id } => format!("CUDA GPU {}", gpu_id),
        #[cfg(target_os = "macos")]
        DeviceLocation::Metal { gpu_id } => format!("Metal GPU {}", gpu_id),
        _ => "Unknown".to_string(),
    }
}

fn get_vram_size(_device: &Device) -> Result<u64> {
    // Candle doesn't expose direct VRAM queries
    // Return 0 as fallback
    Ok(0)
}

/// List all available devices.
pub fn list_devices() -> Vec<DeviceInfo> {
    let mut devices = Vec::new();

    // Always include CPU
    if let Ok(info) = init_cpu() {
        devices.push(info);
    }

    // Try CUDA
    if let Ok(device) = Device::cuda_if_available(0) {
        let name = get_device_name(&device);
        devices.push(DeviceInfo {
            device,
            name,
            vram_bytes: 0,
            is_gpu: true,
        });
    }

    devices
}

/// Run a device health check and report results.
pub fn health_check(device: &Device) -> Result<()> {
    let name = get_device_name(device);
    info!("health check: testing device '{}'", name);

    // Simple computation test: create a tensor and perform an operation
    let a = candle_core::Tensor::zeros((2, 2), candle_core::DType::F32, device)
        .context("failed to create test tensor")?;
    let _b = a
        .matmul(&a)
        .context("failed to run matrix multiply on device")?;

    info!("health check: device '{}' passed", name);
    Ok(())
}
