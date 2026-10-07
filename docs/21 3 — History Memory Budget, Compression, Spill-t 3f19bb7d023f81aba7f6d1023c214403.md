# 21.3 — History Memory Budget, Compression, Spill-to-Disk, Pruning & User UX

# Budgets

HistoryManager tracks RAM bytes, compressed bytes and disk-spill bytes independently. Defaults scale by system memory/storage policy with user override advanced.

# Retention

Keep recent hot entries in RAM; cold raster blobs compress asynchronously; older large blobs can spill to app-managed history temp store with checksums.

# Pruning

When hard budget exceeded:

1. evict redo branches already invalidated;
2. compress cold eligible payloads;
3. spill eligible;
4. prune oldest undo entries up to retention floor.

Never drop newest active transaction.

# UX

History panel marks oldest retained boundary if truncation occurred. Low disk/history-store failure raises warning but does not corrupt current document.

# Save point

Saved revision shown in History but save itself is not a document mutation entry. Undoing before save makes document dirty relative to saved snapshot.

# Clear

Clear History is explicit destructive session action with confirmation when large history; current document remains unchanged.

# Temp cleanup

History spill store keyed by session, removed on clean close after history discarded. Crash recovery does not assume spill store alone is sufficient recovery journal.

# Tests

Artificial budget pressure, asynchronous compression race, disk unavailable, prune with current pointer mid-history, undo beyond saved revision and crash cleanup.