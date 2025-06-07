/// Runtime extensible mapping system for vidyut-lipi.
///
/// This module provides:
/// 1. Runtime pattern discovery during transliteration
/// 2. Dynamic schema extension for unknown accent patterns
/// 3. Round-trip validation to ensure lossless conversion
/// 4. YAML-based configuration for extension rules
/// 5. Automatic source detection for sakha-veda-source combinations

/// Core runtime extensible mapping implementation
pub mod runtime_extensible;
/// Runtime extension for adding mappings to schemes
pub mod runtime_extension;
/// Configuration loading and management
pub mod config;
/// Automatic source detection for text files
pub mod source_detection;
/// Vedic-specific extensions (currently empty)
pub mod vedic;

#[cfg(test)]
mod tests;

pub use runtime_extensible::{
    RuntimeExtensibleMapping, 
    DiscoveredPattern, 
    PatternMatcher,
    ExtensionError
};
pub use config::{
    VidyutExtensionConfig, 
    SchemeExtensionConfig, 
    SourceConfig,
    ConfigError
};
pub use source_detection::{
    SourceDetector,
    DetectionResult,
    SourceMetadata,
    DetectionError
};