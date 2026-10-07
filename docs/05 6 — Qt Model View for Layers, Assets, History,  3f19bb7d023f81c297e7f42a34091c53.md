# 05.6 — Qt Model/View for Layers, Assets, History, Data Merge, Jobs & Large Collections

# Principle

Large dynamic collections use QAbstractItemModel and delegates, not nested QWidget rows.

# Snapshot/diff bridge

Native/application service emits revisioned collection snapshot or typed diff. Qt model translates stable IDs into QModelIndex structure.

# Stable identity

internalPointer must not point to invalidated domain memory. Use adapter-owned node/index records keyed by stable IDs.

# Layers

Hierarchical model with lazy thumbnails and roles for name/type/visibility/lock/badges/selection capability. Drag MIME payload carries semantic IDs, validated on drop through Action/Command.

# Assets

Paginated/virtualized source; thumbnails async and cancel stale requests.

# History

Flat chronological transaction model with current pointer; large history can lazily request labels/details.

# Data Merge

Table model can stream/page many records instead of loading 100k rows into widget memory.

# Jobs

Job model updates coalesced progress; avoid one model reset per progress tick.

# Sorting/filtering

QSortFilterProxyModel or custom proxies operate on presentation metadata and do not mutate canonical order unless user triggers explicit sort/reorder Action.