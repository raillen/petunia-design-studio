# 19.18 — Channels Panel

# Identity

PanelId ptnd.panel.channels.

# Model

Composite plus semantic channels from current PixelTarget/document: R/G/B, C/M/Y/K, Gray, Alpha, Spot plates, stored alpha channels/masks as applicable.

# Distinguish states

Visibility/view solo is view state.

Editable active channel is target state.

Stored channel content is canonical raster resource.

# Actions

Show/hide/solo; select as target; Duplicate Channel; New Alpha Channel; Delete stored channel; Load as Selection; Selection to Channel; Fill/Invert stored channel.

# Safety

Editing a component channel can have destructive implications; target indicator and status must be explicit. Composite row cannot accidentally become editable raw buffer.

# Thumbnails

Async small grayscale previews.

# Accessibility

Each row announces channel name, type, visible, active/editable and lock state.

# Tests

RGB/CMYK/alpha/spot, solo view, selection conversion, mask target, delete referenced channel and undo.