# 09.9.5 — Font Resources, Variable Fonts, OpenType, Embedding, Subsetting & Missing-Font Recovery

# Font identity

FontFaceDescriptor: family, style, PostScript name, weight/stretch/slant, source fingerprint, variation axes, optional embedded ResourceId. Display family name alone is not durable identity.

# Discovery

Platform adapter enumerates installed fonts and maps files/faces. Cache database stores fingerprint/metadata and invalidates on font set changes.

# Variable fonts

Axis tag/range/default and named instances. Text style stores axis values. UI supports registered/custom axes with units/tooltips.

# OpenType

Feature schema supports standard tags plus arbitrary advanced tag/value where font exposes them. Feature applicability is derived from shaping tables.

# Embedding

PTND may embed font resource only when user/license policy permits. Manifest records original metadata and embedding status. Application does not bypass fsType/license restrictions for exports.

# Subsetting

PDF/export subsetting performed by exporter/font library; document embedded font may remain complete unless storage policy explicitly subsets with editability implications.

# Missing

On load, requested descriptor unresolved -> MissingFontReference + fallback. Missing Fonts manager can map replacement globally/per face while retaining reversible substitution metadata where desired.

# Font change

Replacing font invalidates shaping/layout but not raw story text. Metrics changes can create overset; preflight updates.

# Tests

Multiple faces same family, variable fonts, renamed files, embedded/relinked fonts, license restrictions, fallback and substitution roundtrip.