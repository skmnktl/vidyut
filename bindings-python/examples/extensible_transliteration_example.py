#!/usr/bin/env python3
"""
Example usage of vidyut-lipi's extensible transliteration features.

This example demonstrates how to use the ExtensibleLipika for processing
Vedic texts with unknown accent patterns.
"""

import vidyut.lipi as lipi


def basic_extensible_example():
    """Basic example of extensible transliteration."""
    print("=== Basic Extensible Transliteration ===")
    
    # Create an extensible transliterator
    lipika = lipi.ExtensibleLipika()
    
    # Text with potential unknown patterns
    vedic_text = "agni#mīḷe# purohitam"
    
    try:
        # Attempt extensible transliteration
        result = lipika.transliterate_extensible(
            vedic_text,
            lipi.Scheme.BarahaSouth,
            lipi.Scheme.Slp1,
            source_id="vedanidhi"
        )
        
        print(f"Input: {vedic_text}")
        print(f"Output: {result.result}")
        print(f"Round-trip valid: {result.round_trip_valid}")
        
        if result.discovered_patterns:
            print(f"\\nDiscovered {len(result.discovered_patterns)} patterns:")
            for pattern in result.discovered_patterns:
                print(f"  {pattern.source} → {pattern.target} "
                      f"({pattern.pattern_type}, confidence: {pattern.confidence:.2f})")
        
        if result.warnings:
            print(f"\\nWarnings: {result.warnings}")
            
    except Exception as e:
        print(f"Extensible transliteration failed: {e}")
        print("Falling back to basic transliteration...")
        
        # Fall back to basic transliteration
        basic_result = lipika.transliterate_base(
            vedic_text,
            lipi.Scheme.BarahaSouth,
            lipi.Scheme.Slp1
        )
        print(f"Basic result: {basic_result}")


def custom_mapping_example():
    """Example of adding custom pattern mappings."""
    print("\\n=== Custom Mapping Example ===")
    
    lipika = lipi.ExtensibleLipika()
    
    # Add custom mappings for specific accent patterns
    custom_mappings = [
        ("q", "{anudAtta}"),      # Anudatta accent
        ("Q", "{udAtta}"),        # Udatta accent
        ("#", "{section}"),       # Section marker
    ]
    
    print("Adding custom mappings:")
    for source_pattern, target_pattern in custom_mappings:
        try:
            lipika.add_custom_mapping(
                lipi.Scheme.BarahaSouth,
                lipi.Scheme.Slp1,
                source_pattern,
                target_pattern,
                source_id="custom"
            )
            print(f"  ✓ {source_pattern} → {target_pattern}")
        except Exception as e:
            print(f"  ✗ Failed to add {source_pattern}: {e}")
    
    # Test with custom patterns
    test_text = "agniQ m#īḷeq purohitam"
    print(f"\\nTesting with: {test_text}")
    
    try:
        result = lipika.transliterate_extensible(
            test_text,
            lipi.Scheme.BarahaSouth,
            lipi.Scheme.Slp1,
            source_id="custom"
        )
        print(f"Result: {result.result}")
    except Exception as e:
        print(f"Custom mapping failed: {e}")
        print(f"Base result: {lipika.transliterate_base(test_text, lipi.Scheme.BarahaSouth, lipi.Scheme.Slp1)}")


def statistics_example():
    """Example of gathering and analyzing statistics."""
    print("\\n=== Statistics Example ===")
    
    lipika = lipi.ExtensibleLipika()
    
    # Process multiple texts
    texts = [
        "agni#mīḷe# purohitam",
        "yajñasya# devam̐ r̥tvijam",
        "hotāram̐ ratnadhātamam"
    ]
    
    print("Processing texts:")
    for i, text in enumerate(texts, 1):
        print(f"  {i}. {text}")
        try:
            result = lipika.transliterate_extensible(
                text,
                lipi.Scheme.BarahaSouth,
                lipi.Scheme.Slp1,
                source_id="vedanidhi"
            )
            print(f"     → {result.result}")
        except Exception:
            result = lipika.transliterate_base(
                text, lipi.Scheme.BarahaSouth, lipi.Scheme.Slp1
            )
            print(f"     → {result} (basic)")
    
    # Get statistics
    stats = lipika.get_stats()
    print(f"\\nFinal Statistics:")
    print(f"  Patterns discovered: {stats.patterns_discovered}")
    print(f"  Extensions applied: {stats.extensions_applied}")
    print(f"  Round-trip successes: {stats.round_trip_successes}")
    print(f"  Round-trip failures: {stats.round_trip_failures}")
    print(f"  Unknown patterns: {stats.unknown_patterns_count}")
    
    # Show all discovered patterns
    all_patterns = lipika.get_all_discovered_patterns()
    if all_patterns:
        print(f"\\nAll discovered patterns ({len(all_patterns)}):")
        for pattern in all_patterns:
            print(f"  {pattern.source} → {pattern.target} "
                  f"(type: {pattern.pattern_type}, freq: {pattern.frequency})")


def export_patterns_example():
    """Example of exporting discovered patterns."""
    print("\\n=== Export Patterns Example ===")
    
    lipika = lipi.ExtensibleLipika()
    
    # Process some text to discover patterns
    sample_text = "agni#mīḷe# purohitam yajñasya# devam̐"
    
    try:
        lipika.transliterate_extensible(
            sample_text,
            lipi.Scheme.BarahaSouth,
            lipi.Scheme.Slp1,
            source_id="vedanidhi"
        )
    except Exception:
        pass  # Even if it fails, some patterns might be discovered
    
    # Export patterns to YAML
    try:
        yaml_export = lipika.export_discovered_patterns_yaml()
        print("Exported patterns (YAML):")
        print(yaml_export)
        
        # Save to file
        with open("discovered_patterns.yaml", "w") as f:
            f.write(yaml_export)
        print("\\n✓ Patterns saved to 'discovered_patterns.yaml'")
        
    except Exception as e:
        print(f"Export failed: {e}")


def pattern_analysis_example():
    """Example of analyzing discovered patterns by type."""
    print("\\n=== Pattern Analysis Example ===")
    
    lipika = lipi.ExtensibleLipika()
    
    # Process various texts to accumulate patterns
    sample_texts = [
        "agni#mīḷe# purohitam",
        "yajñasya& devam̐ r̥tvijam~",
        "hotāram̐Q ratnadhātamam|"
    ]
    
    for text in sample_texts:
        try:
            lipika.transliterate_extensible(
                text,
                lipi.Scheme.BarahaSouth,
                lipi.Scheme.Slp1
            )
        except Exception:
            pass
    
    # Analyze patterns by type
    all_patterns = lipika.get_all_discovered_patterns()
    
    if all_patterns:
        print(f"Pattern analysis ({len(all_patterns)} total patterns):")
        
        # Group by pattern type
        pattern_types = {}
        for pattern in all_patterns:
            ptype = pattern.pattern_type
            if ptype not in pattern_types:
                pattern_types[ptype] = []
            pattern_types[ptype].append(pattern)
        
        for ptype, patterns in pattern_types.items():
            print(f"\\n{ptype} patterns ({len(patterns)}):")
            # Sort by frequency
            patterns.sort(key=lambda p: p.frequency, reverse=True)
            for pattern in patterns[:3]:  # Top 3
                print(f"  {pattern.source} → {pattern.target} "
                      f"(freq: {pattern.frequency}, conf: {pattern.confidence:.2f})")
    else:
        print("No patterns discovered in this session.")


def main():
    """Run all examples."""
    print("Vidyut-Lipi Extensible Transliteration Examples")
    print("=" * 50)
    
    try:
        basic_extensible_example()
        custom_mapping_example()
        statistics_example()
        export_patterns_example()
        pattern_analysis_example()
        
        print("\\n" + "=" * 50)
        print("✅ All examples completed successfully!")
        
    except ImportError as e:
        print(f"❌ Import error: {e}")
        print("\\nThis likely means the native module hasn't been built yet.")
        print("To build the Python package:")
        print("  1. Install maturin: pip install maturin")
        print("  2. Build the package: maturin develop")
        print("  3. Run this example again")
        
    except Exception as e:
        print(f"❌ Unexpected error: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()