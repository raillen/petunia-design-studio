//! Resource resolution, persistence and immutable evaluation at the session boundary.
use super::StudioSession;
use crate::{ContextStack, Result};
use petunia_engine::persistence::{self, SaveCompletion, SavePolicy, SaveSnapshot};
use petunia_engine::ptnd::PackageLimits;
use petunia_engine::spatial::SnapSettings;
use petunia_engine::{compile, DocumentRevision, HistoryDescription};
use petunia_render_model::{CompileWarning, RenderQuality, RenderSnapshot};
use std::path::Path;
use std::sync::Arc;
impl StudioSession {
    fn check_resource_budget(&self, extra_bytes: usize, extra_pixels: usize) -> Result<()> {
        let images = self.images.values().fold(0usize, |total, image| {
            total.saturating_add(image.pixels.len().saturating_mul(16))
        });
        let retained = self
            .blobs
            .retained_bytes()
            .saturating_mul(2)
            .saturating_add(images);
        if retained
            .saturating_add(extra_bytes.saturating_mul(2))
            .saturating_add(extra_pixels.saturating_mul(16))
            > 256 << 20
        {
            return Err(petunia_engine::EngineError::Limit(
                "session resource retention budget exceeded".into(),
            )
            .into());
        }
        Ok(())
    }
    /// Collect bytes alongside the dependency closure for native copy/paste.
    pub fn copy_selection(&self) -> Result<NativeClipboard> {
        use petunia_engine::fragments::{collect_fragment, FragmentMetadata};
        let fragment = collect_fragment(
            &self.document,
            self.selection.objects(),
            FragmentMetadata {
                source_document: self.document.id,
                source_revision: self.revision().0,
            },
        )?;
        let mut blobs = std::collections::BTreeMap::new();
        for record in &fragment.resources {
            if let Some(bytes) = self.blobs.get(record.id) {
                blobs.insert(record.id, bytes.as_ref().clone());
            } else if matches!(record.source, petunia_core::ResourceSource::Embedded { .. }) {
                return Err(crate::UiError::State(
                    "clipboard embedded resource bytes unavailable".into(),
                ));
            }
        }
        Ok(NativeClipboard { fragment, blobs })
    }

    /// Validate resources first; one transaction inserts all remapped authorial entities.
    pub fn paste(&mut self, clipboard: &NativeClipboard) -> Result<()> {
        use petunia_engine::fragments::{fragment_into_parent_ops, remap_fragment, IdRemapping};
        let mapping = IdRemapping::fresh_for_fragment(&clipboard.fragment);
        let fragment = remap_fragment(&clipboard.fragment, &mapping);
        let operations = fragment_into_parent_ops(
            &self.document,
            &fragment,
            petunia_core::ParentRef::Page(self.active_page),
        )?;
        let mut resources = Vec::new();
        for record in &clipboard.fragment.resources {
            if let Some(bytes) = clipboard.blobs.get(&record.id) {
                if record
                    .content_hash
                    .as_ref()
                    .is_some_and(|hash| *hash != petunia_core::ContentHash::new(bytes))
                {
                    return Err(crate::UiError::State(
                        "clipboard resource hash mismatch".into(),
                    ));
                }
                let image = if record.kind == petunia_core::ResourceKind::Image {
                    Some(Self::decode_image(bytes)?)
                } else {
                    None
                };
                let id = mapping.resources.get(&record.id).copied().ok_or_else(|| {
                    crate::UiError::State("clipboard resource mapping missing".into())
                })?;
                resources.push((id, record.kind, bytes.clone(), image));
            } else if matches!(record.source, petunia_core::ResourceSource::Embedded { .. }) {
                return Err(crate::UiError::State(
                    "clipboard embedded bytes missing".into(),
                ));
            }
        }
        let bytes = resources.iter().map(|(_, _, bytes, _)| bytes.len()).sum();
        let pixels = resources
            .iter()
            .filter_map(|(_, _, _, image)| image.as_ref())
            .map(|image| image.pixels.len())
            .sum();
        self.check_resource_budget(bytes, pixels)?;
        self.commit_request(
            petunia_engine::TransactionRequest {
                command_id: petunia_engine::CommandId::new_v4(),
                operations,
                merge_key: None,
            },
            HistoryDescription::InsertObjects,
        )?;
        for (id, kind, bytes, image) in resources {
            if kind == petunia_core::ResourceKind::Font {
                self.fonts.load_data(bytes.clone());
            }
            self.blobs.insert(id, bytes);
            if let Some(image) = image {
                self.images.insert(id, Arc::new(image));
            }
        }
        self.selection.set_objects(fragment.roots);
        self.sync_layers();
        Ok(())
    }
    /// The edited page is session state; switching it does not create history.
    pub fn set_active_page(&mut self, page: petunia_core::PageId) -> Result<()> {
        if self.document.scene.page_roots(page).is_none() {
            return Err(crate::error::UiError::State("unknown page".into()));
        }
        self.active_page = page;
        self.selection.clear();
        self.contexts = ContextStack::new();
        self.snap_latch.set(None);
        Ok(())
    }

    #[must_use]
    pub fn active_page(&self) -> petunia_core::PageId {
        self.active_page
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.history.is_dirty()
    }

    /// Current file destination is session metadata, not document data.
    #[must_use]
    pub fn destination_path(&self) -> Option<&Path> {
        self.destination.as_ref().map(|(path, _)| path.as_path())
    }

    /// Resolve fonts explicitly at the adapter boundary, never during compilation.
    pub fn fonts_mut(&mut self) -> &mut petunia_engine::text::FontRegistry {
        &mut self.fonts
    }

    pub fn snap_settings_mut(&mut self) -> &mut SnapSettings {
        &mut self.snap_settings
    }

    /// Supply embedded or explicitly authorized linked bytes to the runtime.
    pub fn set_resource_bytes(
        &mut self,
        id: petunia_core::ResourceId,
        bytes: Vec<u8>,
    ) -> Result<()> {
        let resource = self
            .document
            .resources
            .get(id)
            .ok_or_else(|| crate::error::UiError::State("unknown resource".into()))?;
        let kind = resource.kind;
        if resource
            .content_hash
            .as_ref()
            .is_some_and(|hash| *hash != petunia_core::ContentHash::new(&bytes))
        {
            return Err(crate::error::UiError::State(
                "resource content hash mismatch".into(),
            ));
        }
        let image = if kind == petunia_core::ResourceKind::Image {
            Some(Self::decode_image(&bytes)?)
        } else {
            None
        };
        self.check_resource_budget(
            bytes.len(),
            image.as_ref().map_or(0, |image| image.pixels.len()),
        )?;
        if kind == petunia_core::ResourceKind::Font {
            self.fonts.load_data(bytes.clone());
        }
        if let Some(image) = image {
            self.images.insert(id, Arc::new(image));
        }
        self.blobs.insert(id, bytes);
        Ok(())
    }

    fn decode_image(bytes: &[u8]) -> Result<petunia_render_model::image::ResolvedImage> {
        let decoded = petunia_engine::io::import_raster_image(
            bytes,
            &petunia_engine::io::IoLimits {
                max_embedded_bytes: 32 << 20,
                ..petunia_engine::io::IoLimits::default()
            },
        )?;
        let linear = |byte: u8| {
            let value = f32::from(byte) / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        let pixels = decoded
            .rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| petunia_render_model::RenderColor {
                r: linear(p[0]),
                g: linear(p[1]),
                b: linear(p[2]),
                a: f32::from(p[3]) / 255.0,
            })
            .collect();
        petunia_render_model::image::ResolvedImage::new(decoded.width, decoded.height, pixels)
            .ok_or_else(|| crate::error::UiError::State("invalid decoded image dimensions".into()))
    }

    /// Immutable compiler inputs: only resources present in this authorial revision.
    #[must_use]
    pub fn compile_snapshot(
        &self,
        quality: RenderQuality,
    ) -> (RenderSnapshot, Vec<CompileWarning>) {
        let providers = compile::CompileProviders {
            fonts: Some(&self.fonts),
            images: self
                .images
                .iter()
                .filter(|(id, _)| self.document.resources.get(**id).is_some())
                .map(|(id, image)| (*id, Arc::clone(image)))
                .collect(),
            color_profiles: self
                .document
                .resources
                .iter()
                .filter_map(|(id, record)| {
                    (record.kind == petunia_core::ResourceKind::IccProfile)
                        .then(|| {
                            self.blobs.get(id).map(|bytes| {
                                (
                                    petunia_core::ColorSpaceRef::EmbeddedIcc(id),
                                    Arc::from(bytes.as_slice()),
                                )
                            })
                        })
                        .flatten()
                })
                .collect(),
            ..compile::CompileProviders::default()
        };
        compile::compile_document_with_providers(
            &self.document,
            self.revision(),
            quality,
            &providers,
        )
    }

    #[must_use]
    pub fn save_snapshot(&self) -> SaveSnapshot {
        SaveSnapshot {
            document: self.document.clone(),
            revision: self.revision(),
            blobs: self.blobs.clone(),
            preview: self.preview.clone(),
            extensions: self.extensions.clone(),
        }
    }

    /// A worker can return completion after newer edits without clearing their dirty state.
    pub fn acknowledge_save(&mut self, completion: SaveCompletion) -> Result<()> {
        if completion.document_id != self.document.id {
            return Err(crate::error::UiError::State(
                "save belongs to another document".into(),
            ));
        }
        self.history.mark_saved(completion.revision)?;
        self.destination = Some((completion.path, completion.fingerprint));
        Ok(())
    }

    pub fn save_as(&mut self, path: &Path, policy: SavePolicy) -> Result<()> {
        let completion = persistence::save_snapshot(
            &self.save_snapshot(),
            path,
            &policy,
            &PackageLimits::default(),
        )?;
        self.acknowledge_save(completion)
    }

    pub fn save(&mut self) -> Result<()> {
        let (path, baseline) = self
            .destination
            .clone()
            .ok_or_else(|| crate::error::UiError::State("choose a destination first".into()))?;
        self.save_as(&path, SavePolicy::IfUnchanged(baseline))
    }

    /// Open into a temporary session first: malformed resources cannot destroy the live document.
    pub fn open(&mut self, path: &Path) -> Result<()> {
        let opened = persistence::open_document(path, &PackageLimits::default())?;
        let mut replacement = Self::new(opened.package.document.metadata.title.clone());
        replacement.document = opened.package.document;
        replacement.active_page = replacement.document.scene.default_page();
        for (id, bytes) in opened.package.blobs {
            replacement.set_resource_bytes(id, bytes)?;
        }
        replacement.preview = opened.package.preview;
        replacement.extensions = opened.package.extensions;
        replacement.destination = Some((opened.path, opened.fingerprint));
        replacement.view = self.view;
        replacement.workspace = self.workspace.clone();
        replacement.shortcuts = self.shortcuts.clone();
        replacement.layers_revision = DocumentRevision(u64::MAX);
        replacement.sync_layers();
        *self = replacement;
        Ok(())
    }

    /// The PNG bytes and document surface replacement cross the writer boundary together.
    pub fn commit_raster_edit(&mut self, edit: petunia_engine::brush::RasterEdit) -> Result<()> {
        let image = Self::decode_image(&edit.png)?;
        self.check_resource_budget(edit.png.len(), image.pixels.len())?;
        self.commit_request(edit.request, HistoryDescription::EditObjects)?;
        self.blobs.insert(edit.resource, edit.png);
        self.images.insert(edit.resource, Arc::new(image));
        Ok(())
    }
}

/// Clipboard payload remains session state and never serializes a whole Document.
#[derive(Debug, Clone)]
pub struct NativeClipboard {
    pub fragment: petunia_engine::fragments::DocumentFragment,
    pub blobs: std::collections::BTreeMap<petunia_core::ResourceId, Vec<u8>>,
}
