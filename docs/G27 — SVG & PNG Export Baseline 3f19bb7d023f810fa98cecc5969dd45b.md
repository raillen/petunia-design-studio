# G27 — SVG & PNG Export Baseline

# Goal

Produce first useful external outputs with fidelity reporting.

# Depends

G23–G26, G10–G11.

# Authority

24.1–24.2, 09.12.*, 08.12.

# Owner

editor-engineer.

# Deliverables

PNG exporter through final renderer; SVG exporter for shapes/paths/basic appearance; ExportRequest/Report, temp+validate+atomic commit and basic Export dialog/panel path.

# Acceptance

Vector fixture SVG reopens in reference viewer and Petunia with expected grade; PNG golden correct profile/alpha baseline; unsupported feature yields degradation item.

# Evidence

format fixtures and external viewer validation.