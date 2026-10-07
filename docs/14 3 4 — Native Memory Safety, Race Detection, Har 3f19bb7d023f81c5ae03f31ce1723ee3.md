# 14.3.4 — Native Memory Safety, Race Detection, Hardening & Crash Triage Protocol

# Sanitizers

ASan/UBSan on representative C++ tests/integration; TSan dedicated concurrency suites; LSan where platform/toolchain practical.

# Static

clang-tidy/security/lifetime rules, compiler warnings, dependency vulnerability scans. Static finding severity reviewed, not blindly suppressed.

# Hardening

Stack protections, ASLR/DEP platform defaults, control-flow/security flags where toolchain supports without unacceptable compatibility cost, fortified APIs and symbol strategy.

# Concurrency

TileStore, JobScheduler, renderer resource queues, event bridges, document snapshots and shutdown stress under TSan/repetition.

# Crash triage

Minidump/backtrace -> BuildId/source revision -> sanitizer reproduction if possible -> minimal fixture -> severity/security assessment -> regression.

# Third-party native crash

Codec/library/parser failure isolated where architecture supports; otherwise main process must preserve original files/recovery and produce bounded diagnostic.

# Release

No known untriaged sanitizer memory corruption in release-critical paths.