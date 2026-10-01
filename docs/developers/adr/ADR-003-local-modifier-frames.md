# ADR-003: Persistent local modifier frames

- **Status:** Accepted
- **Date:** 2026-10-01
- **Scope:** Milestone Required (MVP)
- **Affects:** document, application, shell, native packages; schema 3
- **Updates:** ADR-002 decision 4 and schema version; its local-path, integrity and publication contracts remain applicable

## Context

Schema 2 paths were local, but modifier parameters used parent coordinates. Subtracting the current bounds origin on every read left crop, perspective and transparency behind after moving. Resizing changed their relationship to the source. Baking the entire evaluated chain and removing only contours could apply surviving geometry twice. Changing bounds origin during bake displaced rotated geometry and masks.

## Decision

1. Persist `ModifierSpace::Local { reference_size: [width, height] }` per entry. `Parent` is an explicit compatibility/input descriptor. Normalize it once using current bounds, including disabled entries. Placement/resize never rewrite parameters or reference size; IDs/order survive normalization.
2. Load schemas 1/2 by migrating missing/parent markers according to their documented convention. Schema 3 requires explicit local markers. Reject unknown versions/invalid frames. Saves write schema 3, so older readers reject rather than misinterpret it. Package/payload versions must agree before migration.
3. For geometry, map the current local outline into each entry's reference frame, evaluate, then map back. Nonuniform resize scales contour geometry in both axes without an invented average distance. Crop/warp flattening tolerance accounts for scale. Source paths and recipes stay editable.
4. Transparency pulls the sample into the reference frame before projection. Projecting onto resized endpoints is wrong under nonuniform scale. Stop sampling preserves source order/duplicate behavior without per-sample allocation or sorting. Local/world sampling APIs are available; ancestor compositing is separate.
5. Empty crop yields an empty evaluated outline and retains the source. Invalid warp/offset retains its existing fallback; broader geometry limits/diagnostics remain required.
6. Explicit `BakeGeometry` consumes enabled geometry and rebases surviving transparency/disabled entries, preserving their per-axis scale. New placement includes the object's rotation. Prepare and validate the candidate/ChangeSet before publication; empty results stay empty. `BakeContour` consumes only a contour prefix and retains following modifiers. If other geometry precedes contour, fail with a reason to use `BakeGeometry`; never silently consume another effect.
7. Convert tool world inputs through object/ancestor transforms. Perspective overlay/preview/commit use local frames; untouched rotated corners stay unchanged. A world-axis-aligned crop not representable by a local rectangle fails with a polygon-mask requirement. Polygon masks remain pending; no enclosing AABB substitutes for the requested crop.
8. Validate replacement chains without cloning the whole target/image. Guide creation allocates unique local IDs in a domain command, replacing millisecond-based IDs that collided on consecutive ruler gestures. History replays stored identities.

## Evidence and limits

Focused tests cover schema migration, movement/nonuniform resize, contour scaling, diagonal opacity, rotated ancestors, disabled entries, invalid frames, empty crop, rotated/partial bake, history branches and package round trips. A shell pointer fixture covers rotated perspective; ruler fixtures cover consecutive guide creation. `cargo xtask migrations` includes modifier frames/packages. Generated evidence: [contracts.json](/implementation/contracts.json), [checks.json](/implementation/checks.json).

This completes the modifier-frame portion of M0, not M0 or the MVP. Shared rendering, paint frames, spatial opacity in preview/export, resource COW, quotas, workers, pixel/mask layers, recovery and Linux hardware/accessibility gates remain open. Center-opacity export remains an approximation. See the [execution ledger](/developers/implementation-progress).
