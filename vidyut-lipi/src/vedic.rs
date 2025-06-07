/*!
Vedic text processing extensions for vidyut-lipi.

This module provides specialized support for handling vedic texts with custom accent
encoding schemes. It extends the standard vidyut-lipi transliteration system with:

- Extended SLP1 format for canonical vedic accent representation
- Pattern discovery for unknown accent encoding schemes  
- Bijective mapping system for lossless roundtrip conversion
- Schema inheritance and extension system
- Integration with udapaana corpus aggregation

# Example Usage

```rust
use vidyut_lipi::vedic::{VedicSchemaManager, PatternDiscovery};

// Discover patterns in unknown vedic text
let discovery = PatternDiscovery::new();
let patterns = discovery.analyze_file("rigveda_mandal1.txt")?;

// Create or update schema
let mut manager = VedicSchemaManager::new();
let schema_id = manager.create_schema("rv_mandal1_baraha", patterns)?;

// Convert to extended SLP1
let schema = manager.get_schema(&schema_id)?;
let extended_slp1 = schema.to_extended_slp1(&source_text)?;

// Bijective conversion back to source
let reconstructed = schema.from_extended_slp1(&extended_slp1)?;
assert_eq!(source_text, reconstructed);
```
*/

pub mod extended_slp1;
pub mod pattern_discovery;
pub mod schema;
pub mod storage;

pub use extended_slp1::ExtendedSlp1;
pub use pattern_discovery::{PatternDiscovery, DiscoveredPattern, PatternConfidence};
pub use schema::{VedicSchema, VedicSchemaManager, SchemaConfig};
pub use storage::SchemaStorage;

use crate::errors::LipiError;

/// Result type for vedic processing operations.
pub type VedicResult<T> = Result<T, VedicError>;

/// Errors that can occur during vedic text processing.
#[derive(Debug, Clone, PartialEq)]
pub enum VedicError {
    /// Schema not found.
    SchemaNotFound(String),
    
    /// Pattern discovery failed.
    PatternDiscoveryFailed(String),
    
    /// Bijective mapping validation failed.
    BijectivityError { 
        original: String, 
        reconstructed: String,
        extended_form: String,
    },
    
    /// Schema validation failed.
    ValidationError(String),
    
    /// Database operation failed.
    StorageError(String),
    
    /// TOML parsing error.
    ConfigError(String),
    
    /// General lipi error.
    LipiError(LipiError),
}

impl From<LipiError> for VedicError {
    fn from(err: LipiError) -> Self {
        VedicError::LipiError(err)
    }
}

impl std::fmt::Display for VedicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VedicError::SchemaNotFound(id) => write!(f, "Schema not found: {}", id),
            VedicError::PatternDiscoveryFailed(msg) => write!(f, "Pattern discovery failed: {}", msg),
            VedicError::BijectivityError { original, reconstructed, extended_form } => {
                write!(f, "Bijectivity error: original '{}' != reconstructed '{}' (extended: '{}')", 
                       original, reconstructed, extended_form)
            },
            VedicError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            VedicError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            VedicError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            VedicError::LipiError(err) => write!(f, "Lipi error: {}", err),
        }
    }
}

impl std::error::Error for VedicError {}