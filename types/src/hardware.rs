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

    /// RAM amount in GB (optional - may not be available from all sources)
    pub ram_gb: Option<i32>,

    /// RAM type (e.g., "DDR4", "DDR5") (optional - may not be available from all sources)
    pub ram_type: Option<String>,

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

/// Hardware category for filtering
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HardwareCategory {
    /// Consumer GPU (e.g., RTX 4090, RTX 3090)
    ConsumerGpu,
    /// Consumer CPU (e.g., Ryzen, Intel Core)
    ConsumerCpu,
    /// Datacenter GPU (e.g., A100, H100, L4, L40)
    DatacenterGpu,
    /// Datacenter CPU (e.g., Xeon, EPYC)
    DatacenterCpu,
}

impl HardwareCategory {
    /// Get a human-readable label for the category
    pub fn label(&self) -> &'static str {
        match self {
            HardwareCategory::ConsumerGpu => "Consumer GPU",
            HardwareCategory::ConsumerCpu => "Consumer CPU",
            HardwareCategory::DatacenterGpu => "Datacenter GPU",
            HardwareCategory::DatacenterCpu => "Datacenter CPU",
        }
    }
}

impl HardwareConfig {
    /// Create a new hardware configuration
    pub fn new(
        gpu_model: String,
        gpu_memory_gb: i32,
        cpu_model: String,
        cpu_arch: String,
        ram_gb: Option<i32>,
        ram_type: Option<String>,
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
    pub fn cpu_only(cpu_model: String, cpu_arch: String, ram_gb: Option<i32>, ram_type: Option<String>) -> Self {
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
            HardwareType::CpuOnly => self.ram_gb.map_or(false, |ram| ram >= required_gb),
        }
    }

    /// Get effective memory for model loading (GPU memory or RAM)
    pub fn effective_memory_gb(&self) -> Option<i32> {
        match self.hardware_type() {
            HardwareType::Gpu => Some(self.gpu_memory_gb),
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

    /// Determine the hardware category based on GPU and CPU model
    pub fn hardware_category(&self) -> HardwareCategory {
        // Check GPU first
        if self.gpu_model.contains("RTX") || self.gpu_model.contains("GTX") {
            HardwareCategory::ConsumerGpu
        } else if self.gpu_model.contains("A100") || self.gpu_model.contains("H100") 
            || self.gpu_model.contains("L4") || self.gpu_model.contains("L40")
            || self.gpu_model.contains("V100") || self.gpu_model.contains("T4") {
            HardwareCategory::DatacenterGpu
        } else if self.gpu_model == "CPU Only" || self.gpu_model == "N/A" || self.gpu_memory_gb == 0 {
            // CPU only - check CPU model
            if self.cpu_model.contains("Xeon") || self.cpu_model.contains("EPYC") {
                HardwareCategory::DatacenterCpu
            } else {
                HardwareCategory::ConsumerCpu
            }
        } else {
            // Unknown GPU, default to consumer
            HardwareCategory::ConsumerGpu
        }
    }
}

impl std::fmt::Display for HardwareConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(ram_gb), Some(ram_type)) = (self.ram_gb, &self.ram_type) {
            write!(
                f,
                "{} ({}GB) + {} {} ({}GB {})",
                self.gpu_model,
                self.gpu_memory_gb,
                self.cpu_model,
                self.cpu_arch,
                ram_gb,
                ram_type
            )
        } else {
            write!(
                f,
                "{} ({}GB) + {} {}",
                self.gpu_model,
                self.gpu_memory_gb,
                self.cpu_model,
                self.cpu_arch
            )
        }
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
            Some(64),
            Some("DDR4".to_string()),
        );
        assert_eq!(gpu_config.hardware_type(), HardwareType::Gpu);

        let cpu_config = HardwareConfig::cpu_only(
            "Intel i9-13900K".to_string(),
            "x86_64".to_string(),
            Some(64),
            Some("DDR5".to_string()),
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
            Some(64),
            Some("DDR4".to_string()),
        );

        assert!(config.supports_memory_gb(20));
        assert!(!config.supports_memory_gb(32));
        assert_eq!(config.effective_memory_gb(), Some(24));
    }
}
