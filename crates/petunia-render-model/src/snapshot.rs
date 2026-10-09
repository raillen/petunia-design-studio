//! Render snapshots: revisioned, immutable, explicitly ordered.

use crate::primitive::RenderPrimitive;
use crate::resource::RenderResourceTable;
use serde::{Deserialize, Serialize};

/// Snapshot identity: the authorial revision it was compiled from.
/// The engine maps its own revision counter onto this value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SnapshotRevision(pub u64);

/// Quality class of one compilation. Interactive previews may relax
/// tolerances; exports use the maximum contract. Quality never
/// changes topology, order, blend modes or fill rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderQuality {
    InteractivePreview,
    Authoring,
    Export,
}

/// One evaluated snapshot: pages in order plus the resource table.
/// Paint order inside primitives is semantic when it represents
/// z-order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderSnapshot {
    pub revision: SnapshotRevision,
    pub pages: Vec<RenderPage>,
    pub resources: RenderResourceTable,
}

/// One page of evaluated primitives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderPage {
    pub page: petunia_core::PageId,
    pub size: petunia_core::Size2,
    pub primitives: Vec<RenderPrimitive>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_serialization_round_trip() {
        let snapshot = RenderSnapshot {
            revision: SnapshotRevision(12),
            pages: Vec::new(),
            resources: RenderResourceTable::default(),
        };
        let json = serde_json::to_string(&snapshot).expect("serializable");
        let back: RenderSnapshot = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back.revision, SnapshotRevision(12));
    }
}
