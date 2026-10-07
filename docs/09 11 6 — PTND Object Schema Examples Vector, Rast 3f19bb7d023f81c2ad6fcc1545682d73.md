# 09.11.6 — PTND Object Schema Examples: Vector, Raster, Text, Masks, Effects, Symbols & Data Merge

# Vector example

```json
{
  "id":"obj-1",
  "type":"ptnd.vector.path",
  "transform":[1,0,0,1,100,50],
  "visible":true,
  "locked":false,
  "opacity":1.0,
  "blendMode":"normal",
  "path":{
    "fillRule":"nonzero",
    "contours":[{
      "id":"contour-1",
      "closed":true,
      "nodes":[
        {"id":"node-1","x":0,"y":0,"kind":"cusp","in":null,"out":null}
      ]
    }]
  },
  "appearance":"appearance-1"
}
```

# Raster example semantics

RasterLayer references raster ResourceId/tile set; does not inline megabytes. Pixel extent and transform distinct. Original linked/placed encoded source may remain separate ResourceId.

# Text example semantics

TextObject references StoryId; Frame geometry includes width/height/insets/columns and flow next/previous identifiers. Character runs live in story record, not duplicated per frame.

# Mask

Mask binding record: maskId, kind pixel|vector, source ObjectId/ResourceId, transform, invert, combine mode, enabled. Object can maintain ordered mask stack if feature supports.

# Effect

```
EffectRecord
 id
 kind: "ptnd.effect.gaussian_blur"
 schemaVersion
 enabled
 params { radius, edgeMode, ... }
 mask?
```

# Symbol

Definition stores root object subtree/reference set. Instance stores definitionId, transform, override map keyed stable PropertyId/object path.

# Data Merge

DataSourceRecord describes provider/source metadata/fingerprint without secrets. BindingRecord targets ObjectId + PropertyId/path + expression AST/source + fallback/format/null policy.

# Extension

Plugin may attach ExtensionPayloadRef to object but cannot insert arbitrary fields into core object namespace.

# Schema examples

Repository must include canonical minimal and rich JSON examples validated in CI, used in docs and third-party tooling tests.