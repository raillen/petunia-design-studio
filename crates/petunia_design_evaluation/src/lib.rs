#![forbid(unsafe_code)]

//! Derived-data cache with explicit invalidation (09.4 contract sketch).
//!
//! The evaluator never mutates the document. Every [`ChangeSet`] applied
//! through history bumps the generation and dirties the cache; the next
//! [`Evaluator::evaluate`] recomputes exactly once.

use petunia_design_document::{ChangeSet, Document};

/// Cheap summary of derived document state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocSummary {
    /// Number of surfaces.
    pub surfaces: usize,
    /// Total objects across surfaces.
    pub objects: usize,
    /// Cache generation that produced this summary.
    pub generation: u64,
}

/// Generation-tracked evaluator with hit/miss accounting for tests.
#[derive(Debug, Default)]
pub struct Evaluator {
    generation: u64,
    dirty: bool,
    cached_fingerprint: u64,
    cached_summary: DocSummary,
    /// Recomputations performed (evidence for invalidation tests).
    pub recomputes: u64,
    /// Cache hits served.
    pub hits: u64,
}

impl Evaluator {
    /// Creates a fresh evaluator (generation 0, dirty).
    #[must_use]
    pub fn new() -> Self {
        Self {
            dirty: true,
            ..Self::default()
        }
    }

    /// Current generation. Bumped once per [`Self::note_changes`].
    #[must_use]
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Records that a change set was applied. Empty sets still bump the
    /// generation: callers must not fabricate no-op mutations to probe it.
    pub fn note_changes(&mut self, _changes: &ChangeSet) {
        self.generation += 1;
        self.dirty = true;
    }

    /// Returns the derived summary, recomputing only when dirty or when the
    /// document fingerprint changed under the evaluator.
    pub fn evaluate(&mut self, document: &Document) -> DocSummary {
        let fingerprint = fingerprint(document);
        if !self.dirty && fingerprint == self.cached_fingerprint {
            self.hits += 1;
            return self.cached_summary.clone();
        }
        self.recomputes += 1;
        self.dirty = false;
        self.cached_fingerprint = fingerprint;
        self.cached_summary = DocSummary {
            surfaces: document.surfaces().len(),
            objects: document.surfaces().iter().map(|s| s.objects().len()).sum(),
            generation: self.generation,
        };
        self.cached_summary.clone()
    }
}

/// FNV-1a 64 over canonical JSON. Deterministic within one schema version;
/// not a cryptographic digest.
fn fingerprint(document: &Document) -> u64 {
    let json = document.to_json().unwrap_or_default();
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in json.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_design_document::{DocumentMutator, DocumentObject};
    use petunia_design_foundation::IdGenerator;

    fn one_object_doc() -> (Document, IdGenerator) {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface, "Page").unwrap();
        mutator
            .add_object(surface, DocumentObject::new(gen.next_object(), "Rect"))
            .unwrap();
        (doc, gen)
    }

    #[test]
    fn second_evaluate_is_a_cache_hit() {
        let (doc, _) = one_object_doc();
        let mut evaluator = Evaluator::new();
        let first = evaluator.evaluate(&doc);
        assert_eq!(first.objects, 1);
        let second = evaluator.evaluate(&doc);
        assert_eq!(second, first);
        assert_eq!(evaluator.hits, 1);
        assert_eq!(evaluator.recomputes, 1);
    }

    #[test]
    fn changes_invalidate_exactly_once() {
        let (mut doc, mut gen) = one_object_doc();
        let mut evaluator = Evaluator::new();
        evaluator.evaluate(&doc);
        let surface = doc.surfaces()[0].id;
        let changes = DocumentMutator::new(&mut doc)
            .add_object(surface, DocumentObject::new(gen.next_object(), "Circle"))
            .unwrap();
        evaluator.note_changes(&changes);
        let summary = evaluator.evaluate(&doc);
        assert_eq!(summary.objects, 2);
        assert_eq!(evaluator.recomputes, 2);
        // Steady state again: hits resume.
        evaluator.evaluate(&doc);
        assert_eq!(evaluator.hits, 1);
    }
}
