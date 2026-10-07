# Clang-Tidy Configuration Checks

## Required Checks Configuration (.clang-tidy)
```yaml
Checks: >
  -*,
  bugprone-*,
  cert-*,
  clang-analyzer-*,
  cppcoreguidelines-*,
  modernize-*,
  performance-*,
  portability-*,
  readability-*
WarningsAsErrors: '*'
HeaderFilterRegex: '.*'
```

## Non-Negotiable Core Rules
- `cppcoreguidelines-owning-memory`: Flag all naked new/delete calls.
- `cppcoreguidelines-pro-type-reinterpret-cast`: Prohibit reinterpret_cast.
- `cppcoreguidelines-pro-type-cstyle-cast`: Prohibit C-style casts.
- `modernize-use-auto`: Use auto where type is redundantly stated.
- `modernize-use-nodiscard`: Explicit nodiscard on fallible operations.
- `bugprone-dangling-handle`: Detect views referencing temporary storage.
