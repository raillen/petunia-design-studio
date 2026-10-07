# 09.11.8 — PTND Migration Framework, Forward Compatibility, Opaque Preservation & Reference Validator CLI

# Migration dimensions

containerVersion, schemaVersion, resource codec version and extension schema migrate independently.

# Core schema migration

Functions vN -> vN+1 over validated old DTO. Each step pure/deterministic and emits MigrationReport: changed fields, warnings, lost unsupported optional state (ideally none), new capabilities.

# Forward compatibility

Reader with older schema never guesses meaning of newer required fields. minimumReaderVersion/requiredCapabilities gate safe behavior.

# Opaque preservation

Unknown optional extension directories copied byte-for-byte on save if untouched and safe. Manifest entries preserved. If core edit invalidates extension-declared dependency and host cannot understand it, save warns/blocks according extension contract.

# Validator CLI

```
petunia-cli validate file.ptnd
petunia-cli inspect-manifest file.ptnd
petunia-cli list-resources file.ptnd
petunia-cli migrate --to current input.ptnd output.ptnd
petunia-cli salvage input.ptnd output.ptnd
```

Machine-readable JSON report option.

# Reference unpack

Tool extracts safely to selected directory after traversal/symlink checks; intended for developers, not normal editing.

# Compatibility fixtures

Every released schema version remains fixture. CI opens and migrates all supported historical fixtures.

# Third-party spec

Publish package profile, JSON Schemas, PTILE spec and validation examples independent from Petunia source code so other tools can implement readers/writers.