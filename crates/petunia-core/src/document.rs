//! Top-level Petunia document: aggregate root, pages and registries.
//!
//! [`Document`] gathers configuration, pages, the scene graph and
//! every shared registry, but executes no algorithms and knows no UI,
//! renderer or filesystem. Session state (selection, tools, zoom,
//! clipboard, job progress) never enters this model.

use crate::appearance::{Appearance, AppearanceKind, Paint, PatternSource};
use crate::color::{ColorSpaceRef, DocumentColorSpec};
use crate::error::{CoreError, Result};
use crate::guides::{GridRegistry, GuideRegistry, SliceRegistry};
use crate::id::{DocumentId, ObjectId, PageId, ResourceId, SpreadId};
use crate::math::{Insets, Point, Size2};
use crate::paint::{ColorSource, SwatchRegistry};
use crate::resources::ResourceRegistry;
use crate::scene::{SceneGraph, SceneItem};
use crate::styles::{StyleDefinition, StyleRegistry};
use crate::symbols::SymbolRegistry;
use crate::text::{validate_flow_links, TextObject};
use crate::units::Unit;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Page geometry: size plus printable margins and bleed.
///
/// Sizes are finite and positive; margins and bleed are finite and
/// stay non-negative in v0.1. Negative values only enter through a
/// future feature with explicit semantics.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PageSpec {
    pub size: Size2,
    pub margins: Insets,
    pub bleed: Insets,
}

impl PageSpec {
    /// Build a page spec, enforcing the v0.1 range rules.
    pub fn new(size: Size2, margins: Insets, bleed: Insets) -> Result<Self> {
        if !size.width.is_finite() || !size.height.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite page size rejected: {size:?}"
            )));
        }
        if size.width <= 0.0 || size.height <= 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "page size must stay positive, got {size:?}"
            )));
        }
        for (label, insets) in [("margins", margins), ("bleed", bleed)] {
            for side in [insets.left, insets.top, insets.right, insets.bottom] {
                if !side.is_finite() || side < 0.0 {
                    return Err(CoreError::InvariantViolation(format!(
                        "page {label} must stay finite and non-negative, got {side}"
                    )));
                }
            }
        }
        Ok(Self {
            size,
            margins,
            bleed,
        })
    }

    /// Re-check the construction invariants on any instance,
    /// including deserialized ones.
    pub fn validate(&self) -> Result<()> {
        Self::new(self.size, self.margins, self.bleed).map(|_| ())
    }
}

/// One editorial page: identity, name and geometry. Root ordering
/// lives in the scene graph keyed by this same [`PageId`]; there is
/// exactly one owner for the order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Page {
    pub id: PageId,
    pub name: String,
    pub spec: PageSpec,
}

impl Page {
    /// Build a page around an already-validated spec.
    #[must_use]
    pub fn new(id: PageId, name: impl Into<String>, spec: PageSpec) -> Self {
        Self {
            id,
            name: name.into(),
            spec,
        }
    }
}

/// Ordered page collection of a document. Lookup by identity;
/// spreads carry the editorial arrangement separately.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PageCollection {
    #[serde(deserialize_with = "crate::serialization::deserialize_unique_btree_map")]
    pages: BTreeMap<PageId, Page>,
}

impl PageCollection {
    /// Empty collection. A valid document always holds at least one
    /// page; [`Document::new`] creates it.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace a page.
    pub fn insert(&mut self, page: Page) {
        self.pages.insert(page.id, page);
    }

    /// Look up a page by identity.
    #[must_use]
    pub fn get(&self, id: PageId) -> Option<&Page> {
        self.pages.get(&id)
    }

    /// Mutably borrow a page by identity.
    pub fn get_mut(&mut self, id: PageId) -> Option<&mut Page> {
        self.pages.get_mut(&id)
    }

    /// All page identities in deterministic order.
    #[must_use]
    pub fn ids(&self) -> Vec<PageId> {
        self.pages.keys().copied().collect()
    }

    /// Number of tracked pages.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pages.len()
    }

    /// True when no page is tracked.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }
}

/// Placement of one page inside a spread's editorial space. Scene
/// nodes stay in page-local coordinates; moving a page inside a
/// spread never rewrites them.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SpreadPagePlacement {
    pub page: PageId,
    pub origin: Point,
}

/// Editorial arrangement of pages. It organizes pages without
/// changing node ownership or page-local geometry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Spread {
    pub id: SpreadId,
    pub pages: Vec<SpreadPagePlacement>,
}

impl Spread {
    /// Placement origins must be finite; referenced pages are checked
    /// against the document during [`Document::validate`].
    pub fn new(id: SpreadId, pages: Vec<SpreadPagePlacement>) -> Result<Self> {
        for placement in &pages {
            if !placement.origin.x.is_finite() || !placement.origin.y.is_finite() {
                return Err(CoreError::InvariantViolation(format!(
                    "non-finite spread origin rejected: {:?}",
                    placement.origin,
                )));
            }
        }
        Ok(Self { id, pages })
    }
}

/// Authorial and editorial metadata. Timestamps mark save events,
/// never pointer moves; operational state stays out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: String,
    pub author: String,
    /// Milliseconds since the Unix epoch, set once at creation.
    pub created_unix_ms: u64,
    /// Milliseconds since the Unix epoch, updated on successful save.
    pub modified_unix_ms: u64,
    pub notes: String,
    /// Namespaced custom metadata.
    #[serde(deserialize_with = "crate::serialization::deserialize_unique_btree_map")]
    pub custom: BTreeMap<String, String>,
}

impl DocumentMetadata {
    /// Fresh metadata stamped with the current time.
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        let now = unix_millis();
        Self {
            title: title.into(),
            author: String::new(),
            created_unix_ms: now,
            modified_unix_ms: now,
            notes: String::new(),
            custom: BTreeMap::new(),
        }
    }

    /// Record a successful save. Preview, selection, cache rebuilds
    /// and temporary autosaves never call this on the main document.
    pub fn touch(&mut self) {
        self.modified_unix_ms = unix_millis();
    }
}

fn unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

/// Document canvas setup: display units, raster defaults and the
/// working color context. Changing these never reinterprets existing
/// colors silently; that takes explicit commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSetup {
    pub units: Unit,
    pub default_raster_dpi: f64,
    pub color: DocumentColorSpec,
    pub default_page: PageSpec,
}

impl Default for DocumentSetup {
    fn default() -> Self {
        Self {
            units: Unit::Px,
            default_raster_dpi: 72.0,
            color: DocumentColorSpec {
                rgb_working: crate::color::ColorSpaceRef::Builtin(
                    crate::color::BuiltinColorSpace::Srgb,
                ),
                cmyk_working: None,
                gray_working: None,
                rendering_intent: crate::color::RenderingIntent::RelativeColorimetric,
            },
            default_page: PageSpec {
                size: Size2 {
                    width: 1920.0,
                    height: 1080.0,
                },
                margins: Insets::ZERO,
                bleed: Insets::ZERO,
            },
        }
    }
}

/// In-memory document model: the authorial aggregate root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub metadata: DocumentMetadata,
    pub setup: DocumentSetup,
    pub pages: PageCollection,
    #[serde(deserialize_with = "crate::serialization::deserialize_unique_btree_map")]
    pub spreads: BTreeMap<SpreadId, Spread>,
    pub scene: SceneGraph,
    pub resources: ResourceRegistry,
    pub styles: StyleRegistry,
    pub symbols: SymbolRegistry,
    pub swatches: SwatchRegistry,
    pub guides: GuideRegistry,
    pub grids: GridRegistry,
    pub slices: SliceRegistry,
}

impl Document {
    #[must_use]
    pub fn new(title: impl Into<String>) -> Self {
        let scene = SceneGraph::new();
        let page_id = scene.default_page();
        let setup = DocumentSetup::default();
        let mut pages = PageCollection::new();
        pages.insert(Page::new(page_id, "Page 1", setup.default_page));
        Self {
            id: DocumentId::new_v4(),
            metadata: DocumentMetadata::new(title),
            setup,
            pages,
            spreads: BTreeMap::new(),
            scene,
            resources: ResourceRegistry::new(),
            styles: StyleRegistry::new(),
            symbols: SymbolRegistry::new(),
            swatches: SwatchRegistry::new(),
            guides: GuideRegistry::new(),
            grids: GridRegistry::new(),
            slices: SliceRegistry::new(),
        }
    }

    /// Canvas size of the default page, falling back to the setup
    /// default when the page entry is missing (never on a valid
    /// document; renderers must not fail on metadata alone).
    #[must_use]
    pub fn default_page_size(&self) -> Size2 {
        self.pages
            .get(self.scene.default_page())
            .map(|page| page.spec.size)
            .unwrap_or(self.setup.default_page.size)
    }

    /// Serializes the document through its versioned DTO: schema
    /// first, then canonical JSON. The domain model never defines the
    /// container strategy directly.
    pub fn to_json(&self) -> Result<String> {
        use crate::dto::DocumentDtoV1;
        Ok(serde_json::to_string_pretty(
            &DocumentDtoV1::from_document(self),
        )?)
    }

    /// Deserializes through structural validation, migration-free
    /// today, and full domain validation. Untrusted input never
    /// becomes a `Document` without passing both layers.
    pub fn from_json(json: &str) -> Result<Self> {
        Self::from_json_limited(json, u64::MAX)
    }

    /// Same as [`Document::from_json`], refusing inputs above
    /// `max_bytes` before any allocation beyond the input itself.
    /// Untrusted callers use this; the PTND loader enforces its own
    /// entry budgets first.
    pub fn from_json_limited(json: &str, max_bytes: u64) -> Result<Self> {
        use crate::dto::DocumentDtoV1;
        if json.len() as u64 > max_bytes {
            return Err(CoreError::InvariantViolation(format!(
                "document of {} bytes exceeds limit {max_bytes}",
                json.len(),
            )));
        }
        let dto: DocumentDtoV1 = serde_json::from_str(json)?;
        dto.into_document()
    }

    /// Full document validation: pages, registries, cross references,
    /// finite numbers and no dangling mandatory references.
    pub fn validate(&self) -> Result<()> {
        if self.pages.is_empty() {
            return Err(CoreError::InvariantViolation(
                "a valid document holds at least one page".to_string(),
            ));
        }
        // Pages and scene root lists describe the same ownership.
        for page_id in self.scene.page_ids() {
            if self.pages.get(page_id).is_none() {
                return Err(CoreError::DanglingReference(format!(
                    "scene page {page_id} has no page entry"
                )));
            }
        }
        for page_id in self.pages.ids() {
            if self.scene.page_roots(page_id).is_none() {
                return Err(CoreError::DanglingReference(format!(
                    "page {page_id} has no scene root list"
                )));
            }
            let page = self.pages.get(page_id).expect("checked above");
            page.spec.validate()?;
            if page.name.trim().is_empty() {
                return Err(CoreError::InvariantViolation(format!(
                    "page {page_id} needs a name"
                )));
            }
        }
        if !self.setup.default_raster_dpi.is_finite() || self.setup.default_raster_dpi <= 0.0 {
            return Err(CoreError::InvariantViolation(format!(
                "default raster DPI must stay finite and positive, got {}",
                self.setup.default_raster_dpi
            )));
        }
        self.check_color_space(&self.setup.color.rgb_working, "working RGB space")?;
        if let Some(space) = &self.setup.color.cmyk_working {
            self.check_color_space(space, "working CMYK space")?;
        }
        if let Some(space) = &self.setup.color.gray_working {
            self.check_color_space(space, "working gray space")?;
        }
        self.scene.validate()?;
        self.validate_spreads()?;
        self.validate_styles()?;
        self.validate_symbols()?;
        self.validate_swatches()?;
        self.validate_scene_refs()?;
        self.validate_guides_grids_slices()?;
        Ok(())
    }

    fn validate_spreads(&self) -> Result<()> {
        for (id, spread) in &self.spreads {
            if spread.id != *id {
                return Err(CoreError::InvariantViolation(format!(
                    "spread key {id} does not match spread {}",
                    spread.id,
                )));
            }
            for placement in &spread.pages {
                if self.pages.get(placement.page).is_none() {
                    return Err(CoreError::DanglingReference(format!(
                        "spread {id} places missing page {}",
                        placement.page,
                    )));
                }
                if !placement.origin.x.is_finite() || !placement.origin.y.is_finite() {
                    return Err(CoreError::InvariantViolation(format!(
                        "non-finite spread origin in {id}: {:?}",
                        placement.origin,
                    )));
                }
            }
        }
        Ok(())
    }

    fn validate_styles(&self) -> Result<()> {
        for (id, definition) in self.styles.iter() {
            match definition {
                StyleDefinition::Appearance(style) => {
                    self.check_appearance(&style.appearance, &format!("style {id}"))?;
                }
                StyleDefinition::Character(style) => {
                    style.validate()?;
                    style.font.validate()?;
                    if let Some(resource) = style.font.resource {
                        self.check_resource(resource, &format!("font of style {id}"))?;
                    }
                    self.check_color_source(&style.color, &format!("style {id} color"))?;
                }
                StyleDefinition::Paragraph(style) => {
                    style.validate()?;
                    if let Some(grid) = style.baseline_grid {
                        if self.grids.get(grid).is_none() {
                            return Err(CoreError::DanglingReference(format!(
                                "paragraph style {id} references missing grid {grid}"
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_symbols(&self) -> Result<()> {
        use std::collections::HashSet;
        for (id, definition) in self.symbols.iter() {
            let mut seen = HashSet::new();
            for root in &definition.roots {
                if !seen.insert(root) {
                    return Err(CoreError::InvariantViolation(format!(
                        "symbol {id} lists duplicate root {root}"
                    )));
                }
            }
        }
        for (id, instance) in self.symbol_instances() {
            let Some(definition) = self.symbols.get(instance.definition) else {
                return Err(CoreError::DanglingReference(format!(
                    "symbol instance {id} references missing definition {}",
                    instance.definition,
                )));
            };
            let _ = definition;
            for override_ in &instance.overrides {
                if self.scene.get_node(override_.target).is_none() {
                    return Err(CoreError::DanglingReference(format!(
                        "symbol override of {id} targets missing node {}",
                        override_.target,
                    )));
                }
                if let crate::symbols::SymbolOverrideValue::Resource(resource) = &override_.value {
                    self.check_resource(*resource, &format!("symbol override of {id}"))?;
                }
                if let crate::symbols::SymbolOverrideValue::Paint(source) = &override_.value {
                    self.check_color_source(source, &format!("symbol override of {id}"))?;
                }
            }
        }
        Ok(())
    }

    /// Scene items holding symbol instances, paired with their node id.
    fn symbol_instances(&self) -> Vec<(ObjectId, crate::symbols::SymbolInstance)> {
        // SceneGraph lookup is by identity; iteration over storage is
        // intentionally unavailable. Scene-level traversal helpers
        // arrive with the query slice; until then validation walks the
        // reachable roots below.
        let mut out = Vec::new();
        for roots in self.scene.root_lists() {
            let mut stack: Vec<ObjectId> = roots.to_vec();
            while let Some(id) = stack.pop() {
                let Some(node) = self.scene.get_node(id) else {
                    continue;
                };
                if let SceneItem::SymbolInstance(instance) = &node.item {
                    out.push((id, instance.clone()));
                }
                if let Some(children) = node.item.children() {
                    stack.extend(children.iter().copied());
                }
            }
        }
        out
    }

    fn validate_swatches(&self) -> Result<()> {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        for id in self.swatches.order() {
            if !seen.insert(id) {
                return Err(CoreError::InvariantViolation(format!(
                    "duplicate swatch {id} in palette order"
                )));
            }
            let Some(swatch) = self.swatches.get(*id) else {
                return Err(CoreError::DanglingReference(format!(
                    "palette order lists missing swatch {id}"
                )));
            };
            self.check_swatch_value(&swatch.value, &format!("swatch {id}"))?;
        }
        // Every entry must be reachable from the explicit order.
        for (id, swatch) in self.swatches.iter() {
            if !seen.contains(&id) {
                return Err(CoreError::InvariantViolation(format!(
                    "swatch {} ({}) is missing from the palette order",
                    id, swatch.name,
                )));
            }
        }
        Ok(())
    }

    fn check_swatch_value(&self, value: &crate::paint::SwatchValue, what: &str) -> Result<()> {
        match value {
            crate::paint::SwatchValue::Color(color) => self.check_color_value(color, what),
            crate::paint::SwatchValue::Gradient(gradient) => {
                gradient.validate()?;
                for stop in &gradient.stops {
                    self.check_color_source(&stop.color, &format!("{what} stop"))?;
                }
                Ok(())
            }
        }
    }

    fn validate_guides_grids_slices(&self) -> Result<()> {
        use crate::guides::{GridScope, GuideScope, SliceSource};
        for (id, guide) in self.guides.iter() {
            if !guide.position.is_finite() {
                return Err(CoreError::InvariantViolation(format!(
                    "non-finite guide position in {id}"
                )));
            }
            match guide.scope {
                GuideScope::Document => {}
                GuideScope::Page(page) => {
                    if self.pages.get(page).is_none() {
                        return Err(CoreError::DanglingReference(format!(
                            "guide {id} scopes missing page {page}"
                        )));
                    }
                }
                GuideScope::Artboard(object) => {
                    self.check_object(object, &format!("guide {id} scope"))?
                }
            }
        }
        for (id, grid) in self.grids.iter() {
            match grid.scope {
                GridScope::Document => {}
                GridScope::Page(page) => {
                    if self.pages.get(page).is_none() {
                        return Err(CoreError::DanglingReference(format!(
                            "grid {id} scopes missing page {page}"
                        )));
                    }
                }
                GridScope::Artboard(object) => {
                    self.check_object(object, &format!("grid {id} scope"))?
                }
            }
            if !grid.origin.x.is_finite() || !grid.origin.y.is_finite() {
                return Err(CoreError::InvariantViolation(format!(
                    "non-finite grid origin in {id}"
                )));
            }
            match &grid.spec {
                crate::guides::GridSpec::Affine(spec) => {
                    for basis in [spec.basis_u, spec.basis_v] {
                        if !basis.dx.is_finite() || !basis.dy.is_finite() {
                            return Err(CoreError::InvariantViolation(format!(
                                "non-finite grid basis in {id}"
                            )));
                        }
                        if basis.dx == 0.0 && basis.dy == 0.0 {
                            return Err(CoreError::InvariantViolation(format!(
                                "zero-length grid basis in {id}"
                            )));
                        }
                    }
                    if spec.subdivisions_u == 0 || spec.subdivisions_v == 0 {
                        return Err(CoreError::InvariantViolation(format!(
                            "grid {id} subdivisions start at one"
                        )));
                    }
                    let cross =
                        spec.basis_u.dx * spec.basis_v.dy - spec.basis_u.dy * spec.basis_v.dx;
                    if cross == 0.0 {
                        return Err(CoreError::InvariantViolation(format!(
                            "degenerate grid basis in {id}"
                        )));
                    }
                }
                crate::guides::GridSpec::Baseline(spec) => {
                    if !spec.spacing.is_finite() || spec.spacing <= 0.0 {
                        return Err(CoreError::InvariantViolation(format!(
                            "grid {id} needs finite positive baseline spacing"
                        )));
                    }
                    if !spec.offset.is_finite() {
                        return Err(CoreError::InvariantViolation(format!(
                            "non-finite baseline offset in {id}"
                        )));
                    }
                    if spec.subdivisions == 0 {
                        return Err(CoreError::InvariantViolation(format!(
                            "grid {id} subdivisions start at one"
                        )));
                    }
                }
                crate::guides::GridSpec::Perspective(spec) => {
                    if !spec.spacing.dx.is_finite()
                        || !spec.spacing.dy.is_finite()
                        || spec.spacing.dx <= 0.0
                        || spec.spacing.dy <= 0.0
                    {
                        return Err(CoreError::InvariantViolation(format!(
                            "grid {id} needs finite positive perspective spacing"
                        )));
                    }
                    if spec.subdivisions == 0 {
                        return Err(CoreError::InvariantViolation(format!(
                            "grid {id} subdivisions start at one"
                        )));
                    }
                    let [h00, h01, h02, h10, h11, h12, h20, h21, h22] = spec.transform.coefficients;
                    if spec.transform.coefficients.iter().any(|c| !c.is_finite()) {
                        return Err(CoreError::InvariantViolation(format!(
                            "non-finite homography in {id}"
                        )));
                    }
                    let det = h00 * (h11 * h22 - h12 * h21) - h01 * (h10 * h22 - h12 * h20)
                        + h02 * (h10 * h21 - h11 * h20);
                    if det == 0.0 {
                        return Err(CoreError::InvariantViolation(format!(
                            "degenerate homography in {id}"
                        )));
                    }
                }
            }
        }
        for (id, slice) in self.slices.iter() {
            if slice.name.trim().is_empty() {
                return Err(CoreError::InvariantViolation(format!(
                    "slice {id} needs a name"
                )));
            }
            if slice.presets.is_empty() {
                return Err(CoreError::InvariantViolation(format!(
                    "slice {id} needs at least one preset"
                )));
            }
            for preset in &slice.presets {
                if !preset.scale.is_finite() || preset.scale <= 0.0 {
                    return Err(CoreError::InvariantViolation(format!(
                        "slice {id} needs a finite positive export scale"
                    )));
                }
                self.check_color_space(&preset.color.target, &format!("slice {id} preset"))?;
            }
            match &slice.source {
                SliceSource::Page(page) => {
                    if self.pages.get(*page).is_none() {
                        return Err(CoreError::DanglingReference(format!(
                            "slice {id} sources missing page {page}"
                        )));
                    }
                }
                SliceSource::Artboard(object) | SliceSource::Object(object) => {
                    self.check_object(*object, &format!("slice {id} source"))?
                }
                SliceSource::Rect { page, rect } => {
                    if self.pages.get(*page).is_none() {
                        return Err(CoreError::DanglingReference(format!(
                            "slice {id} sources missing page {page}"
                        )));
                    }
                    for edge in [rect.x, rect.y, rect.width, rect.height] {
                        if !edge.is_finite() {
                            return Err(CoreError::InvariantViolation(format!(
                                "non-finite rect slice edge in {id}"
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_scene_refs(&self) -> Result<()> {
        let mut flows = Vec::new();
        for roots in self.scene.root_lists() {
            let mut stack: Vec<ObjectId> = roots.to_vec();
            while let Some(id) = stack.pop() {
                let Some(node) = self.scene.get_node(id) else {
                    continue;
                };
                // Appearance lives on path and shape items, never on
                // the node: every paint resolves through the stack.
                match &node.item {
                    SceneItem::Path(object) => {
                        self.check_appearance(&object.appearance, &format!("path {id}"))?;
                    }
                    SceneItem::Shape(object) => {
                        self.check_appearance(&object.appearance, &format!("shape {id}"))?;
                    }
                    SceneItem::Text(text) => {
                        crate::text::TextObject::validate(
                            &text.text,
                            &text.runs,
                            &text.paragraphs,
                        )?;
                        self.check_text_runs(text, id)?;
                        self.check_text_container(text, id)?;
                        flows.push((id, text.flow.next));
                    }
                    SceneItem::Image(image) => {
                        self.check_resource(image.resource, &format!("image {id}"))?;
                        if let Some(rect) = &image.source_rect {
                            crate::crop::ImageSourceRect::new(rect.min, rect.max)?;
                        }
                    }
                    SceneItem::PixelLayer(layer) => {
                        self.check_resource(
                            layer.surface.resource,
                            &format!("pixel surface of {id}"),
                        )?;
                    }
                    SceneItem::Trace(trace) => {
                        trace.spec.validate()?;
                        self.check_resource(trace.source, &format!("trace source of {id}"))?;
                    }
                    SceneItem::GeneratedVector(generated) => {
                        self.check_generator(&generated.generator, id)?;
                        self.check_appearance(&generated.appearance, &format!("generated {id}"))?;
                    }
                    SceneItem::Group(_) => {}
                    SceneItem::SymbolInstance(_) => {}
                }
                if let Some(children) = node.item.children() {
                    stack.extend(children.iter().copied());
                }
            }
        }
        validate_flow_links(&flows)?;
        for (owner, next) in &flows {
            if let Some(target) = next {
                let Some(node) = self.scene.get_node(*target) else {
                    return Err(CoreError::DanglingReference(format!(
                        "text flow of {owner} targets missing node {target}"
                    )));
                };
                if !matches!(node.item, SceneItem::Text(_)) {
                    return Err(CoreError::InvariantViolation(format!(
                        "text flow of {owner} targets non-text node {target}"
                    )));
                }
            }
        }
        Ok(())
    }

    fn check_text_runs(&self, text: &TextObject, owner: ObjectId) -> Result<()> {
        for run in &text.runs {
            let Some(definition) = self.styles.get(run.style.style) else {
                return Err(CoreError::DanglingReference(format!(
                    "text {owner} references missing style {}",
                    run.style.style,
                )));
            };
            if !matches!(definition, StyleDefinition::Character(_)) {
                return Err(CoreError::InvariantViolation(format!(
                    "text {owner} run needs a character style, got {:?}",
                    definition.kind(),
                )));
            }
        }
        for paragraph in &text.paragraphs {
            let Some(definition) = self.styles.get(paragraph.style.style) else {
                return Err(CoreError::DanglingReference(format!(
                    "text {owner} references missing paragraph style {}",
                    paragraph.style.style,
                )));
            };
            if !matches!(definition, StyleDefinition::Paragraph(_)) {
                return Err(CoreError::InvariantViolation(format!(
                    "text {owner} paragraph needs a paragraph style, got {:?}",
                    definition.kind(),
                )));
            }
        }
        Ok(())
    }

    fn check_text_container(&self, text: &TextObject, owner: ObjectId) -> Result<()> {
        match &text.container {
            crate::text::TextContainer::Artistic => Ok(()),
            crate::text::TextContainer::Frame(spec) => {
                if !spec.size.width.is_finite()
                    || !spec.size.height.is_finite()
                    || spec.size.width <= 0.0
                    || spec.size.height <= 0.0
                {
                    return Err(CoreError::InvariantViolation(format!(
                        "text {owner} frame needs a finite positive size, got {:?}",
                        spec.size,
                    )));
                }
                Ok(())
            }
            crate::text::TextContainer::OnPath(reference) => {
                if !reference.start_offset.is_finite() {
                    return Err(CoreError::InvariantViolation(format!(
                        "text {owner} path offset must stay finite"
                    )));
                }
                let Some(target) = self.scene.get_node(reference.target) else {
                    return Err(CoreError::DanglingReference(format!(
                        "text {owner} follows missing path {}",
                        reference.target,
                    )));
                };
                if !matches!(target.item, SceneItem::Path(_) | SceneItem::Shape(_)) {
                    return Err(CoreError::InvariantViolation(format!(
                        "text {owner} follows non-path node {}",
                        reference.target,
                    )));
                }
                Ok(())
            }
        }
    }

    fn check_generator(
        &self,
        generator: &crate::generated::GeneratorSpec,
        owner: ObjectId,
    ) -> Result<()> {
        match generator {
            crate::generated::GeneratorSpec::QrCode(spec) => {
                if let crate::generated::QrVersionPolicy::Fixed(version) = spec.version {
                    if !(1..=40).contains(&version) {
                        return Err(CoreError::InvariantViolation(format!(
                            "QR of {owner} needs version 1..=40, got {version}"
                        )));
                    }
                }
                if let crate::generated::QrMaskPolicy::Fixed(mask) = spec.mask {
                    if mask > 7 {
                        return Err(CoreError::InvariantViolation(format!(
                            "QR of {owner} needs mask 0..=7, got {mask}"
                        )));
                    }
                }
                let crate::generated::QrPayload::Text(payload) = &spec.payload;
                if payload.is_empty() {
                    return Err(CoreError::InvariantViolation(format!(
                        "QR of {owner} needs a non-empty payload"
                    )));
                }
                Ok(())
            }
            crate::generated::GeneratorSpec::Barcode(spec) => {
                if spec.data.is_empty() {
                    return Err(CoreError::InvariantViolation(format!(
                        "barcode of {owner} needs data"
                    )));
                }
                if !spec.quiet_zone_modules.is_finite() || spec.quiet_zone_modules < 0.0 {
                    return Err(CoreError::InvariantViolation(format!(
                        "barcode of {owner} needs a finite non-negative quiet zone"
                    )));
                }
                if !spec.bar_height_modules.is_finite() || spec.bar_height_modules <= 0.0 {
                    return Err(CoreError::InvariantViolation(format!(
                        "barcode of {owner} needs a finite positive bar height"
                    )));
                }
                Ok(())
            }
        }
    }

    fn check_appearance(&self, appearance: &Appearance, what: &str) -> Result<()> {
        appearance.validate()?;
        for item in &appearance.items {
            match &item.kind {
                AppearanceKind::Fill(paint) => self.check_paint(paint, &format!("{what} fill"))?,
                AppearanceKind::Stroke(style) => {
                    self.check_paint(
                        &Paint::Solid(style.paint.clone()),
                        &format!("{what} stroke"),
                    )?;
                    for marker in [style.start_marker, style.end_marker].iter().flatten() {
                        if self.scene.get_node(*marker).is_none() {
                            return Err(CoreError::DanglingReference(format!(
                                "{what} stroke references missing marker {marker}"
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn check_paint(&self, paint: &Paint, what: &str) -> Result<()> {
        match paint {
            Paint::Solid(source) => self.check_color_source(source, what),
            Paint::LinearGradient(gradient) | Paint::RadialGradient(gradient) => {
                gradient.validate()?;
                for stop in &gradient.stops {
                    self.check_color_source(&stop.color, &format!("{what} stop"))?;
                }
                Ok(())
            }
            Paint::Pattern(pattern) => {
                match &pattern.source {
                    PatternSource::Raster(resource) => {
                        self.check_resource(*resource, &format!("{what} pattern"))?
                    }
                    PatternSource::Vector(object) => {
                        self.check_object(*object, &format!("{what} pattern"))?
                    }
                }
                if !pattern.transform.a.is_finite() {
                    return Err(CoreError::InvariantViolation(format!(
                        "{what} pattern transform must stay finite"
                    )));
                }
                Ok(())
            }
        }
    }

    fn check_color_source(&self, source: &ColorSource, what: &str) -> Result<()> {
        match source {
            ColorSource::Value(color) => self.check_color_value(color, what),
            ColorSource::Swatch(id) => {
                if self.swatches.get(*id).is_none() {
                    return Err(CoreError::DanglingReference(format!(
                        "{what} references missing swatch {id}"
                    )));
                }
                Ok(())
            }
        }
    }

    fn check_color_value(&self, color: &crate::color::ColorValue, what: &str) -> Result<()> {
        match color {
            crate::color::ColorValue::Process(process) => {
                process.validate()?;
                self.check_color_space(&process.space, what)
            }
            crate::color::ColorValue::Spot(reference) => {
                if !reference.tint.is_finite() || !(0.0..=1.0).contains(&reference.tint) {
                    return Err(CoreError::InvariantViolation(format!(
                        "{what} spot tint out of 0..=1: {}",
                        reference.tint
                    )));
                }
                // Spot ink definitions have no dedicated registry in
                // this revision; existence stays unchecked and is
                // recorded as follow-up work.
                Ok(())
            }
        }
    }

    fn check_color_space(&self, space: &ColorSpaceRef, what: &str) -> Result<()> {
        if let ColorSpaceRef::EmbeddedIcc(resource) = space {
            self.check_resource(*resource, &format!("{what} ICC profile"))?;
        }
        Ok(())
    }

    fn check_resource(&self, id: ResourceId, what: &str) -> Result<()> {
        if self.resources.get(id).is_none() {
            return Err(CoreError::DanglingReference(format!(
                "{what} references missing resource {id}"
            )));
        }
        Ok(())
    }

    fn check_object(&self, id: ObjectId, what: &str) -> Result<()> {
        if self.scene.get_node(id).is_none() {
            return Err(CoreError::DanglingReference(format!(
                "{what} references missing object {id}"
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crop::{BindingSourceUse, ClipBinding};
    use crate::id::StyleId;
    use crate::path::VectorPath;
    use crate::scene::{ParentRef, SceneNode};

    fn document_with_box() -> (Document, ObjectId) {
        let mut document = Document::new("Poster");
        let page = document.scene.default_page();
        let node = SceneNode::new_path(
            "Rectangle",
            VectorPath::rect(10.0, 10.0, 200.0, 100.0),
            ParentRef::Page(page),
        );
        let id = node.id;
        document.scene.insert_node(node);
        (document, id)
    }

    #[test]
    fn new_document_is_valid_with_one_page() {
        let document = Document::new("Poster");
        assert_eq!(document.pages.len(), 1);
        assert_eq!(
            document.scene.page_roots(document.scene.default_page()),
            Some([].as_slice()),
        );
        assert!(document.validate().is_ok());
        assert_eq!(document.metadata.title, "Poster");
    }

    #[test]
    fn document_roundtrip_json_preserves_geometry() {
        let (document, id) = document_with_box();
        let json = document.to_json().expect("to_json succeeds");
        let restored = Document::from_json(&json).expect("from_json succeeds");

        assert_eq!(document.id, restored.id);
        assert_eq!(document.metadata.title, restored.metadata.title);
        assert_eq!(document.scene.len(), restored.scene.len());
        assert_eq!(restored.scene.parent_of(id), document.scene.parent_of(id));
    }

    #[test]
    fn page_spec_rejects_degenerate_geometry() {
        let size = Size2::new(0.0, 10.0).expect("zero is representable");
        assert!(PageSpec::new(size, Insets::ZERO, Insets::ZERO).is_err());
        let size = Size2::new(10.0, 10.0).expect("valid");
        let margins = Insets::new(-1.0, 0.0, 0.0, 0.0).expect("insets allow negatives");
        assert!(PageSpec::new(size, margins, Insets::ZERO).is_err());
    }

    #[test]
    fn dangling_clip_source_fails_validation() {
        let (mut document, target) = document_with_box();
        let missing = ObjectId::new_v4();
        document.scene.get_node_mut(target).expect("target").clip = Some(
            ClipBinding::new(target, missing, BindingSourceUse::BindingOnly).expect("binding"),
        );
        assert!(document.validate().is_err());
    }

    #[test]
    fn missing_style_reference_fails_validation() {
        use crate::text::{CharacterStyleRef, TextFlow, TextObject, TextRun};
        let mut document = Document::new("Type");
        let page = document.scene.default_page();
        let text = TextObject::new(
            "Hi".to_string(),
            vec![TextRun {
                range: crate::text::TextRange::new(0, 2, "Hi").expect("range"),
                style: CharacterStyleRef {
                    style: StyleId::new_v4(),
                },
            }],
            Vec::new(),
            crate::text::TextContainer::Artistic,
            TextFlow { next: None },
        )
        .expect("text");
        let node = SceneNode {
            id: ObjectId::new_v4(),
            name: "text".to_string(),
            parent: ParentRef::Page(page),
            visible: true,
            locked: false,
            transform: crate::math::Transform2D::IDENTITY,
            opacity: 1.0,
            clip: None,
            mask: None,
            item: SceneItem::Text(text),
        };
        document.scene.insert_node(node);
        assert!(document.validate().is_err());
    }

    #[test]
    fn missing_slice_source_fails_validation() {
        use crate::guides::{
            ExportColorOptions, ExportFormat, ExportSlice, SliceExportPreset, SliceSource,
        };
        let mut document = Document::new("Slices");
        let preset = SliceExportPreset::new(
            ExportFormat::Png,
            2.0,
            ExportColorOptions {
                target: crate::color::ColorSpaceRef::Builtin(crate::color::BuiltinColorSpace::Srgb),
            },
            None,
        )
        .expect("preset");
        let slice = ExportSlice::new(
            "hero",
            SliceSource::Object(ObjectId::new_v4()),
            vec![preset],
        )
        .expect("slice");
        document.slices.insert(slice);
        assert!(document.validate().is_err());
    }
}
