// llm-benchmark-types/src/hardware.rs

use serde::{Deserialize, Serialize};

/// Hardware configuration used for benchmark runs
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct HardwareConfig {
    /// GPU model (e.g., "RTX 4090", "CPU Only")
    pub gpu_model: String,

    /// GPU memory in GB (0 for CPU-only)
    pub gpu_memory_gb: i32,

    /// CPU model (e.g., "AMD Threadripper 1950X")
    pub cpu_model: String,

    /// CPU architecture (e.g., "Zen1", "Zen2", "x86_64")
    pub cpu_arch: String,

    /// RAM amount in GB
    pub ram_gb: i32,

    /// RAM type (e.g., "DDR4", "DDR5")
    pub ram_type: String,

    /// Virtualization type if applicable (e.g., "KVM", "Docker")
    pub virtualization_type: Option<String>,

    /// List of optimizations applied (e.g., ["pci_passthrough", "hugepages_1gb"])
    pub optimizations: Vec<String>,
}

/// Simplified hardware type for filtering and display
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HardwareType {
    /// GPU-accelerated inference
    Gpu,
    /// CPU-only inference
    CpuOnly,
}

impl HardwareConfig {
    /// Create a new hardware configuration
    pub fn new(
        gpu_model: String,
        gpu_memory_gb: i32,
        cpu_model: String,
        cpu_arch: String,
        ram_gb: i32,
        ram_type: String,
    ) -> Self {
        Self {
            gpu_model,
            gpu_memory_gb,
            cpu_model,
            cpu_arch,
            ram_gb,
            ram_type,
            virtualization_type: None,
            optimizations: Vec::new(),
        }
    }

    /// Create a CPU-only configuration
    pub fn cpu_only(cpu_model: String, cpu_arch: String, ram_gb: i32, ram_type: String) -> Self {
        Self::new(
            "CPU Only".to_string(),
            0,
            cpu_model,
            cpu_arch,
            ram_gb,
            ram_type,
        )
    }

    /// Add an optimization to the configuration
    pub fn with_optimization(mut self, optimization: String) -> Self {
        self.optimizations.push(optimization);
        self
    }

    /// Set virtualization type
    pub fn with_virtualization(mut self, virt_type: String) -> Self {
        self.virtualization_type = Some(virt_type);
        self
    }

    /// Determine the hardware type
    pub fn hardware_type(&self) -> HardwareType {
        if self.gpu_model == "CPU Only" || self.gpu_memory_gb == 0 {
            HardwareType::CpuOnly
        } else {
            HardwareType::Gpu
        }
    }

    /// Generate a short hardware summary string
    pub fn summary(&self) -> String {
        format!("{} / {}", self.gpu_model, self.cpu_arch)
    }

    /// Check if this configuration supports a given memory requirement
    pub fn supports_memory_gb(&self, required_gb: i32) -> bool {
        match self.hardware_type() {
            HardwareType::Gpu => self.gpu_memory_gb >= required_gb,
            HardwareType::CpuOnly => self.ram_gb >= required_gb,
        }
    }

    /// Get effective memory for model loading (GPU memory or RAM)
    pub fn effective_memory_gb(&self) -> i32 {
        match self.hardware_type() {
            HardwareType::Gpu => self.gpu_memory_gb,
            HardwareType::CpuOnly => self.ram_gb,
        }
    }

    /// Check if running in virtualized environment
    pub fn is_virtualized(&self) -> bool {
        self.virtualization_type.is_some()
    }

    /// Check if a specific optimization is enabled
    pub fn has_optimization(&self, optimization: &str) -> bool {
        self.optimizations.iter().any(|opt| opt == optimization)
    }
}

impl std::fmt::Display for HardwareConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}GB) + {} {} ({}GB {})",
            self.gpu_model,
            self.gpu_memory_gb,
            self.cpu_model,
            self.cpu_arch,
            self.ram_gb,
            self.ram_type
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_type_detection() {
        let gpu_config = HardwareConfig::new(
            "RTX 4090".to_string(),
            24,
            "AMD Threadripper".to_string(),
            "Zen2".to_string(),
            64,
            "DDR4".to_string(),
        );
        assert_eq!(gpu_config.hardware_type(), HardwareType::Gpu);

        let cpu_config = HardwareConfig::cpu_only(
            "Intel i9-13900K".to_string(),
            "x86_64".to_string(),
            64,
            "DDR5".to_string(),
        );
        assert_eq!(cpu_config.hardware_type(), HardwareType::CpuOnly);
    }

    #[test]
    fn test_memory_requirements() {
        let config = HardwareConfig::new(
            "RTX 4090".to_string(),
            24,
            "AMD Threadripper".to_string(),
            "Zen2".to_string(),
            64,
            "DDR4".to_string(),
        );

        assert!(config.supports_memory_gb(20));
        assert!(!config.supports_memory_gb(32));
        assert_eq!(config.effective_memory_gb(), 24);
    }
}
