# G030 — First Integrated Commercial-Mini Workflow Gate

# Goal

Evaluate whether architecture has crossed from framework scaffold to credible design-editor foundation.

# Depends

G028, G029.

# Primary

release-verifier-style role + tester + UX/accessibility/performance reviewers.

# Scenario

Create a small real brand/social composition using Surfaces, parametric shapes, Pen path, fills/gradients/strokes, text placeholder only if text Goal subsequently exists or use vector marks, alignment/snapping, layers, save/reopen, PNG/SVG.

# Criteria

No P0 correctness/data-loss/security issues; no known mutation bypass; save/recovery fault suite baseline passes; critical UI accessible; renderer stable across supported test OS; performance within or near documented budgets with justified gaps; docs match behavior.

# Deliverables

Commercial-mini audit report, capability matrix status, architecture drift findings, prioritized next Goal wave (text/layout then raster), screenshots/outputs and benchmark bundle.

# Decision

GO allows broad feature expansion. NO-GO requires fixing foundational gaps before adding large new subsystems.

# Non-goals

Marketing release, feature parity with Affinity, installer production.