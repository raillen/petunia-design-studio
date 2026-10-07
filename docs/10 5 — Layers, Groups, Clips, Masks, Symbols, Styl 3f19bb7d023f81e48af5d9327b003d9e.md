# 10.5 — Layers, Groups, Clips, Masks, Symbols, Styles, Assets & Resource Libraries

# Layer tree

Paint/hierarchy order canonical. UI uses snapshot/diff model. Object names optional but generated labels do not become identity.

# Create/delete

Add Layer/Group/Object uses transaction. Delete verifies locked/read-only rules, removes or retains unreferenced resources per resource GC policy, and is fully undoable.

# Reorder/reparent

Drag in Layers computes ReparentObjectsCommand with target parent/index. Preview indentation/insertion. Prevent cycles, illegal Surface crossing or symbol constraints.

# Group

Group wraps selected siblings preserving world transforms and paint order. Ungroup composes transforms into children preserving visual result.

# Clip

Clip object/group semantics specify clip provider and clipped children. Dragging a layer into clip zone has distinct insertion visual from normal nesting.

# Masks

Attach pixel/vector mask to object/group. Mask target selection is explicit. Invert/disable/unlink commands. Vector mask stays editable vector.

# Symbols

SymbolDefinition owns source content; SymbolInstance references definition + transform + allowed overrides. Editing definition updates all instances. Detach resolves a local copy. Override schema uses stable property paths/IDs.

# Styles

ObjectStyle/TextStyle contains appearance/text properties with inheritance/links by StyleId. Apply can link or copy depending style type. Redefine from selection is explicit.

# Assets

AssetLibrary entries may be embedded package resources, application library or external plugin source. Drag placing creates object/resource reference. Asset preview derived.

# Resource lifecycle

Reference count/graph analysis separates unreferenced from removable; purge is explicit maintenance action. Save package may omit truly unreferenced resources only under documented policy.

# Linked resources

Placed image/data resource can be linked. ResourceManager tracks grant/path/hash/status, detects changed/missing, relinks without changing ResourceId unless replacing semantic resource deliberately.

# GUI

Layers tree virtualized; visibility/lock inline controls; badges for mask/effect/symbol/link. Assets/Symbols/Styles searchable panel with drag/drop and context menus.

# Commands

CreateGroup, Ungroup, Reparent, Reorder, AttachClip, AttachMask, CreateSymbol, DetachSymbol, SetOverride, CreateStyle, ApplyStyle, UpdateStyle, PlaceAsset, RelinkResource, EmbedResource.

# Tests

Transform-preserving group/ungroup, hierarchy cycle rejection, clip/mask ordering, symbol update/override, missing resources, drag/drop semantics and roundtrip.