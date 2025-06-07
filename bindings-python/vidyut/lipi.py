"""`vidyut.lipi` transliterates scripts used within the Indosphere.

This module provides both basic transliteration capabilities and advanced runtime extensible
transliteration for handling unknown patterns in texts, particularly useful for Vedic texts
with custom accent encodings.

Basic Usage:
    >>> import vidyut.lipi as lipi
    >>> result = lipi.transliterate("namaste", lipi.Scheme.Devanagari, lipi.Scheme.Slp1)
    >>> print(result)

Advanced Usage with Pattern Discovery:
    >>> lipika = lipi.ExtensibleLipika()
    >>> result = lipika.transliterate_extensible(
    ...     "agni#mīḷe# purohitam",
    ...     lipi.Scheme.BarahaSouth,
    ...     lipi.Scheme.Slp1,
    ...     source_id="vedanidhi"
    ... )
    >>> print(f"Result: {result.result}")
    >>> print(f"Discovered {len(result.discovered_patterns)} patterns")
"""

from vidyut.vidyut import lipi as __mod

# Basic functions
detect = __mod.detect
transliterate = __mod.transliterate

# Basic types
Scheme = __mod.Scheme

# Advanced extensible transliteration types
PatternType = __mod.PatternType
DiscoveredPattern = __mod.DiscoveredPattern
ExtensionStats = __mod.ExtensionStats
ExtensibleTransliterationResult = __mod.ExtensibleTransliterationResult
ExtensibleLipika = __mod.ExtensibleLipika
