# 19.16 — New Document / Document Setup Dialog Specifications

# New Document

DialogId [ptnd.dialog.new](http://ptnd.dialog.new)_document. Fields: preset category/name, width/height/units, orientation, DPI/PPI metadata, document color model/bit depth/profile, transparent/background, Surface role/count, margins, bleed and optional columns.

# Presets

Built-in/user document presets from ResourceLibrary. Editing fields marks Custom; Save Preset explicit.

# Validation

Positive finite dimensions, bounded pixel count for raster-default docs, profile compatibility and bleed/margin ranges. Estimated raster memory can warn, not silently clamp.

# Create

Build typed NewDocumentRequest; application validates and creates one transaction/session initialization. Dialog closes only after successful session creation or reports error.

# Document Setup

Same semantic schemas for existing document but changes can require color conversion, Surface resize or profile assign/convert decisions. Destructive/expensive changes preview/preflight.

# Accessibility/tests

Logical tab order, units announced, preset search keyboard. Test giant dimensions, CMYK profile, invalid margin, recent preset and cancel no session.