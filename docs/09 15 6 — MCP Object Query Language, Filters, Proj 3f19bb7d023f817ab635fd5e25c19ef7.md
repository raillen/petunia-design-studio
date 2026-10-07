# 09.15.6 — MCP Object Query Language, Filters, Projection & Context-Efficient Retrieval

# Goal

Allow agents to find objects without downloading entire document or executing arbitrary code.

# Query

Scope: document/Surface/parent subtree.

Filter combinators bounded: type is/in, name exact/contains, tags, visible/locked, property comparisons on indexable PropertyIds, spatial intersects/within bounds, relation parent/descendant, selected boolean.

Logical AND/OR depth bounded.

# Projection

Caller requests fields: id,type,name,parent,bounds,selected,property IDs subset, resource refs. Default summary is compact.

# Sorting

paint order, name, type, spatial x/y or selected supported. Arbitrary expression sorting not V1.

# Pagination

Limit max; cursor tied to revision/query hash. If document revision invalidates stable result ordering, return stale cursor and restart recommendation.

# Text/privacy

Object text content excluded from generic summary unless caller requests text property with [document.read](http://document.read) scope. Pixel/resource bytes never returned by query.

# Spatial

Bounds query uses derived spatial index and can specify document/Surface coordinates.

# Explain

Optional query explain returns normalized filter, estimated result count/cost category, not internal sensitive implementation.

# Tests

100k objects, nested filters, stale cursor, permissions, spatial queries and compact token-size benchmarks.