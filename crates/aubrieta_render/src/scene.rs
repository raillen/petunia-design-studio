//! Scene extraction and backend port.

use aubrieta_document::Document;
use aubrieta_foundation::{AubrietaError, SurfaceId};

/// One surface's renderable fragment. Geometry per object arrives with the
/// vector-engine wave; the fragment contract is stable already.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SceneFragment {
    /// Source surface.
    pub surface: SurfaceId,
    /// Surface label snapshot.
    pub surface_name: String,
    /// Objects on this surface.
    pub object_count: usize,
    /// Distinct semantic fill tokens referenced (`none` when absent).
    pub fills: Vec<String>,
}

/// Rebuildable scene: full extraction from a document, no cached pixels.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scene {
    /// Fragments in document order.
    pub fragments: Vec<SceneFragment>,
}

impl Scene {
    /// Extracts a scene from a document. Total: reads only, never mutates.
    #[must_use]
    pub fn extract(document: &Document) -> Self {
        let fragments = document
            .surfaces
            .iter()
            .map(|surface| {
                let mut fills: Vec<String> = surface
                    .objects
                    .iter()
                    .map(|o| o.fill.clone().unwrap_or_else(|| "none".to_string()))
                    .collect();
                fills.sort();
                fills.dedup();
                SceneFragment {
                    surface: surface.id,
                    surface_name: surface.name.clone(),
                    object_count: surface.objects.len(),
                    fills,
                }
            })
            .collect();
        Self { fragments }
    }

    /// Total objects across fragments.
    #[must_use]
    pub fn object_count(&self) -> usize {
        self.fragments.iter().map(|f| f.object_count).sum()
    }
}

/// Render backend port. GPU implementations arrive in a later wave.
pub trait RenderBackend {
    /// Renders a scene to a human-readable summary (headless) or presents it.
    fn render(&self, scene: &Scene) -> Result<String, AubrietaError>;
}

/// Deterministic headless backend used by tests, CLI and conformance.
#[derive(Debug, Default)]
pub struct HeadlessSummaryBackend;

impl RenderBackend for HeadlessSummaryBackend {
    fn render(&self, scene: &Scene) -> Result<String, AubrietaError> {
        let mut out = format!(
            "scene fragments={} objects={}",
            scene.fragments.len(),
            scene.object_count()
        );
        for fragment in &scene.fragments {
            out.push_str(&format!(
                "\n- {} objects={} fills=[{}]",
                fragment.surface,
                fragment.object_count,
                fragment.fills.join(",")
            ));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aubrieta_document::{DocumentMutator, DocumentObject};
    use aubrieta_foundation::IdGenerator;

    #[test]
    fn extraction_is_rebuildable_and_total() {
        let mut gen = IdGenerator::new();
        let mut doc = Document::new();
        let surface = gen.next_surface();
        let mut mutator = DocumentMutator::new(&mut doc);
        mutator.add_surface(surface, "Page").unwrap();
        let mut object = DocumentObject::new(gen.next_object(), "Rect");
        object.fill = Some("aubrieta.red/500".to_string());
        mutator.add_object(surface, object).unwrap();

        let first = Scene::extract(&doc);
        let second = Scene::extract(&doc);
        assert_eq!(first, second);
        assert_eq!(first.object_count(), 1);

        let summary = HeadlessSummaryBackend.render(&first).unwrap();
        assert!(summary.contains("objects=1"), "{summary}");
        assert!(summary.contains("aubrieta.red/500"), "{summary}");
    }
}
