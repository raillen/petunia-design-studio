# 04.4 — C++23 Toolchain, Compilers, Standard Library, Sanitizers & Build Profiles

# Standard

C++23 product baseline.

# Toolchains

Linux: Clang and GCC supported in CI.

Windows: MSVC and/or clang-cl release profile by packaging decision.

macOS: AppleClang supported production compiler.

# Compiler policy

Warnings-as-errors for Petunia sources. Third-party warnings isolated. Required warning set is toolchain-specific and maintained centrally.

# Build profiles

Debug — assertions, developer diagnostics.

Dev — optimized enough for realistic UI, rich instrumentation.

Release — optimized, hardened, symbols split.

ASan/UBSan — memory/undefined behavior.

TSan — race-focused dedicated build.

Coverage — tests only.

Profile — frame pointers/instrumentation appropriate to profiler.

Fuzz — sanitizer + fuzz targets.

# Standard library

Use portable standard facilities unless target-specific optimization has measured need. std::jthread, stop_token, span, string_view, chrono, filesystem and expected are preferred where semantics fit.

# Assertions

Internal invariants use assertions/contracts in debug/dev. User/file validation must produce typed errors and cannot rely on assert.

# CMake

Targets declare PUBLIC/PRIVATE dependencies correctly; include directories do not leak implementation. Presets encode supported environments.

# Link-time optimization

LTO/ThinLTO is release candidate only after build/link stability and profiler evidence. PGO may be evaluated after representative workflows exist.