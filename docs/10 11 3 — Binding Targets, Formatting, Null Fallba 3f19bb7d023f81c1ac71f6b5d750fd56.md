# 10.11.3 — Binding Targets, Formatting, Null/Fallback & Image/Color Semantics

# Binding

BindingId targets ObjectId + PropertyId or text token range. Property must be bindable/type-compatible.

# Text

Template segments literal + expression. Formatting evaluates at preview/generation without mutating source template.

# Number/date

Formatting uses explicit locale policy where reproducibility matters.

# Null

Policies: Empty, Fallback, KeepTemplate, Error/SkipRecord.

# Image

Expression yields ResourceRef/path through broker. Fit contain/cover/stretch/original, focal position and missing fallback.

# Color

Expression returns ColorValue or named swatch; parsing syntax explicit.

# Visibility

Boolean binding can control visibility when descriptor allows.

# Tests

Null, wrong types, text overflow, missing image, CMYK/spot color and fallback.