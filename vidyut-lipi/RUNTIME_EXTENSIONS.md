# Runtime Extensible Mapping System for vidyut-lipi

This document describes the runtime extensible mapping system added to vidyut-lipi, which enables dynamic pattern discovery and schema extension during transliteration.

## Overview

The runtime extensible mapping system allows vidyut-lipi to:

1. **Automatically discover unknown patterns** during transliteration
2. **Extend mapping schemas on-the-fly** when encountering new accent encodings
3. **Ensure lossless round-trip conversion** through comprehensive validation
4. **Use YAML-based configuration** for extension rules and pattern definitions

## Key Features

### 1. Runtime Pattern Discovery
- Detects unknown accent patterns automatically during transliteration
- Uses configurable confidence thresholds and pattern matching rules
- Supports various pattern types: accents, nasals, musical notations, section markers

### 2. Dynamic Schema Extension
- Extends source and destination schemas based on discovered patterns
- Generates bidirectional mappings for round-trip validation
- Supports multiple extension strategies (preserve, placeholder, fail, log)

### 3. Source-Specific Configuration
- Sakha-veda-source specific configurations (e.g., Rigveda Shakala from vedanidhi)
- Automatic source detection based on file paths and content patterns
- Customizable processing rules per source

### 4. Comprehensive Validation
- Line-by-line round-trip testing
- Multiple validation tolerance levels (exact, whitespace-normalized, Unicode-normalized)
- Detailed similarity metrics and difference analysis

## Usage

### Basic Usage

```rust
use vidyut_lipi::{ExtensibleLipika, Scheme};

// Initialize with default configuration
let mut lipika = ExtensibleLipika::new();

// Transliterate with automatic pattern discovery
let result = lipika.transliterate_extensible(
    "agni#mīḷe# purohitam",  // Text with unknown accent patterns
    Scheme::BarahaSouth,
    Scheme::Slp1,
    Some("vedanidhi_rigveda_shakala")  // Source identifier
)?;

println!("Transliterated: {}", result.result);
println!("Round-trip valid: {}", result.round_trip_valid);
println!("Patterns discovered: {}", result.discovered_patterns.len());
```

### Advanced Configuration

```rust
// Load custom configuration
let lipika = ExtensibleLipika::from_config_file("custom_extensions.yaml")?;

// Add custom mapping at runtime
lipika.add_custom_mapping(
    Scheme::BarahaSouth,
    Scheme::Slp1, 
    "§",  // Custom pattern
    "{SECTION}",  // Target mapping
    Some("custom_source")
)?;
```

### Automatic Source Detection

```rust
use vidyut_lipi::extensions::SourceDetector;

let mut detector = SourceDetector::with_udapaana_config()?;

// Detect source type automatically
let detection = detector.detect_source(file_path, content)?;
println!("Detected: {} (confidence: {})", detection.scheme_name, detection.confidence);
```

## Configuration Format

Extension configurations are defined in YAML format:

```yaml
version: "1.0"

scheme_extensions:
  RigvedaShakalaVedanidhi:
    base_scheme: "BarahaSouth"
    description: "Rigveda Shakala recension from vedanidhi source"
    
    patterns:
      - name: "anudatta_q"
        pattern: "q"
        pattern_type: "Accent"
        target_mapping:
          Fixed: "{A}"
        confidence: 0.95
    
    sources:
      vedanidhi_rigveda_shakala:
        source_id: "vedanidhi_rigveda_shakala"
        known_mappings:
          "q": "{A}"
          "#": "{S}"
```

## Supported Pattern Types

- **Accent**: Vedic accent markers (anudatta, svarita, etc.)
- **Nasal**: Nasal annotations and modifications
- **Musical**: Samaveda musical notations (including PUA codes)
- **Section**: Text structure markers (kanda, ashtaka, etc.)
- **Custom**: User-defined pattern types

## Source Detection Rules

The system can automatically detect source types based on:

- **File path patterns**: `/rigveda/shakala/vedanidhi/`
- **Content markers**: `शाकलसंहिता`, `अष्टकम्`, etc.
- **Encoding features**: PUA characters, specific accent patterns
- **File characteristics**: Extension, size, format

## Round-Trip Validation

The system performs comprehensive round-trip validation:

1. **Original** → **Target** (forward conversion)
2. **Target** → **Original** (reverse conversion)  
3. **Comparison** with multiple tolerance levels
4. **Similarity scoring** using various metrics

Validation results include:
- Exact match status
- Normalized match status  
- Similarity score (0.0 to 1.0)
- Levenshtein distance
- Character difference count

## Integration with Existing Code

The extensible system is designed to be backward compatible:

```rust
// Existing code continues to work
let lipika = Lipika::new();
let result = lipika.transliterate("text", Scheme::Devanagari, Scheme::Slp1);

// Enhanced functionality available through ExtensibleLipika
let extensible_lipika = ExtensibleLipika::new();
let enhanced_result = extensible_lipika.transliterate_extensible(
    "text", Scheme::Devanagari, Scheme::Slp1, Some("source")
)?;
```

## Error Handling

The system provides comprehensive error handling:

```rust
use vidyut_lipi::extensions::ExtensionError;

match lipika.transliterate_extensible(text, from, to, source) {
    Ok(result) => {
        if !result.round_trip_valid {
            println!("Warning: Round-trip validation failed");
        }
    },
    Err(ExtensionError::UnknownPattern(pattern)) => {
        println!("Unknown pattern encountered: {}", pattern);
    },
    Err(ExtensionError::RoundTripValidationFailed { .. }) => {
        println!("Round-trip validation failed");
    },
    Err(e) => {
        println!("Extension error: {}", e);
    }
}
```

## Performance Considerations

- **Pattern caching**: Discovered patterns are cached for reuse
- **Compiled regex**: Patterns are pre-compiled for efficiency  
- **LRU cache**: Mapping cache with configurable size
- **Incremental discovery**: Patterns discovered incrementally during processing

## Testing

Comprehensive test suite includes:

```bash
# Run all tests
cargo test

# Run extension-specific tests
cargo test extensions

# Run with Vedic features
cargo test --features vedic
```

## Examples

See the `examples/` directory for complete usage examples:

- `udapaana_integration.rs`: Processing udapaana corpus data
- `pattern_discovery.rs`: Custom pattern discovery
- `round_trip_validation.rs`: Validation examples

## Migration Guide

### From Standard Lipika

```rust
// Before
let mut lipika = Lipika::new();
let result = lipika.transliterate(text, from, to);

// After  
let mut lipika = ExtensibleLipika::new();
let result = lipika.transliterate_extensible(text, from, to, None)?;
// Use result.result for the transliterated text
```

### Adding Custom Patterns

```rust
// Define in YAML configuration
// Or add at runtime:
lipika.add_custom_mapping(from, to, "pattern", "target", Some("source"))?;
```

## Contributing

When adding new extension features:

1. Update the configuration schema in `config.rs`
2. Add pattern detection rules in `source_detection.rs` 
3. Implement mapping logic in `runtime_extensible.rs`
4. Add comprehensive tests
5. Update documentation

## License

This extension system is released under the same license as vidyut-lipi (MIT).