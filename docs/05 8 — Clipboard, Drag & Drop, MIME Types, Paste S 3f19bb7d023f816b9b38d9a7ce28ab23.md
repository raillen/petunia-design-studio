# 05.8 — Clipboard, Drag & Drop, MIME Types, Paste Semantics & Cross-App Interoperability

# Clipboard layers

Internal Petunia semantic package + standard external representations.

# Copy

Selection serializes a bounded internal transfer representation with resource references/embedded small payload plus external SVG/PDF/text/image representations when applicable.

# Paste priority

If Petunia internal format is valid and compatible, preserve semantics. Otherwise choose best external format through ImportAdapter capability ranking. User can use Paste Special for explicit choice.

# Security

Clipboard content is untrusted. Same parser limits apply. Never unpickle Python objects or trust file paths without grants.

# Drag from canvas/layers

Internal MIME carries document/session/object IDs plus nonce; drop resolves through same process/service. Cross-app drag offers exported representations/temp brokered files.

# Drag into app

Files -> Place/Import depending target/context.

Color swatch -> semantic color.

Text -> text object/text insertion according focus.

Image -> placed/raster object.

Plugin-provided MIME must register adapter.

# Paste location

Canvas paste centers near viewport/last context using documented offset cascade; repeated paste offsets predictably. Text focus pastes into text, not new object.

# Undo

One Paste transaction including resource imports.