# 09.10.2 — Working/Compositing Space Policy, Linearization, Precision & Mixed-Color Content

# Key decision

Canonical object colors may remain in document-native/source color spaces, but compositing requires a mathematically defined working representation.

# Proposed V1 policy

Interactive/final compositor converts drawable samples to a high-precision **linear-light RGB compositing space** derived from document working profile/reference, using float16 or float32 intermediates selected by quality/performance evidence. CMYK/spot identity remains canonical and is restored/preserved for export where target supports native semantics.

# Why

Many blend/filter operations have well-defined predictable behavior in RGB linear intermediates and GPU support. Direct CMYK compositing for every effect would create inconsistent backend complexity.

# Caveat

Print-critical operations that semantically require native CMYK/spot/overprint must bypass or augment generic RGB compositor; Prepress specification owns those rules. This policy must be validated against professional expectations before lock.

# Precision

CPU reference float32. GPU float16 intermediates allowed only after visual/numeric error benchmarks; operations prone to banding can request float32 target.

# Linearization

ICC transform produces/consumes linear working representation where possible or explicit transfer-function conversion is applied. Never apply blend formulas accidentally to encoded sRGB unless blend standard requires perceptual encoding.

# Mixed objects

RGB placed image, CMYK vector color and Lab swatch can coexist; each evaluation transforms to compositor working space for screen while canonical source semantics remain.

# Export

PDF/native print exporter can serialize original CMYK/spot values for untouched compatible objects rather than roundtrip through RGB. Any operation that bakes appearance may require conversion/preflight.

# Tests

Mixed-space document visual oracle, float16 error/banding, blend modes, CMYK object preservation on PDF path and no repeated-conversion drift.