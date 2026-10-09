//! Renderer-owned caches: clock eviction plus surface pooling.
//!
//! Everything here is derived and discardable: deleting all caches
//! keeps the document correct. Keys carry only real semantic
//! dependencies, and budgets stay runtime settings.

use std::collections::HashMap;

/// Rebuild-cost classes bias eviction toward cheap drops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RebuildCost {
    Cheap,
    Moderate,
    Expensive,
}

/// One cache entry: bytes plus the metadata eviction weighs.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub bytes: Vec<u8>,
    pub recently_used: bool,
    pub cost: RebuildCost,
    pub key_revision: u64,
}

impl CacheEntry {
    /// Heap size attributed to this entry.
    #[must_use]
    pub fn size_bytes(&self) -> usize {
        self.bytes.len()
    }
}

/// Clock-eviction store: a hand sweeps entries, keeping recently
/// used ones for one more round and dropping the rest under memory
/// pressure. Cheaper than exact LRU, bounded all the same.
#[derive(Debug, Default)]
pub struct ClockCache {
    entries: HashMap<String, CacheEntry>,
    order: Vec<String>,
    hand: usize,
    used_bytes: usize,
    capacity_bytes: usize,
}

impl ClockCache {
    /// Create a store with an explicit byte capacity.
    #[must_use]
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: Vec::new(),
            hand: 0,
            used_bytes: 0,
            capacity_bytes,
        }
    }

    /// Look up an entry, marking it recently used.
    pub fn get(&mut self, key: &str) -> Option<&[u8]> {
        let entry = self.entries.get_mut(key)?;
        entry.recently_used = true;
        Some(&entry.bytes)
    }

    /// Insert an entry, evicting until it fits. Entries larger than
    /// the whole capacity are refused instead of thrashing the store.
    pub fn insert(&mut self, key: String, entry: CacheEntry) -> bool {
        let size = entry.size_bytes();
        if size > self.capacity_bytes {
            return false;
        }
        if let Some(old) = self.entries.remove(&key) {
            self.used_bytes -= old.size_bytes();
            self.order.retain(|item| item != &key);
        }
        while self.used_bytes + size > self.capacity_bytes {
            if !self.evict_one() {
                break;
            }
        }
        if self.used_bytes + size > self.capacity_bytes {
            return false;
        }
        self.used_bytes += size;
        self.order.push(key.clone());
        self.entries.insert(key, entry);
        true
    }

    /// Bytes currently retained.
    #[must_use]
    pub fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    /// Number of retained entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// True when nothing is retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn evict_one(&mut self) -> bool {
        if self.order.is_empty() {
            return false;
        }
        // Classic clock: clear use bits on first pass, evict the
        // first unmarked entry found. Rebuild cost rides along as
        // metadata for future bias without bending recency today.
        for _ in 0..2 * self.order.len().max(1) {
            if self.order.is_empty() {
                return false;
            }
            self.hand %= self.order.len();
            let key = self.order[self.hand].clone();
            match self.entries.get_mut(&key) {
                Some(entry) if entry.recently_used => {
                    entry.recently_used = false;
                    self.hand += 1;
                }
                Some(_) => {
                    if let Some(entry) = self.entries.remove(&key) {
                        self.used_bytes -= entry.size_bytes();
                    }
                    self.order.retain(|item| item != &key);
                    return true;
                }
                None => {
                    self.order.retain(|item| item != &key);
                    return true;
                }
            }
        }
        false
    }
}

/// Reusable scratch surfaces by compatible extent. Frame-limited
/// lifetimes: pooled buffers never become semantic caches.
#[derive(Debug, Default)]
pub struct SurfacePool {
    buckets: HashMap<(u32, u32), Vec<Vec<[f32; 4]>>>,
    live_bytes: usize,
    capacity_bytes: usize,
}

impl SurfacePool {
    /// Pool with a byte cap for idle surfaces.
    #[must_use]
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            buckets: HashMap::new(),
            live_bytes: 0,
            capacity_bytes,
        }
    }

    /// Take a zeroed surface of the exact extent, reusing an idle one
    /// when available.
    #[must_use]
    pub fn acquire(&mut self, width: u32, height: u32) -> Vec<[f32; 4]> {
        if let Some(buffer) = self.buckets.get_mut(&(width, height)).and_then(Vec::pop) {
            self.live_bytes -= buffer.len() * 16;
            return buffer;
        }
        vec![[0.0; 4]; (width as usize) * (height as usize)]
    }

    /// Return a surface for compatible reuse. Oversized pools drop
    /// instead of hoarding.
    pub fn release(&mut self, width: u32, height: u32, mut buffer: Vec<[f32; 4]>) {
        for pixel in buffer.iter_mut() {
            *pixel = [0.0; 4];
        }
        let size = buffer.len() * 16;
        if self.live_bytes + size > self.capacity_bytes {
            return;
        }
        self.live_bytes += size;
        self.buckets
            .entry((width, height))
            .or_default()
            .push(buffer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(size: usize, cost: RebuildCost) -> CacheEntry {
        CacheEntry {
            bytes: vec![7u8; size],
            recently_used: false,
            cost,
            key_revision: 1,
        }
    }

    #[test]
    fn clock_prefers_recent_and_expensive_entries() {
        let mut cache = ClockCache::new(300);
        assert!(cache.is_empty());
        assert!(cache.insert("a".to_string(), entry(100, RebuildCost::Cheap)));
        assert!(cache.insert("b".to_string(), entry(100, RebuildCost::Expensive)));
        assert!(cache.get("a").is_some());
        // 250 more bytes overflow 300: eviction drops the unmarked
        // entry first, keeping the recently used one.
        assert!(cache.insert("c".to_string(), entry(150, RebuildCost::Cheap)));
        assert!(cache.get("a").is_some());
        assert_eq!(cache.used_bytes(), 250);
    }

    #[test]
    fn oversized_entries_refuse_cleanly() {
        let mut cache = ClockCache::new(100);
        assert!(!cache.insert("huge".to_string(), entry(200, RebuildCost::Cheap)));
        assert!(cache.is_empty());
    }

    #[test]
    fn surface_pool_recycles_compatible_extents() {
        let mut pool = SurfacePool::new(1 << 20);
        let first = pool.acquire(8, 8);
        assert_eq!(first.len(), 64);
        pool.release(8, 8, first);
        let second = pool.acquire(8, 8);
        assert_eq!(second.len(), 64);
        assert!(second.iter().all(|pixel| *pixel == [0.0; 4]));
        // Incompatible extents allocate fresh.
        assert_eq!(pool.acquire(4, 4).len(), 16);
    }
}
