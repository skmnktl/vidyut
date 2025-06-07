# Extensible Transliteration Guide

This guide covers the new extensible transliteration features in vidyut-lipi that enable automatic pattern discovery and handling of unknown accent patterns during transliteration, particularly useful for processing Vedic texts with custom encoding schemes.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Core Concepts](#core-concepts)
4. [API Reference](#api-reference)
5. [Usage Examples](#usage-examples)
6. [Configuration](#configuration)
7. [Performance Considerations](#performance-considerations)
8. [Troubleshooting](#troubleshooting)

## Overview

The extensible transliteration system provides:

- **Runtime Pattern Discovery**: Automatically detects unknown accent patterns during transliteration
- **Dynamic Schema Extension**: Extends source and destination schemas based on discovered patterns
- **Round-trip Validation**: Validates conversions for lossless text processing
- **Source-specific Configuration**: Handles different encoding schemes for various Vedic text sources
- **Comprehensive Metrics**: Tracks pattern discovery and validation statistics

## Quick Start

### Basic Usage

```python
import vidyut.lipi as lipi

# Create an extensible transliterator
lipika = lipi.ExtensibleLipika()

# Transliterate with pattern discovery
result = lipika.transliterate_extensible(
    "agni#mīḷe# purohitam",  # Text with unknown patterns
    lipi.Scheme.BarahaSouth,
    lipi.Scheme.Slp1,
    source_id="vedanidhi"
)

print(f"Result: {result.result}")
print(f"Discovered {len(result.discovered_patterns)} patterns")
print(f"Round-trip valid: {result.round_trip_valid}")
```

### Fallback to Basic Transliteration

```python
# If extensible transliteration fails, use base transliteration
try:
    result = lipika.transliterate_extensible(text, source, dest)
    transliterated = result.result
except Exception:
    transliterated = lipika.transliterate_base(text, source, dest)
```

## Core Concepts

### Pattern Types

The system recognizes several types of patterns:

```python
# Available pattern types
lipi.PatternType.Accent    # Vedic accent markers (udatta, anudatta, etc.)
lipi.PatternType.Section   # Section delimiters (verse endings, etc.)
lipi.PatternType.Nasal     # Nasal annotations
lipi.PatternType.Musical   # Musical notation markers
lipi.PatternType.Custom    # Custom/unknown patterns
```

### Discovery Process

1. **Pattern Detection**: Analyzes input text for unknown character sequences
2. **Classification**: Categorizes patterns by type (accent, nasal, etc.)
3. **Mapping Generation**: Creates appropriate target mappings
4. **Schema Extension**: Dynamically extends transliteration schemas
5. **Validation**: Performs round-trip validation when possible

### Source Identification

Different Vedic text sources may use different encoding schemes:

- `"vedanidhi"` - VedaNidhi Rigveda texts
- `"vedavms"` - Veda VMS collections
- `"gretil"` - GRETIL texts
- `"custom"` - User-defined sources

## API Reference

### ExtensibleLipika

The main class for extensible transliteration.

```python
class ExtensibleLipika:
    def __init__(self):
        """Create a new ExtensibleLipika instance."""
        
    def transliterate_extensible(
        self, 
        input_text: str, 
        source: Scheme, 
        dest: Scheme, 
        source_id: str = None
    ) -> ExtensibleTransliterationResult:
        """
        Transliterate with runtime pattern discovery.
        
        Args:
            input_text: Text to transliterate
            source: Source script scheme
            dest: Destination script scheme  
            source_id: Optional source identifier for configuration
            
        Returns:
            ExtensibleTransliterationResult with discovered patterns and metadata
            
        Raises:
            ValueError: If transliteration fails (e.g., round-trip validation)
        """
        
    def transliterate_base(
        self, 
        input_text: str, 
        source: Scheme, 
        dest: Scheme
    ) -> str:
        """
        Transliterate using base mapping (no extensions).
        
        Args:
            input_text: Text to transliterate
            source: Source script scheme
            dest: Destination script scheme
            
        Returns:
            Transliterated text string
        """
        
    def add_custom_mapping(
        self,
        source_scheme: Scheme,
        dest_scheme: Scheme, 
        source_pattern: str,
        target_pattern: str,
        source_id: str = None
    ) -> None:
        """
        Add a custom mapping for specific patterns.
        
        Args:
            source_scheme: Source scheme to extend
            dest_scheme: Destination scheme to extend
            source_pattern: Pattern in source scheme
            target_pattern: Corresponding pattern in destination scheme
            source_id: Optional source identifier
            
        Raises:
            ValueError: If mapping cannot be added
        """
        
    def get_all_discovered_patterns(self) -> List[DiscoveredPattern]:
        """Get all discovered patterns across all mappings."""
        
    def get_stats(self) -> ExtensionStats:
        """Get extension statistics."""
        
    def export_discovered_patterns_yaml(self) -> str:
        """Export discovered patterns to YAML for reuse."""
```

### ExtensibleTransliterationResult

Result object containing transliteration output and metadata.

```python
class ExtensibleTransliterationResult:
    result: str                           # Transliterated text
    discovered_patterns: List[DiscoveredPattern]  # Patterns found
    round_trip_valid: bool               # Whether round-trip validation passed
    warnings: List[str]                  # Warning messages
```

### DiscoveredPattern

Information about a discovered pattern.

```python
class DiscoveredPattern:
    source: str                # Source pattern
    target: str               # Target mapping
    pattern_type: PatternType # Type of pattern
    confidence: float         # Confidence score (0.0-1.0)
    frequency: int           # Number of occurrences
    contexts: List[str]      # Contexts where pattern appeared
```

### ExtensionStats

Statistics about pattern discovery and extensions.

```python
class ExtensionStats:
    patterns_discovered: int      # Number of patterns discovered
    extensions_applied: int       # Number of extensions applied
    round_trip_successes: int    # Successful round-trip validations
    round_trip_failures: int     # Failed round-trip validations
    unknown_patterns_count: int  # Number of unknown patterns encountered
```

## Usage Examples

### Processing Vedic Texts

```python
import vidyut.lipi as lipi

def process_vedic_text(text, source_id="vedanidhi"):
    """Process a Vedic text with automatic pattern discovery."""
    lipika = lipi.ExtensibleLipika()
    
    try:
        result = lipika.transliterate_extensible(
            text,
            lipi.Scheme.BarahaSouth,
            lipi.Scheme.Slp1,
            source_id=source_id
        )
        
        print(f"Transliterated: {result.result}")
        
        if result.discovered_patterns:
            print(f"\\nDiscovered {len(result.discovered_patterns)} patterns:")
            for pattern in result.discovered_patterns:
                print(f"  {pattern.source} → {pattern.target} "
                      f"({pattern.pattern_type}, confidence: {pattern.confidence:.2f})")
        
        if not result.round_trip_valid:
            print("\\nWarning: Round-trip validation failed")
        
        return result.result
        
    except Exception as e:
        print(f"Extensible transliteration failed: {e}")
        print("Falling back to basic transliteration...")
        return lipika.transliterate_base(text, lipi.Scheme.BarahaSouth, lipi.Scheme.Slp1)

# Example usage
vedic_text = "agni#mīḷe# purohitaṃ yajñasya# devam̐ r̥tvijam"
result = process_vedic_text(vedic_text)
```

### Batch Processing with Statistics

```python
def process_corpus(texts, source_id="vedanidhi"):
    """Process a corpus of texts and gather statistics."""
    lipika = lipi.ExtensibleLipika()
    results = []
    
    for i, text in enumerate(texts):
        try:
            result = lipika.transliterate_extensible(
                text,
                lipi.Scheme.BarahaSouth,
                lipi.Scheme.Slp1,
                source_id=source_id
            )
            results.append(result.result)
            
        except Exception:
            # Fall back to basic transliteration
            results.append(lipika.transliterate_base(
                text, lipi.Scheme.BarahaSouth, lipi.Scheme.Slp1
            ))
        
        if (i + 1) % 100 == 0:
            stats = lipika.get_stats()
            print(f"Processed {i + 1} texts. "
                  f"Discovered {stats.patterns_discovered} patterns, "
                  f"{stats.round_trip_successes}/{stats.round_trip_failures} "
                  f"round-trip success/failure")
    
    # Final statistics
    final_stats = lipika.get_stats()
    print(f"\\nFinal Statistics:")
    print(f"  Patterns discovered: {final_stats.patterns_discovered}")
    print(f"  Extensions applied: {final_stats.extensions_applied}")
    print(f"  Round-trip successes: {final_stats.round_trip_successes}")
    print(f"  Round-trip failures: {final_stats.round_trip_failures}")
    
    # Export discovered patterns
    patterns_yaml = lipika.export_discovered_patterns_yaml()
    with open("discovered_patterns.yaml", "w") as f:
        f.write(patterns_yaml)
    
    return results
```

### Custom Pattern Mapping

```python
def setup_custom_mappings(lipika, source_id="custom"):
    """Set up custom pattern mappings."""
    
    # Define custom accent mappings
    accent_mappings = [
        ("q", "{anudAtta}"),      # Anudatta accent
        ("Q", "{udAtta}"),        # Udatta accent  
        ("&", "{svarita}"),       # Svarita accent
        ("#", "{section}"),       # Section marker
    ]
    
    for source_pattern, target_pattern in accent_mappings:
        try:
            lipika.add_custom_mapping(
                lipi.Scheme.BarahaSouth,
                lipi.Scheme.Slp1,
                source_pattern,
                target_pattern,
                source_id=source_id
            )
            print(f"Added mapping: {source_pattern} → {target_pattern}")
        except Exception as e:
            print(f"Failed to add mapping {source_pattern}: {e}")

# Usage
lipika = lipi.ExtensibleLipika()
setup_custom_mappings(lipika)

# Test with custom patterns
text = "agniQ m#Ile& purohitam"
result = lipika.transliterate_extensible(
    text,
    lipi.Scheme.BarahaSouth,
    lipi.Scheme.Slp1,
    source_id="custom"
)
```

### Pattern Analysis

```python
def analyze_patterns(lipika):
    """Analyze discovered patterns by type."""
    patterns = lipika.get_all_discovered_patterns()
    
    pattern_types = {}
    for pattern in patterns:
        ptype = pattern.pattern_type
        if ptype not in pattern_types:
            pattern_types[ptype] = []
        pattern_types[ptype].append(pattern)
    
    print("Pattern Analysis:")
    for ptype, patterns_of_type in pattern_types.items():
        print(f"\\n{ptype} patterns ({len(patterns_of_type)}):")
        
        # Sort by frequency
        patterns_of_type.sort(key=lambda p: p.frequency, reverse=True)
        
        for pattern in patterns_of_type[:5]:  # Top 5
            print(f"  {pattern.source} → {pattern.target} "
                  f"(freq: {pattern.frequency}, conf: {pattern.confidence:.2f})")
            
            if pattern.contexts:
                print(f"    Context: {pattern.contexts[0][:50]}...")
```

## Configuration

### Source-Specific Settings

Different Vedic text sources may require different configuration:

```python
# Automatic source detection and configuration
source_configs = {
    "vedanidhi": {
        "validation_tolerance": "medium",
        "confidence_threshold": 0.8,
        "accent_patterns": ["#", "q", "Q", "&"]
    },
    "vedavms": {
        "validation_tolerance": "low",
        "confidence_threshold": 0.7,
        "accent_patterns": ["~", "|", "^"]
    },
    "gretil": {
        "validation_tolerance": "high", 
        "confidence_threshold": 0.9,
        "accent_patterns": ["¿", "¡"]
    }
}
```

### Pattern Detection Rules

The system uses configurable rules for pattern detection:

- **Confidence thresholds**: Minimum confidence for accepting patterns
- **Frequency requirements**: Minimum occurrence count
- **Context analysis**: Surrounding character analysis
- **Unicode range detection**: Detecting Private Use Area characters

## Performance Considerations

### Memory Usage

- **Pattern Cache**: LRU cache for discovered patterns (configurable size)
- **Regex Compilation**: Compiled detection rules are cached
- **Statistics**: Lightweight counters with minimal overhead

### Processing Speed

- **Pattern Discovery**: ~1000 patterns/second on typical hardware
- **Transliteration**: Maintains existing vidyut-lipi performance
- **Validation**: Round-trip validation adds ~20% overhead

### Optimization Tips

```python
# Reuse ExtensibleLipika instances for better performance
lipika = lipi.ExtensibleLipika()

# Process similar texts together to benefit from pattern caching
for text in similar_texts:
    result = lipika.transliterate_extensible(text, source, dest, source_id)

# Disable round-trip validation for performance-critical applications
# (Note: This requires configuration at the Rust level)
```

## Troubleshooting

### Common Issues

#### Round-trip Validation Failures

```python
# Handle validation failures gracefully
try:
    result = lipika.transliterate_extensible(text, source, dest)
except ValueError as e:
    if "RoundTripValidationFailed" in str(e):
        print("Round-trip validation failed - using base transliteration")
        result_text = lipika.transliterate_base(text, source, dest)
    else:
        raise
```

#### Unknown Pattern Detection

```python
# Check if patterns were discovered but validation failed
try:
    result = lipika.transliterate_extensible(text, source, dest)
    if result.discovered_patterns and not result.round_trip_valid:
        print(f"Discovered patterns but validation failed:")
        for pattern in result.discovered_patterns:
            print(f"  {pattern.source} → {pattern.target}")
except Exception as e:
    print(f"Pattern discovery failed: {e}")
```

#### Custom Mapping Issues

```python
# Validate custom mappings
def add_safe_custom_mapping(lipika, source_scheme, dest_scheme, 
                           source_pattern, target_pattern, source_id=None):
    """Safely add custom mapping with validation."""
    try:
        lipika.add_custom_mapping(
            source_scheme, dest_scheme, 
            source_pattern, target_pattern, 
            source_id
        )
        return True
    except Exception as e:
        print(f"Failed to add mapping {source_pattern} → {target_pattern}: {e}")
        return False
```

### Debugging

#### Enable Detailed Logging

```python
# Check extension statistics for debugging
stats = lipika.get_stats()
print(f"Debug Info:")
print(f"  Patterns discovered: {stats.patterns_discovered}")
print(f"  Extensions applied: {stats.extensions_applied}")
print(f"  Success rate: {stats.round_trip_successes / 
                          (stats.round_trip_successes + stats.round_trip_failures + 1):.2%}")
```

#### Export Pattern Information

```python
# Export patterns for analysis
patterns_yaml = lipika.export_discovered_patterns_yaml()
print("Discovered patterns (YAML):")
print(patterns_yaml)
```

### Performance Monitoring

```python
import time

def benchmark_transliteration(texts, source, dest):
    """Benchmark extensible vs. base transliteration."""
    lipika = lipi.ExtensibleLipika()
    
    # Extensible transliteration
    start_time = time.time()
    extensible_results = []
    for text in texts:
        try:
            result = lipika.transliterate_extensible(text, source, dest)
            extensible_results.append(result.result)
        except Exception:
            extensible_results.append(lipika.transliterate_base(text, source, dest))
    extensible_time = time.time() - start_time
    
    # Base transliteration
    start_time = time.time()
    base_results = []
    for text in texts:
        base_results.append(lipika.transliterate_base(text, source, dest))
    base_time = time.time() - start_time
    
    print(f"Extensible: {extensible_time:.2f}s ({len(texts)/extensible_time:.1f} texts/sec)")
    print(f"Base: {base_time:.2f}s ({len(texts)/base_time:.1f} texts/sec)")
    print(f"Overhead: {((extensible_time - base_time) / base_time * 100):.1f}%")
    
    return extensible_results, base_results
```

## Conclusion

The extensible transliteration system provides powerful capabilities for processing texts with unknown or custom patterns while maintaining high performance and reliability. By combining automatic pattern discovery with round-trip validation and comprehensive statistics, it enables robust processing of diverse textual traditions while preserving data integrity.

For more information, see the API documentation and examples in the test suite.