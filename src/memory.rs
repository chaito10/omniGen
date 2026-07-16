//! Memory management for VRAM and system RAM.
//!
//! Tracks memory usage across loaded models and provides
//! automatic unloading when memory pressure is detected.

use anyhow::Result;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{debug, info, warn};

/// The memory manager tracks and controls memory usage.
pub struct MemoryManager {
    /// Currently allocated bytes.
    allocated: AtomicUsize,
    /// Memory budget in bytes (0 = unlimited).
    budget: AtomicUsize,
    /// Whether auto-unload is enabled.
    auto_unload: RwLock<bool>,
    /// Maximum VRAM usage fraction (0.0 - 1.0).
    max_vram_fraction: RwLock<f32>,
}

impl MemoryManager {
    /// Create a new memory manager with the given budget.
    pub fn new(budget_bytes: usize, auto_unload: bool, max_vram_fraction: f32) -> Self {
        Self {
            allocated: AtomicUsize::new(0),
            budget: AtomicUsize::new(budget_bytes),
            auto_unload: RwLock::new(auto_unload),
            max_vram_fraction: RwLock::new(max_vram_fraction),
        }
    }

    /// Reserve memory for a model load.
    pub fn reserve(&self, bytes: usize) -> Result<()> {
        let current = self.allocated.load(Ordering::Relaxed);
        let budget = self.budget.load(Ordering::Relaxed);

        if budget > 0 && current + bytes > budget {
            warn!(
                "memory reservation failed: requested {} bytes, {} allocated, {} budget",
                bytes, current, budget
            );
            anyhow::bail!(
                "insufficient memory: need {} bytes, {} available",
                bytes,
                budget.saturating_sub(current)
            );
        }

        self.allocated.fetch_add(bytes, Ordering::Relaxed);
        debug!(
            "reserved {} bytes (total: {})",
            bytes,
            self.allocated.load(Ordering::Relaxed)
        );
        Ok(())
    }

    /// Release previously reserved memory.
    pub fn release(&self, bytes: usize) {
        self.allocated.fetch_sub(bytes, Ordering::Relaxed);
        debug!(
            "released {} bytes (total: {})",
            bytes,
            self.allocated.load(Ordering::Relaxed)
        );
    }

    /// Get the currently allocated memory in bytes.
    pub fn allocated_bytes(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Get the memory budget in bytes.
    pub fn budget_bytes(&self) -> usize {
        self.budget.load(Ordering::Relaxed)
    }

    /// Check if memory usage is high enough to warrant unloading models.
    pub fn should_unload(&self) -> bool {
        if !*self.auto_unload.read() {
            return false;
        }

        let current = self.allocated.load(Ordering::Relaxed);
        let budget = self.budget.load(Ordering::Relaxed);

        if budget == 0 {
            return false;
        }

        let fraction = current as f32 / budget as f32;
        let max_fraction = *self.max_vram_fraction.read();

        fraction > max_fraction
    }

    /// Get memory usage as a fraction (0.0 - 1.0).
    pub fn usage_fraction(&self) -> f32 {
        let current = self.allocated.load(Ordering::Relaxed);
        let budget = self.budget.load(Ordering::Relaxed);

        if budget == 0 {
            0.0
        } else {
            current as f32 / budget as f32
        }
    }

    /// Get a human-readable memory usage string.
    pub fn usage_string(&self) -> String {
        let allocated = self.allocated.load(Ordering::Relaxed);
        let budget = self.budget.load(Ordering::Relaxed);

        if budget == 0 {
            format!("{} B (unlimited)", format_bytes(allocated))
        } else {
            format!(
                "{} / {} ({:.1}%)",
                format_bytes(allocated),
                format_bytes(budget),
                self.usage_fraction() * 100.0
            )
        }
    }

    /// Update the memory budget.
    pub fn set_budget(&self, bytes: usize) {
        self.budget.store(bytes, Ordering::Relaxed);
        info!("memory budget set to {}", format_bytes(bytes));
    }

    /// Enable or disable auto-unload.
    pub fn set_auto_unload(&self, enabled: bool) {
        *self.auto_unload.write() = enabled;
    }

    /// Set the maximum VRAM usage fraction.
    pub fn set_max_fraction(&self, fraction: f32) {
        *self.max_vram_fraction.write() = fraction.clamp(0.0, 1.0);
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new(0, true, 0.8)
    }
}

/// Format a byte count as a human-readable string.
pub fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    const GB: usize = 1024 * MB;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }

    #[test]
    fn test_reserve_and_release() {
        let mem = MemoryManager::new(1000, false, 0.8);
        assert!(mem.reserve(500).is_ok());
        assert_eq!(mem.allocated_bytes(), 500);
        mem.release(500);
        assert_eq!(mem.allocated_bytes(), 0);
    }

    #[test]
    fn test_budget_exceeded() {
        let mem = MemoryManager::new(1000, false, 0.8);
        assert!(mem.reserve(500).is_ok());
        assert!(mem.reserve(600).is_err());
        mem.release(500);
    }
}
