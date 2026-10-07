# 08.15 — Accessibility, Keyboard, Pointer, Pen, IME, Localization & HiDPI

# Keyboard

Important commands reachable by menu/shortcut/palette. Panels cyclable. Drag-only workflows have numeric/command alternatives. Focus order follows task.

# Screen readers

QAccessible roles/names/states/actions. Canvas offers semantic selection/tool/status tree; Layers/Properties fully semantic.

# Pointer

Invisible hit-region expansion. Hover never sole route.

# Pen

Pressure, tilt, rotation, eraser, barrel buttons normalized. Hover cannot commit. Mapping configurable.

# Touch/trackpad

Pinch zoom/pan. Touch painting explicit setting.

# IME

Composition ranges, grapheme cursor, bidi, shaping, Unicode selection. Never treat UTF-8 byte offsets as user characters.

# Localization

TextId for visible text; locale-aware numbers/units; RTL where applicable; stable ActionId remains technical.

# HiDPI

Qt DIP UI + renderer pixel ratio. Mixed-DPI monitor move recreates target safely.

# Cognitive clarity

Stable context groups, explicit live/destructive distinction, progressive disclosure.

# Verification

Keyboard-only scripts, screen-reader matrix, contrast, 200% scaling, large handles, reduced motion, alternate locales.