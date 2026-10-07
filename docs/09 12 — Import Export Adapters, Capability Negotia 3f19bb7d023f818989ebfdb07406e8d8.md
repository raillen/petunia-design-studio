# 09.12 — Import/Export Adapters, Capability Negotiation, Fidelity & Preflight

# Adapter rule

External formats translate to/from canonical model. Nenhum formato externo define storage interno.

# Import phases

Acquire bounded input -> detect/sniff -> parse private AST -> validate hostile structures -> capability/ambiguity analysis -> ImportDecision -> convert to Petunia staging DTO -> validate -> one atomic document transaction -> ImportReport.

# File detection

Explicit format > strong signature > structural sniff > extension hint. Sniff reads bounded bytes and cannot unpack unlimited content.

# Staging

Parser never incrementally mutates DocumentStore. Failure leaves document unchanged unless explicit salvage mode.

# Export

Exporter gets immutable evaluated snapshot at committed revision + ExportOptions + cancellation + diagnostics. Editing can continue while export uses snapshot.

# Export phases

resolve target/options -> capability analysis -> degradation/preflight -> policy decisions -> snapshot -> temp encode -> validate -> atomic destination commit -> ExportReport.

# Capability model

Geometry, text, raster, appearance, color, document/pages, resources, metadata, extension/data. Values: Native, RepresentableWithConstraints, RequiresExpansion, RequiresRasterization, Unsupported.

# Fidelity grades

Exact, EquivalentAppearance, Approximate, DestructiveDegradation, Unsupported. Preset maps grades to allow/warn/confirm/reject.

# Baseline writers

PNG/JPEG/WebP/TIFF, SVG, PDF. Additional adapters ship only with explicit test corpus/licensing/security/fidelity.

# PDF

Professional vector/text/color output. PDF/X becomes release target only when validator/conformance evidence exists; external validation/conversion can be supported before native certification.

# SVG

Editable vector projection; unsupported effects choose expand/rasterize/reject per policy. One SVG per Surface can be embedded as optional interchange.

# Raster

Area, dimensions, scale, resampling, bit depth, alpha, target profile, metadata.

# Security

All parsers hostile-input, fuzzed and size/decompression bounded; external links/network disabled by default.

[09.12.1 — Format Support Matrix, Import/Export Targets & Fidelity Roadmap](09%2012%201%20%E2%80%94%20Format%20Support%20Matrix,%20Import%20Export%20Tar%203f19bb7d023f81f6835cd787c782e97c.md)

[09.12.2 — Export Presets, Degradation Plan, PDF/SVG/Raster Semantics & Output Validation](09%2012%202%20%E2%80%94%20Export%20Presets,%20Degradation%20Plan,%20PDF%20SV%203f19bb7d023f8130aa95ebdc70280ee9.md)

[09.12.3 — Detailed Fidelity Capability Matrix: Native, SVG, PDF, Raster & Roadmap Formats](09%2012%203%20%E2%80%94%20Detailed%20Fidelity%20Capability%20Matrix%20Nati%203f19bb7d023f817597e5d4f25d95e402.md)

[09.12.4 — Import Mapping Rules: SVG/PDF/Raster Ambiguities, Staging AST & Editable Reconstruction](09%2012%204%20%E2%80%94%20Import%20Mapping%20Rules%20SVG%20PDF%20Raster%20Ambi%203f19bb7d023f81e0bc63f5c96e36d06e.md)

[09.12.5 — Export Naming, Multi-Output Transactions, Atomicity, Conflicts & Validation](09%2012%205%20%E2%80%94%20Export%20Naming,%20Multi-Output%20Transactions%203f19bb7d023f8173a8ede451e8487dd4.md)