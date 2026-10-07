# Escape-Hatch Formal Waiver Template

```json
{
  "id": "EH-CPP-001",
  "language": "cpp",
  "file": "src/hardware/hal_driver.cpp",
  "line_range": [45, 52],
  "hatch_type": "raw_cast",
  "justification": "Memory-mapped I/O register manipulation requires explicit address conversion.",
  "safety_invariants": "Base MMIO address verified at initialization; hardware page protected in kernel mode.",
  "quarantine_boundary": "Isolated exclusively within hal_driver.cpp internal implementation; public interface exposes safe strongly typed functions.",
  "reviewer": "systems-architect",
  "approved_at": "2026-09-09",
  "status": "active"
}
```
