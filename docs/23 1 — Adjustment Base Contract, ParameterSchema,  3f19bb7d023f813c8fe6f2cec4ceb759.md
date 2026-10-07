# 23.1 — Adjustment Base Contract, ParameterSchema, Preview Sessions & Serialization

# Base descriptor

EffectDescriptor {

EffectKind stable ID;

version;

category;

accepted input models/formats;

ParameterSchema[];

ROI policy;

alpha policy;

color-space policy;

CPU/GPU capabilities;

deterministic flag;

export/fidelity metadata;

help/reference IDs;

}

# Parameter

Stable PropertyId, type, unit, default, min/max/step UI hints, validation, animatable/bindable flag, preview cost hint and serialization field mapping.

# Preview session

Selecting/editing effect opens EffectPreviewSession tied to object/effect EntryId and input revision. Scrubs update staged params; renderer evaluates preview; Apply/finish commits one SetEffectParams transaction; Cancel restores original.

# Versioning

Effect params include effectVersion. Migration is per effect kind, deterministic and independent from GUI.

# Unknown effect

Built-in unknown version -> compatibility failure/read-only unless safe fallback snapshot exists. Plugin effect missing -> preserve opaque canonical payload and optional baked fallback only if provider contract declares one.

# Quality

Canonical params never store “preview quality”; quality is evaluation policy.

# Tests

Schema validation, default roundtrip, staged cancel, migration, CPU/GPU parity and export analyzer mapping.