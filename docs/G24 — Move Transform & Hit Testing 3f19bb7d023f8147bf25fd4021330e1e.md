# G24 — Move/Transform & Hit Testing

# Goal

Make created objects selectable and transformable with professional feedback.

# Depends

G23, G09.6.5.

# Authority

18.01, 10.1, 09.6.5.

# Owner

editor-engineer.

# Deliverables

SpatialIndex baseline, HitTestService, selection state, transform gesture/session, bbox handles, TransformObjects command and Transform panel binding.

# Acceptance

Click/marquee selection, drag move, resize, rotate baseline, numeric transform, multi-selection and undo/save roundtrip.

# Performance

Hit-test meets provisional p95 budget.

# Evidence

100k-object benchmark subset and interaction goldens.