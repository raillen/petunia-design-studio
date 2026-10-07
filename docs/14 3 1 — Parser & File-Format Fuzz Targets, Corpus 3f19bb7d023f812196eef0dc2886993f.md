# 14.3.1 — Parser & File-Format Fuzz Targets, Corpus Strategy & Resource Limits

# Targets

PTND ZIP envelope, manifest/document JSON, PTILE header/payload, SVG/XML, PDF adapter, PNG/JPEG/TIFF/WebP metadata/wrapper paths, ICC, fonts where library exposes safe fuzz entry, Data Merge CSV/JSON and library/plugin packages.

# Harness

In-memory bounded input where possible; no network/filesystem side effects; sanitizer build; deterministic timeout/iteration; parser only or parse+validate+materialize stages separated.

# Limits

Entry count, nesting, dimensions, multiplication overflow, decompressed bytes, compression ratio, string length, object count, recursion and CPU timeout.

# Corpus

Minimal valid, maximal valid, malformed boundary cases, dictionary seeds from schemas and previous CVE/regression patterns without copying exploit payload unnecessarily.

# Crash handling

Any crash/OOM/hang is security defect until triaged. Reproducer minimized and added to regression corpus.

# Differential

When two parsers/reference validators exist, disagreement corpus helps find acceptance bugs but reference is not blindly trusted.

# Evidence

Fuzzer engine/version, sanitizer, duration/executions, corpus size/coverage indicator and zero/unresolved findings.