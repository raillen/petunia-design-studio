//! Document fragments for duplicate, copy and paste.
//!
//! A fragment carries roots plus the nodes, registry records and
//! swatches its dependency closure requires — never the whole
//! document. Paste remaps every owned identity and rewrites internal
//! references with one mapping; undo removes the new entities while
//! redo restores the same new IDs.

use crate::error::{EngineError, Result};
use crate::transaction::DocumentOp;
use crate::transaction::RegistryOp;
use petunia_core::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// How one reference kind travels into a fragment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferencePolicy {
    /// Included and remapped (nodes, clip relations, flow links).
    OwnedDependency,
    /// Reused when the destination already has a compatible
    /// identity, copied otherwise (styles, symbols, resources).
    SharedDependency,
    /// Kept as an external pointer when policy allows.
    ExternalDependency,
    /// Dropped with a warning when semantics allow.
    OptionalReference,
}

/// Internal interchange DTO: roots plus closure, never a
/// mini-document with its own authority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentFragment {
    #[serde(default = "fragment_schema_version")]
    pub schema_version: u32,
    pub roots: Vec<ObjectId>,
    pub nodes: Vec<SceneNode>,
    pub resources: Vec<ResourceRecord>,
    pub styles: Vec<(StyleId, StyleDefinition)>,
    pub symbols: Vec<SymbolDefinition>,
    pub swatches: Vec<Swatch>,
    #[serde(default)]
    pub spots: Vec<SpotColor>,
    #[serde(default)]
    pub grids: Vec<GridDefinition>,
    pub metadata: FragmentMetadata,
}

/// Native clipboard schema is independent of document serialization.
pub const FRAGMENT_SCHEMA_VERSION: u32 = 1;
fn fragment_schema_version() -> u32 {
    FRAGMENT_SCHEMA_VERSION
}

/// Provenance of one fragment.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FragmentMetadata {
    pub source_document: DocumentId,
    pub source_revision: u64,
}

/// One old-to-new identity mapping from a paste or duplicate.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IdRemapping {
    pub objects: HashMap<ObjectId, ObjectId>,
    pub resources: HashMap<ResourceId, ResourceId>,
    pub styles: HashMap<StyleId, StyleId>,
    pub symbols: HashMap<SymbolId, SymbolId>,
    pub swatches: HashMap<SwatchId, SwatchId>,
    pub spots: HashMap<SpotColorId, SpotColorId>,
    pub grids: HashMap<GridId, GridId>,
    pub effects: HashMap<EffectId, EffectId>,
    pub appearance_items: HashMap<AppearanceItemId, AppearanceItemId>,
    pub contours: HashMap<ContourId, ContourId>,
    pub anchors: HashMap<NodeId, NodeId>,
}

impl IdRemapping {
    /// Fresh node identities. Remapping a complete fragment also creates fresh
    /// identities for every registry, effect, contour, anchor and appearance item.
    #[must_use]
    pub fn fresh_for(nodes: &[SceneNode]) -> Self {
        Self {
            objects: nodes
                .iter()
                .map(|node| (node.id, ObjectId::new_v4()))
                .collect(),
            ..Self::default()
        }
    }
    #[must_use]
    pub fn map_object(&self, id: ObjectId) -> ObjectId {
        self.objects.get(&id).copied().unwrap_or(id)
    }
    #[must_use]
    pub fn fresh_for_fragment(fragment: &DocumentFragment) -> Self {
        let mut mapping = Self::fresh_for(&fragment.nodes);
        mapping.complete(fragment);
        mapping
    }
    fn complete(&mut self, fragment: &DocumentFragment) {
        for id in fragment.resources.iter().map(|record| record.id) {
            self.resources.entry(id).or_insert_with(ResourceId::new_v4);
        }
        for id in fragment.styles.iter().map(|(id, _)| *id) {
            self.styles.entry(id).or_insert_with(StyleId::new_v4);
        }
        for id in fragment.symbols.iter().map(|record| record.id) {
            self.symbols.entry(id).or_insert_with(SymbolId::new_v4);
        }
        for id in fragment.swatches.iter().map(|record| record.id) {
            self.swatches.entry(id).or_insert_with(SwatchId::new_v4);
        }
        for record in &fragment.grids {
            self.grids.entry(record.id).or_insert_with(GridId::new_v4);
        }
        for id in fragment.spots.iter().map(|record| record.id) {
            self.spots.entry(id).or_insert_with(SpotColorId::new_v4);
        }
        for symbol in &fragment.symbols {
            for id in symbol.nodes.keys() {
                self.objects.entry(*id).or_insert_with(ObjectId::new_v4);
            }
        }
        for node in fragment.nodes.iter().chain(
            fragment
                .symbols
                .iter()
                .flat_map(|symbol| symbol.nodes.values()),
        ) {
            let mut clone = node.clone();
            visit_node(&mut clone, self);
        }
        for (_, style) in &fragment.styles {
            let mut clone = style.clone();
            let _ = visit_style(&mut clone, self);
        }
    }
}
impl References for IdRemapping {
    fn grid(&mut self, id: &mut GridId) {
        if let Some(mapped) = self.grids.get(id) {
            *id = *mapped;
        }
    }
    fn object(&mut self, id: &mut ObjectId) {
        if let Some(mapped) = self.objects.get(id) {
            *id = *mapped;
        }
    }
    fn resource(&mut self, id: &mut ResourceId) {
        if let Some(mapped) = self.resources.get(id) {
            *id = *mapped;
        }
    }
    fn style(&mut self, id: &mut StyleId) {
        if let Some(mapped) = self.styles.get(id) {
            *id = *mapped;
        }
    }
    fn symbol(&mut self, id: &mut SymbolId) {
        if let Some(mapped) = self.symbols.get(id) {
            *id = *mapped;
        }
    }
    fn swatch(&mut self, id: &mut SwatchId) {
        if let Some(mapped) = self.swatches.get(id) {
            *id = *mapped;
        }
    }
    fn spot(&mut self, id: &mut SpotColorId) {
        if let Some(mapped) = self.spots.get(id) {
            *id = *mapped;
        }
    }
    fn effect(&mut self, id: &mut EffectId) {
        *id = *self.effects.entry(*id).or_insert_with(EffectId::new_v4);
    }
    fn appearance_item(&mut self, id: &mut AppearanceItemId) {
        *id = *self
            .appearance_items
            .entry(*id)
            .or_insert_with(AppearanceItemId::new_v4);
    }
    fn contour(&mut self, id: &mut ContourId) {
        *id = *self.contours.entry(*id).or_insert_with(ContourId::new_v4);
    }
    fn anchor(&mut self, id: &mut NodeId) {
        *id = *self.anchors.entry(*id).or_insert_with(NodeId::new_v4);
    }
}

/// Collect roots plus their transitive scene subtrees. Registry
/// records travel when the caller supplies them; node closure never
/// leaves dangling internal references silently.
pub fn collect_fragment(
    document: &Document,
    roots: &[ObjectId],
    metadata: FragmentMetadata,
) -> Result<DocumentFragment> {
    document.validate()?;
    let mut fragment = DocumentFragment {
        schema_version: FRAGMENT_SCHEMA_VERSION,
        roots: Vec::new(),
        nodes: Vec::new(),
        resources: Vec::new(),
        styles: Vec::new(),
        symbols: Vec::new(),
        swatches: Vec::new(),
        spots: Vec::new(),
        grids: Vec::new(),
        metadata,
    };
    let mut dependencies = Dependencies::default();
    dependencies
        .pending
        .extend(roots.iter().rev().map(|id| Dependency::Object(*id)));
    let mut seen = HashSet::new();
    while let Some(dependency) = dependencies.pending.pop() {
        if !seen.insert(dependency) {
            continue;
        }
        if seen.len() > 100_000 {
            return Err(EngineError::BudgetExceeded(
                "fragment dependency limit exceeded".into(),
            ));
        }
        match dependency {
            Dependency::Object(id) => {
                let mut node = document.scene.get_node(id).cloned().ok_or_else(|| {
                    EngineError::Execution(format!("fragment object {id} missing"))
                })?;
                visit_node(&mut node, &mut dependencies);
                fragment.nodes.push(node);
            }
            Dependency::Resource(id) => {
                fragment
                    .resources
                    .push(document.resources.get(id).cloned().ok_or_else(|| {
                        EngineError::Execution(format!("fragment resource {id} missing"))
                    })?)
            }
            Dependency::Style(id) => {
                let mut style = document.styles.get(id).cloned().ok_or_else(|| {
                    EngineError::Execution(format!("fragment style {id} missing"))
                })?;
                visit_style(&mut style, &mut dependencies)?;
                fragment.styles.push((id, style));
            }
            Dependency::Symbol(id) => {
                let mut symbol = document.symbols.get(id).cloned().ok_or_else(|| {
                    EngineError::Execution(format!("fragment symbol {id} missing"))
                })?;
                dependencies.internal_objects.extend(symbol.nodes.keys());
                for node in symbol.nodes.values_mut() {
                    visit_node(node, &mut dependencies);
                }
                fragment.symbols.push(symbol);
            }
            Dependency::Swatch(id) => {
                let mut swatch = document.swatches.get(id).cloned().ok_or_else(|| {
                    EngineError::Execution(format!("fragment swatch {id} missing"))
                })?;
                match &mut swatch.value {
                    SwatchValue::Color(color) => visit_color(color, &mut dependencies),
                    SwatchValue::Gradient(gradient) => visit_gradient(gradient, &mut dependencies),
                }
                fragment.swatches.push(swatch);
            }
            Dependency::Grid(id) => {
                let mut grid = *document
                    .grids
                    .get(id)
                    .ok_or_else(|| EngineError::Execution("fragment grid missing".into()))?;
                if let GridScope::Artboard(object) = &mut grid.scope {
                    dependencies.object(object);
                }
                fragment.grids.push(grid);
            }
            Dependency::Spot(id) => {
                let mut spot =
                    document.spots.get(id).cloned().ok_or_else(|| {
                        EngineError::Execution(format!("fragment spot {id} missing"))
                    })?;
                visit_space(&mut spot.alternate.space, &mut dependencies);
                fragment.spots.push(spot);
            }
        }
    }
    let included: HashSet<_> = fragment.nodes.iter().map(|node| node.id).collect();
    for node in &mut fragment.nodes {
        if !matches!(node.parent, ParentRef::Object(parent) if included.contains(&parent)) {
            fragment.roots.push(node.id);
            // Detached selections preserve their position in Page space.
            node.transform = document
                .scene
                .world_transform(node.id)
                .ok_or_else(|| EngineError::Execution("fragment transform missing".into()))?;
        }
    }
    Ok(fragment)
}

/// Rewrite every owned reference inside the fragment with `mapping`:
/// node identities, group membership, clip relations and text flow
/// links. Shared style/symbol/resource identities are preserved.
#[must_use]
pub fn remap_fragment(fragment: &DocumentFragment, mapping: &IdRemapping) -> DocumentFragment {
    let mut remapped = fragment.clone();
    let mut mapping = mapping.clone();
    mapping.complete(fragment);
    for node in &mut remapped.nodes {
        remap_node(node, &mut mapping);
    }
    for record in &mut remapped.resources {
        mapping.resource(&mut record.id);
        if let ResourceSource::Embedded { entry } = &mut record.source {
            *entry = format!("{}.bin", record.id);
        }
    }
    for grid in &mut remapped.grids {
        mapping.grid(&mut grid.id);
        if let GridScope::Artboard(id) = &mut grid.scope {
            mapping.object(id);
        }
    }
    for (id, style) in &mut remapped.styles {
        mapping.style(id);
        let _ = visit_style(style, &mut mapping);
    }
    for symbol in &mut remapped.symbols {
        mapping.symbol(&mut symbol.id);
        for root in &mut symbol.roots {
            mapping.object(root);
        }
        let mut nodes = std::collections::BTreeMap::new();
        for (_, mut node) in std::mem::take(&mut symbol.nodes) {
            remap_node(&mut node, &mut mapping);
            nodes.insert(node.id, node);
        }
        symbol.nodes = nodes;
    }
    for swatch in &mut remapped.swatches {
        mapping.swatch(&mut swatch.id);
        match &mut swatch.value {
            SwatchValue::Color(color) => visit_color(color, &mut mapping),
            SwatchValue::Gradient(gradient) => visit_gradient(gradient, &mut mapping),
        }
    }
    for spot in &mut remapped.spots {
        mapping.spot(&mut spot.id);
        visit_space(&mut spot.alternate.space, &mut mapping);
    }
    for root in &mut remapped.roots {
        mapping.object(root);
    }
    remapped
}

fn remap_node(node: &mut SceneNode, mapping: &mut IdRemapping) {
    mapping.object(&mut node.id);
    if let ParentRef::Object(parent) = &mut node.parent {
        mapping.object(parent);
    }
    visit_node(node, mapping);
}

/// Turn a remapped fragment into insert operations under `parent`
/// for one paste transaction. Roots append to the external parent;
/// nested nodes rejoin their (already remapped) fragment parents.
/// Undo removes the new entities; redo restores the same new IDs
/// from the recorded operations.
pub fn fragment_into_ops(
    document: &Document,
    fragment: &DocumentFragment,
    parent: ObjectId,
) -> Result<Vec<DocumentOp>> {
    fragment_into_parent_ops(document, fragment, ParentRef::Object(parent))
}

/// Materialize a native paste into a page or group in one transaction. All
/// registry identities must already be remapped; existing records are never
/// silently overwritten by clipboard content.
pub fn fragment_into_parent_ops(
    document: &Document,
    fragment: &DocumentFragment,
    parent: ParentRef,
) -> Result<Vec<DocumentOp>> {
    validate_fragment(fragment)?;
    for record in &fragment.resources {
        if document.resources.get(record.id).is_some() {
            return Err(EngineError::Execution(
                "paste resource identity collision".into(),
            ));
        }
    }
    for (id, _) in &fragment.styles {
        if document.styles.get(*id).is_some() {
            return Err(EngineError::Execution(
                "paste style identity collision".into(),
            ));
        }
    }
    for record in &fragment.symbols {
        if document.symbols.get(record.id).is_some() {
            return Err(EngineError::Execution(
                "paste symbol identity collision".into(),
            ));
        }
    }
    for record in &fragment.swatches {
        if document.swatches.get(record.id).is_some() {
            return Err(EngineError::Execution(
                "paste swatch identity collision".into(),
            ));
        }
    }
    for record in &fragment.spots {
        if document.spots.get(record.id).is_some() {
            return Err(EngineError::Execution(
                "paste spot identity collision".into(),
            ));
        }
    }
    for grid in &fragment.grids {
        if document.grids.get(grid.id).is_some() {
            return Err(EngineError::Execution(
                "paste grid identity collision".into(),
            ));
        }
    }
    let host_children = match parent {
        ParentRef::Page(page) => document
            .scene
            .page_roots(page)
            .ok_or_else(|| EngineError::Execution("paste page missing".into()))?
            .len(),
        ParentRef::Object(parent) => document
            .scene
            .children_of(parent)
            .ok_or_else(|| EngineError::Execution("paste parent missing or not a group".into()))?
            .len(),
    };
    let mut ops = registry_ops(fragment);
    let parent_inverse = match parent {
        ParentRef::Page(_) => Transform2D::IDENTITY,
        ParentRef::Object(object) => document
            .scene
            .world_transform(object)
            .and_then(|world| world.inverse())
            .ok_or_else(|| {
                EngineError::Execution("paste parent has no safe inverse transform".into())
            })?,
    };
    let page = match parent {
        ParentRef::Page(page) => page,
        ParentRef::Object(object) => std::iter::once(object)
            .chain(document.scene.ancestors(object))
            .find_map(|id| match document.scene.parent_of(id) {
                Some(ParentRef::Page(page)) => Some(page),
                _ => None,
            })
            .ok_or_else(|| EngineError::Execution("paste parent page missing".into()))?,
    };
    for op in &mut ops {
        if let DocumentOp::Registry(RegistryOp::Grid {
            value: Some(grid), ..
        }) = op
        {
            if matches!(grid.scope, GridScope::Page(_)) {
                grid.scope = GridScope::Page(page);
            }
        }
    }
    let mut remaining: Vec<_> = fragment.nodes.iter().collect();
    let mut placed = HashSet::new();
    let mut appended = 0;
    while !remaining.is_empty() {
        let before = remaining.len();
        let mut next = Vec::new();
        for source in remaining {
            let (host, index) = if fragment.roots.contains(&source.id) {
                let index = host_children + appended;
                appended += 1;
                (parent, index)
            } else if let Some(host) = find_fragment_parent(fragment, source.id) {
                if !placed.contains(&host) {
                    next.push(source);
                    continue;
                }
                let index = placement_index(fragment, &placed, host, source.id);
                (ParentRef::Object(host), index)
            } else {
                return Err(EngineError::Execution(format!(
                    "fragment node {} has no owner",
                    source.id
                )));
            };
            let mut node = source.clone();
            if fragment.roots.contains(&source.id) {
                node.transform = parent_inverse.concat(node.transform);
            }
            node.parent = host;
            if let SceneItem::Group(children) = &mut node.item {
                children.clear();
            }
            placed.insert(node.id);
            ops.push(match host {
                ParentRef::Page(_) => DocumentOp::InsertRoot {
                    index,
                    node: Box::new(node),
                },
                ParentRef::Object(parent) => DocumentOp::InsertNode {
                    parent,
                    index,
                    node: Box::new(node),
                },
            });
        }
        if next.len() == before {
            return Err(EngineError::Execution(
                "fragment hierarchy is cyclic or incomplete".into(),
            ));
        }
        remaining = next;
    }
    Ok(ops)
}

/// Reject malformed clipboard structure and oversized dependency packages
/// before attempting to prepare mutations or allocating a shadow document.
pub fn validate_fragment(fragment: &DocumentFragment) -> Result<()> {
    if fragment.schema_version != FRAGMENT_SCHEMA_VERSION {
        return Err(EngineError::Execution("unsupported fragment schema".into()));
    }
    let count = fragment.nodes.len()
        + fragment.resources.len()
        + fragment.styles.len()
        + fragment.symbols.len()
        + fragment.swatches.len()
        + fragment.spots.len();
    if count > 100_000 {
        return Err(EngineError::BudgetExceeded(
            "fragment entity limit exceeded".into(),
        ));
    }
    let nodes: HashSet<_> = fragment.nodes.iter().map(|node| node.id).collect();
    if nodes.len() != fragment.nodes.len()
        || fragment.roots.iter().copied().collect::<HashSet<_>>().len() != fragment.roots.len()
        || fragment.roots.iter().any(|root| !nodes.contains(root))
    {
        return Err(EngineError::Execution(
            "fragment identities or roots are invalid".into(),
        ));
    }
    let mut anchors = 0usize;
    for node in fragment.nodes.iter().chain(
        fragment
            .symbols
            .iter()
            .flat_map(|symbol| symbol.nodes.values()),
    ) {
        if let SceneItem::Path(path) = &node.item {
            anchors = anchors.saturating_add(
                path.path
                    .contours
                    .iter()
                    .map(|contour| contour.nodes.len())
                    .sum::<usize>(),
            );
        }
        if let SceneItem::Text(text) = &node.item {
            if text.text.len() > 16 * 1024 * 1024 {
                return Err(EngineError::BudgetExceeded(
                    "fragment text size exceeded".into(),
                ));
            }
        }
    }
    if anchors > 1_000_000 {
        return Err(EngineError::BudgetExceeded(
            "fragment geometry limit exceeded".into(),
        ));
    }
    let unique = fragment
        .resources
        .iter()
        .map(|record| record.id)
        .collect::<HashSet<_>>()
        .len()
        == fragment.resources.len()
        && fragment
            .styles
            .iter()
            .map(|(id, _)| *id)
            .collect::<HashSet<_>>()
            .len()
            == fragment.styles.len()
        && fragment
            .symbols
            .iter()
            .map(|record| record.id)
            .collect::<HashSet<_>>()
            .len()
            == fragment.symbols.len()
        && fragment
            .swatches
            .iter()
            .map(|record| record.id)
            .collect::<HashSet<_>>()
            .len()
            == fragment.swatches.len()
        && fragment
            .spots
            .iter()
            .map(|record| record.id)
            .collect::<HashSet<_>>()
            .len()
            == fragment.spots.len();
    let unique = unique
        && fragment
            .grids
            .iter()
            .map(|grid| grid.id)
            .collect::<HashSet<_>>()
            .len()
            == fragment.grids.len();
    if !unique {
        return Err(EngineError::Execution(
            "duplicate fragment registry identity".into(),
        ));
    }
    for (_, style) in &fragment.styles {
        let mut style = style.clone();
        visit_style(&mut style, &mut Dependencies::default())?;
    }
    Ok(())
}

fn registry_ops(fragment: &DocumentFragment) -> Vec<DocumentOp> {
    let mut ops = Vec::new();
    for grid in &fragment.grids {
        ops.push(DocumentOp::Registry(RegistryOp::Grid {
            id: grid.id,
            value: Some(Box::new(*grid)),
        }));
    }
    for record in &fragment.resources {
        ops.push(DocumentOp::Registry(RegistryOp::Resource {
            id: record.id,
            value: Some(Box::new(record.clone())),
        }));
    }
    for record in &fragment.spots {
        ops.push(DocumentOp::Registry(RegistryOp::Spot {
            id: record.id,
            value: Some(Box::new(record.clone())),
        }));
    }
    for record in &fragment.swatches {
        ops.push(DocumentOp::Registry(RegistryOp::Swatch {
            id: record.id,
            value: Some(Box::new(record.clone())),
        }));
    }
    for (id, record) in &fragment.styles {
        ops.push(DocumentOp::Registry(RegistryOp::Style {
            id: *id,
            value: Some(Box::new(record.clone())),
        }));
    }
    for record in &fragment.symbols {
        ops.push(DocumentOp::Registry(RegistryOp::Symbol {
            id: record.id,
            value: Some(Box::new(record.clone())),
        }));
    }
    ops
}

fn find_fragment_parent(fragment: &DocumentFragment, child: ObjectId) -> Option<ObjectId> {
    fragment.nodes.iter().find_map(|node| match &node.item {
        SceneItem::Group(children) if children.contains(&child) => Some(node.id),
        _ => None,
    })
}

fn placement_index(
    fragment: &DocumentFragment,
    placed: &HashSet<ObjectId>,
    host: ObjectId,
    child: ObjectId,
) -> usize {
    let Some(host_node) = fragment.nodes.iter().find(|node| node.id == host) else {
        return usize::MAX / 2;
    };
    let SceneItem::Group(children) = &host_node.item else {
        return usize::MAX / 2;
    };
    children
        .iter()
        .take_while(|member| **member != child)
        .filter(|member| placed.contains(member))
        .count()
}

/// Typed reference traversal is shared by dependency collection and remapping.
/// Owning parents are handled separately: copying a child never copies siblings.
trait References {
    fn grid(&mut self, id: &mut GridId);
    fn object(&mut self, id: &mut ObjectId);
    fn resource(&mut self, id: &mut ResourceId);
    fn style(&mut self, id: &mut StyleId);
    fn symbol(&mut self, id: &mut SymbolId);
    fn swatch(&mut self, id: &mut SwatchId);
    fn spot(&mut self, id: &mut SpotColorId);
    fn effect(&mut self, _: &mut EffectId) {}
    fn appearance_item(&mut self, _: &mut AppearanceItemId) {}
    fn contour(&mut self, _: &mut ContourId) {}
    fn anchor(&mut self, _: &mut NodeId) {}
}

fn visit_space(space: &mut ColorSpaceRef, refs: &mut impl References) {
    if let ColorSpaceRef::EmbeddedIcc(id) = space {
        refs.resource(id);
    }
}
fn visit_color(color: &mut ColorValue, refs: &mut impl References) {
    match color {
        ColorValue::Process(process) => visit_space(&mut process.space, refs),
        ColorValue::Spot(spot) => refs.spot(&mut spot.id),
    }
}
fn visit_source(source: &mut ColorSource, refs: &mut impl References) {
    match source {
        ColorSource::Value(color) => visit_color(color, refs),
        ColorSource::Swatch(id) => refs.swatch(id),
    }
}
fn visit_gradient(gradient: &mut Gradient, refs: &mut impl References) {
    for stop in &mut gradient.stops {
        visit_source(&mut stop.color, refs);
    }
}
fn visit_paint(paint: &mut Paint, refs: &mut impl References) {
    match paint {
        Paint::Solid(source) => visit_source(source, refs),
        Paint::LinearGradient(gradient)
        | Paint::RadialGradient(gradient)
        | Paint::ConicalGradient(gradient) => visit_gradient(gradient, refs),
        Paint::Pattern(pattern) => match &mut pattern.source {
            PatternSource::Raster(id) => refs.resource(id),
            PatternSource::Vector(id) => refs.object(id),
        },
    }
}
fn visit_appearance(appearance: &mut Appearance, refs: &mut impl References) {
    for item in &mut appearance.items {
        refs.appearance_item(&mut item.id);
        match &mut item.kind {
            AppearanceKind::Fill(paint) => visit_paint(paint, refs),
            AppearanceKind::Stroke(stroke) => {
                visit_source(&mut stroke.paint, refs);
                if let Some(id) = &mut stroke.start_marker {
                    refs.object(id);
                }
                if let Some(id) = &mut stroke.end_marker {
                    refs.object(id);
                }
            }
        }
    }
}
fn visit_style(style: &mut StyleDefinition, refs: &mut impl References) -> Result<()> {
    match style {
        StyleDefinition::Appearance(style) => visit_appearance(&mut style.appearance, refs),
        StyleDefinition::Character(style) => {
            visit_source(&mut style.color, refs);
            if let Some(id) = &mut style.font.resource {
                refs.resource(id);
            }
        }
        StyleDefinition::Paragraph(style) => {
            if let Some(grid) = &mut style.baseline_grid {
                refs.grid(grid);
            }
        }
    }
    Ok(())
}
fn visit_node(node: &mut SceneNode, refs: &mut impl References) {
    if let Some(clip) = &mut node.clip {
        refs.object(&mut clip.target);
        refs.object(&mut clip.source);
    }
    if let Some(mask) = &mut node.mask {
        refs.object(&mut mask.source);
    }
    for effect in &mut node.geometry_effects.items {
        refs.effect(&mut effect.id);
        if let Some(id) = &mut effect.mask {
            refs.object(id);
        }
    }
    for effect in &mut node.post_effects.items {
        refs.effect(&mut effect.id);
        if let Some(id) = &mut effect.mask {
            refs.object(id);
        }
        match &mut effect.operation {
            PostPaintEffect::Adjustment(id) => refs.object(id),
            PostPaintEffect::DropShadow(shadow) | PostPaintEffect::InnerShadow(shadow) => {
                visit_source(&mut shadow.color, refs)
            }
            PostPaintEffect::Glow(glow) => visit_source(&mut glow.color, refs),
            PostPaintEffect::GaussianBlur(_) => {}
        }
    }
    match &mut node.item {
        SceneItem::Path(path) => {
            for contour in &mut path.path.contours {
                refs.contour(&mut contour.id);
                for anchor in &mut contour.nodes {
                    refs.anchor(&mut anchor.id);
                }
            }
            visit_appearance(&mut path.appearance, refs);
        }
        SceneItem::Shape(shape) => visit_appearance(&mut shape.appearance, refs),
        SceneItem::Group(children) => {
            for id in children {
                refs.object(id);
            }
        }
        SceneItem::Text(text) => {
            for run in &mut text.runs {
                refs.style(&mut run.style.style);
            }
            for run in &mut text.paragraphs {
                refs.style(&mut run.style.style);
            }
            if let Some(id) = &mut text.flow.next {
                refs.object(id);
            }
            if let TextContainer::OnPath(path) = &mut text.container {
                refs.object(&mut path.target);
            }
        }
        SceneItem::Image(image) => refs.resource(&mut image.resource),
        SceneItem::PixelLayer(layer) => refs.resource(&mut layer.surface.resource),
        SceneItem::Trace(trace) => {
            refs.resource(&mut trace.source);
            if let Some(color) = &mut trace.spec.ignore_background {
                visit_color(color, refs);
            }
        }
        SceneItem::GeneratedVector(generated) => visit_appearance(&mut generated.appearance, refs),
        SceneItem::SymbolInstance(instance) => {
            refs.symbol(&mut instance.definition);
            for override_ in &mut instance.overrides {
                refs.object(&mut override_.target);
                match &mut override_.value {
                    SymbolOverrideValue::Paint(source) => visit_source(source, refs),
                    SymbolOverrideValue::Resource(id) => refs.resource(id),
                    SymbolOverrideValue::TextContent(_) | SymbolOverrideValue::Visibility(_) => {}
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Dependency {
    Grid(GridId),
    Object(ObjectId),
    Resource(ResourceId),
    Style(StyleId),
    Symbol(SymbolId),
    Swatch(SwatchId),
    Spot(SpotColorId),
}
#[derive(Default)]
struct Dependencies {
    pending: Vec<Dependency>,
    internal_objects: HashSet<ObjectId>,
}
impl References for Dependencies {
    fn grid(&mut self, id: &mut GridId) {
        self.pending.push(Dependency::Grid(*id));
    }
    fn object(&mut self, id: &mut ObjectId) {
        if !self.internal_objects.contains(id) {
            self.pending.push(Dependency::Object(*id));
        }
    }
    fn resource(&mut self, id: &mut ResourceId) {
        self.pending.push(Dependency::Resource(*id));
    }
    fn style(&mut self, id: &mut StyleId) {
        self.pending.push(Dependency::Style(*id));
    }
    fn symbol(&mut self, id: &mut SymbolId) {
        self.pending.push(Dependency::Symbol(*id));
    }
    fn swatch(&mut self, id: &mut SwatchId) {
        self.pending.push(Dependency::Swatch(*id));
    }
    fn spot(&mut self, id: &mut SpotColorId) {
        self.pending.push(Dependency::Spot(*id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_core::{SceneNode, VectorPath};

    fn document_with_group() -> (Document, ObjectId, ObjectId) {
        let mut document = Document::new("frag");
        let page = document.scene.default_page();
        let mut group = SceneNode::new_path(
            "group",
            VectorPath::new(),
            petunia_core::ParentRef::Page(page),
        );
        group.item = SceneItem::Group(Vec::new());
        let parent = group.id;
        document.scene.insert_node(group);
        let child = SceneNode::new_path(
            "box",
            VectorPath::rect(0.0, 0.0, 5.0, 5.0),
            petunia_core::ParentRef::Object(parent),
        );
        let child_id = child.id;
        document
            .scene
            .insert_child(parent, child, None)
            .expect("child");
        (document, parent, child_id)
    }

    fn metadata() -> FragmentMetadata {
        FragmentMetadata {
            source_document: DocumentId::new_v4(),
            source_revision: 7,
        }
    }

    #[test]
    fn collect_remap_preserves_internal_relations() {
        let (document, parent, child) = document_with_group();
        let fragment = collect_fragment(&document, &[parent], metadata()).expect("collects");
        assert_eq!(fragment.nodes.len(), 2);
        let mapping = IdRemapping::fresh_for(&fragment.nodes);
        let remapped = remap_fragment(&fragment, &mapping);
        // Both identities are new and distinct.
        assert_ne!(remapped.roots[0], parent);
        let group = remapped
            .nodes
            .iter()
            .find(|node| node.id == remapped.roots[0])
            .expect("group");
        let SceneItem::Group(children) = &group.item else {
            panic!("group keeps its shape");
        };
        assert_eq!(children.len(), 1);
        assert_ne!(children[0], child);
        // The child's own mapping matches the group's reference.
        assert_eq!(mapping.map_object(child), children[0]);
    }

    #[test]
    fn paste_ops_rebuild_the_subtree() {
        use crate::transaction::{commit_transaction, prepare_transaction, DocumentRevision};
        let (mut document, parent, _) = document_with_group();
        let fragment = collect_fragment(&document, &[parent], metadata()).expect("collects");
        let mapping = IdRemapping::fresh_for(&fragment.nodes);
        let remapped = remap_fragment(&fragment, &mapping);
        let ops = fragment_into_ops(&document, &remapped, parent).expect("paste ops");
        assert_eq!(ops.len(), 2);
        let prepared = prepare_transaction(
            &document,
            crate::transaction::TransactionRequest {
                command_id: crate::transaction::CommandId::new_v4(),
                operations: ops,
                merge_key: None,
            },
            DocumentRevision::GENESIS,
        )
        .expect("prepares");
        commit_transaction(&mut document, prepared).expect("commits");
        // Original group plus pasted group and pasted child.
        assert_eq!(document.scene.len(), 4);
    }

    #[test]
    fn missing_roots_fail_loudly() {
        let (document, _, _) = document_with_group();
        assert!(collect_fragment(&document, &[ObjectId::new_v4()], metadata()).is_err());
    }
}
