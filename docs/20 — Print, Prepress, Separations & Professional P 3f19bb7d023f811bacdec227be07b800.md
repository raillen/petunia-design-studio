# 20 — Print, Prepress, Separations & Professional PDF Output

# Goal

Consolidate print-production semantics that cannot remain dispersed across Color and Export.

# Coverage

CMYK/spot, registration, overprint/knockout, separations, total area coverage, bleed/trim/media boxes, marks, output intent, transparency flattening, font embedding/subsetting, PDF/X targets, black preservation and preflight.

# Architecture

PrepressAnalyzer consumes immutable evaluated document snapshot and target PrintPreset, producing SeparationModel + PreflightReport. PDF exporter consumes same target semantics.

# No fake certification

A PDF/X mode is only exposed as conforming after external validator corpus proves exact target profile. Until then UI says “PDF with X-oriented settings” only if truthful.

# View modes

Overprint Preview, Separations Preview, Ink Coverage and Soft Proof are view/evaluation modes and do not mutate document.

[20.1 — Print Document Setup, Bleed, Trim/Media Boxes, Marks & Page Geometry](20%201%20%E2%80%94%20Print%20Document%20Setup,%20Bleed,%20Trim%20Media%20Box%203f19bb7d023f815f94e4e40dc374f69f.md)

[20.2 — Separations Preview, Ink Coverage, TAC, Registration & Spot Plates](20%202%20%E2%80%94%20Separations%20Preview,%20Ink%20Coverage,%20TAC,%20Reg%203f19bb7d023f8150bf84dd029629340c.md)

[20.3 — Overprint/Knockout Preview, Black Handling & Transparency Flattening](20%203%20%E2%80%94%20Overprint%20Knockout%20Preview,%20Black%20Handling%20%203f19bb7d023f81f3ba91e711a8a329ff.md)

[20.4 — PDF/X Profiles, Output Intent, Font Rules & External Validation](20%204%20%E2%80%94%20PDF%20X%20Profiles,%20Output%20Intent,%20Font%20Rules%20&%203f19bb7d023f8145a6a6cfdc3985cdc8.md)

[20.5 — Preflight Rule Engine, Issue Codes, Fix Actions & Export Blocking Policy](20%205%20%E2%80%94%20Preflight%20Rule%20Engine,%20Issue%20Codes,%20Fix%20Act%203f19bb7d023f81e68cf5d5e6af1b7ef4.md)