# 09.10.4 — Print Semantics: Overprint, Knockout, Separations, Ink Coverage, TAC & Rich Black

# Scope

These are print/prepress semantics distinct from screen blend modes.

# Separations

Preflight/render service can produce per-plate coverage for process C/M/Y/K + each SpotColorId. Registration appears on all plates.

# Overprint

Fill/stroke can carry overprint flag where PDF/print semantics support. Overprint preview simulates inks using output profile/print model; ordinary screen mode may not.

# Knockout

Default painting knocks out underlying separations unless overprint. Group/opacity/transparency interactions follow PDF/output model where supported and are explicitly validated.

# Ink coverage

At each rasterized proof sample compute sum of CMYK process percentages; TAC threshold user/output-profile setting flags regions above limit. Spots displayed separately and optionally included policy.

# Rich black

No automatic rewrite of user black by default. Export/preflight can suggest rich black recipes for large solids and preserve pure K text/lines according black-preservation option.

# Black generation

ICC conversion may use profile black generation. Petunia does not invent CMYK separation curves outside profile unless dedicated feature exists.

# UI

Separations Preview panel, plate toggles, overprint preview, ink coverage heatmap/threshold, spot list.

# Export

PDF/X and professional PDF adapters preserve overprint/spot/registration where target permits; raster exports flatten appearance and report lost plate semantics.

# Tests

Known spot/overprint fixtures against trusted PDF renderer/preflight tool, TAC heatmaps and black preservation.