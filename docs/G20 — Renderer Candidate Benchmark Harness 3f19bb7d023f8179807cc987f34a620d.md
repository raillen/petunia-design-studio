# G20 — Renderer Candidate Benchmark Harness

# Goal

Implement comparable benchmark harness before backend selection.

# Depends

G03, G08, G18.

# Authority

04.6, 15.3, 25.

# Owner

technology-decision-agent + renderer-engineer.

# Deliverables

backend interface spike, workloads 100k vectors/8k raster/text/effects/offscreen, timing/memory collection, correctness output capture and build scripts for candidate backends.

# Acceptance

Same scene description can execute candidates; raw measurements reproducible across reference machines; no candidate contaminates Document model.

# Evidence

benchmark dataset and methodology report.