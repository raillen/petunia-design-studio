# 19.14 — Channels / Masks / Histogram / Info Panels Specification

# Channels

Rows for composite/model channels/alpha/spot as supported. Visibility and active edit target are separate states. Load channel as selection and save selection to channel are Actions.

# Masks

List/stack of masks attached to selection/object, type Pixel/Vector, enabled/invert/link, target focus. Create/remove/convert actions.

# Histogram

Scope selector Document/Layer/Selection, channel selector, clipping indicators. Async revisioned analysis; stale jobs discarded.

# Info

Pointer coordinates, source/composite color values, profile/model, selection/object metadata and sampled statistics. Update throttled to readable rate.

# Accessibility/tests

Target changes announced; graphs have textual bin/clipping summary. Test spot channels, mask target, stale histogram and high-frequency pointer info.