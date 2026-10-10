//! Scene graph tree, scene nodes, and appearance models.

use crate::color::ColorRgba;
use crate::crop::ClipBinding;
use crate::error::{CoreError, Result};
use crate::generated::{GeneratedVectorObject, TraceObject};
use crate::id::{ObjectId, PageId};
use crate::math::Point;
use crate::math::Transform2D;
use crate::path::VectorPath;
use crate::raster::{ImageObject, PixelLayer};
use crate::shape::ParametricShape;
use crate::symbols::SymbolInstance;
use crate::text::TextObject;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

/// Appearance fill definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Fill {
    Solid(ColorRgba),
}

/// Appearance stroke definition.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub color: ColorRgba,
    pub width: f64,
}

/// Structural owner of a node: a page root list or a container object.
///
/// Every node has exactly one explicit owner; there is no implicit
/// root. `Page` ownership lives in the page root lists below, while
/// `Object` ownership lives in the container's children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParentRef {
    Page(PageId),
    Object(ObjectId),
}

/// How a mask source modulates the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaskMode {
    Alpha,
    Luminance,
}

/// A mask relation between scene objects: persistent, never a hidden
/// child positioned by convention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaskBinding {
    pub source: ObjectId,
    pub mode: MaskMode,
}

/// The specific content of a scene object. Leaf items carry their own
/// authorial payload; containers below carry ordered children.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SceneItem {
    Path(VectorPath),
    Group(Vec<ObjectId>),
    Shape(ParametricShape),
    Text(TextObject),
    Image(ImageObject),
    PixelLayer(PixelLayer),
    Trace(TraceObject),
    GeneratedVector(GeneratedVectorObject),
    SymbolInstance(SymbolInstance),
}

impl SceneItem {
    /// Ordered children of a container, if this item is one.
    #[must_use]
    pub fn children(&self) -> Option<&[ObjectId]> {
        match self {
            Self::Group(children) => Some(children),
            _ => None,
        }
    }
}

/// An individual object in the scene graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: ObjectId,
    pub name: String,
    /// Explicit structural owner. Parent and children always change
    /// together through the narrow [`SceneGraph`] operations below;
    /// direct mutation bypasses validation and fails [`SceneGraph::validate`].
    pub parent: ParentRef,
    pub visible: bool,
    pub locked: bool,
    pub transform: Transform2D,
    pub opacity: f32,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub clip: Option<ClipBinding>,
    pub mask: Option<MaskBinding>,
    pub item: SceneItem,
}

impl SceneNode {
    /// Borrow the path of a `SceneItem::Path`, when it is one.
    #[must_use]
    pub fn item_path(&self) -> Option<&VectorPath> {
        match &self.item {
            SceneItem::Path(path) => Some(path),
            _ => None,
        }
    }

    /// First anchor of a path item, for headless hit-test helpers.
    #[must_use]
    pub fn item_path_point(&self) -> Option<Point> {
        self.item_path()?
            .contours
            .first()
            .and_then(|contour| contour.nodes.first())
            .map(|node| node.point)
    }

    #[must_use]
    pub fn new_path(name: impl Into<String>, path: VectorPath, parent: ParentRef) -> Self {
        Self {
            id: ObjectId::new_v4(),
            name: name.into(),
            parent,
            visible: true,
            locked: false,
            transform: Transform2D::IDENTITY,
            opacity: 1.0,
            fill: Some(Fill::Solid(ColorRgba::BLACK)),
            stroke: None,
            clip: None,
            mask: None,
            item: SceneItem::Path(path),
        }
    }
}

/// Hierarchical scene graph containing all objects and their ordering.
///
/// Storage is a map by [`ObjectId`]; z-order always comes from the
/// ordered page root lists and container children, never from storage
/// iteration. `BTreeMap` keeps even the serialized form deterministic.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SceneGraph {
    #[serde(deserialize_with = "crate::serialization::deserialize_unique_btree_map")]
    nodes: BTreeMap<ObjectId, SceneNode>,
    #[serde(deserialize_with = "crate::serialization::deserialize_unique_btree_map")]
    pages: BTreeMap<PageId, Vec<ObjectId>>,
    default_page: PageId,
}

impl SceneGraph {
    #[must_use]
    pub fn new() -> Self {
        let default_page = PageId::new_v4();
        let mut pages = BTreeMap::new();
        pages.insert(default_page, Vec::new());
        Self {
            nodes: BTreeMap::new(),
            pages,
            default_page,
        }
    }

    /// Page used by the compatibility helpers below. The future
    /// Document aggregate owns page metadata; this graph only stores
    /// the structural root lists.
    #[must_use]
    pub fn default_page(&self) -> PageId {
        self.default_page
    }

    /// Create an empty page root list.
    pub fn create_page(&mut self) -> PageId {
        let page = PageId::new_v4();
        self.pages.insert(page, Vec::new());
        page
    }

    /// Ordered roots of a page: its z-order, back to front.
    #[must_use]
    pub fn page_roots(&self, page: PageId) -> Option<&[ObjectId]> {
        self.pages.get(&page).map(Vec::as_slice)
    }

    /// Every page identity in deterministic order.
    #[must_use]
    pub fn page_ids(&self) -> Vec<PageId> {
        self.pages.keys().copied().collect()
    }

    /// Every page root list, for whole-graph traversals such as
    /// document validation. Order across pages is page-id order and
    /// carries no z meaning.
    #[must_use]
    pub fn root_lists(&self) -> Vec<&[ObjectId]> {
        self.pages.values().map(Vec::as_slice).collect()
    }

    /// Insert a root node whose parent is `page`. The declared parent
    /// must match the destination; ownership is never fixed up silently.
    pub fn insert_root(&mut self, page: PageId, node: SceneNode) -> Result<()> {
        if !self.pages.contains_key(&page) {
            return Err(CoreError::UnknownPage(page.to_string()));
        }
        if node.parent != ParentRef::Page(page) {
            return Err(CoreError::InvalidParent(format!(
                "node {} declares {parent:?}, not page {page}",
                node.id,
                parent = node.parent,
            )));
        }
        self.insert_validated(page, node, None)
    }

    /// Insert a child node into a group container. The parent must
    /// exist and be a group; the declared parent must match it.
    pub fn insert_child(
        &mut self,
        parent: ObjectId,
        node: SceneNode,
        index: Option<usize>,
    ) -> Result<()> {
        let Some(host) = self.nodes.get(&parent) else {
            return Err(CoreError::ObjectNotFound(parent.to_string()));
        };
        if !matches!(host.item, SceneItem::Group(_)) {
            return Err(CoreError::NotAContainer(parent.to_string()));
        }
        if node.parent != ParentRef::Object(parent) {
            return Err(CoreError::InvalidParent(format!(
                "node {} declares {declared:?}, not object {parent}",
                node.id,
                declared = node.parent,
            )));
        }
        if self.nodes.contains_key(&node.id) {
            return Err(CoreError::DuplicateObject(node.id.to_string()));
        }
        self.validate_node(&node)?;
        let id = node.id;
        self.nodes.insert(id, node);
        let Some(SceneItem::Group(children)) =
            self.nodes.get_mut(&parent).map(|host| &mut host.item)
        else {
            unreachable!("container checked above");
        };
        match index {
            Some(at) => children.insert(at.min(children.len()), id),
            None => children.push(id),
        }
        Ok(())
    }

    /// Shared tail of the narrow insertions once ownership is settled.
    fn insert_validated(
        &mut self,
        page: PageId,
        node: SceneNode,
        index: Option<usize>,
    ) -> Result<()> {
        if self.nodes.contains_key(&node.id) {
            return Err(CoreError::DuplicateObject(node.id.to_string()));
        }
        self.validate_node(&node)?;
        let id = node.id;
        self.nodes.insert(id, node);
        let roots = self
            .pages
            .get_mut(&page)
            .ok_or_else(|| CoreError::UnknownPage(page.to_string()))?;
        match index {
            Some(at) => roots.insert(at.min(roots.len()), id),
            None => roots.push(id),
        }
        Ok(())
    }

    /// Remove a node and its whole subtree. Fails when another node
    /// still references the subtree as a clip or mask source, so no
    /// dangling mandatory reference survives. Returns the removed
    /// nodes parent-first, ready to be re-inserted for undo.
    pub fn remove_subtree(&mut self, id: ObjectId) -> Result<Vec<SceneNode>> {
        if !self.nodes.contains_key(&id) {
            return Err(CoreError::ObjectNotFound(id.to_string()));
        }
        let removed: Vec<ObjectId> = std::iter::once(id).chain(self.descendants(id)).collect();
        let gone: HashSet<ObjectId> = removed.iter().copied().collect();
        for node in self.nodes.values() {
            if gone.contains(&node.id) {
                continue;
            }
            for source in binding_sources(node) {
                if gone.contains(&source) {
                    return Err(CoreError::DanglingReference(format!(
                        "node {} is still a clip/mask source of {}",
                        source, node.id,
                    )));
                }
            }
        }
        self.detach(id);
        let mut out = Vec::with_capacity(removed.len());
        for victim in removed {
            if let Some(node) = self.nodes.remove(&victim) {
                out.push(node);
            }
        }
        Ok(out)
    }

    /// Move a node keeping its local geometry: only the owner changes.
    pub fn reparent_keeping_local(&mut self, id: ObjectId, parent: ParentRef) -> Result<()> {
        self.reparent(id, parent, None)
    }

    /// Move a node keeping its world geometry: the local transform is
    /// recomputed as `inverse(world(new_parent)) × old_world`. Fails
    /// with a typed error when the new parent is not invertible.
    pub fn reparent_keeping_world(&mut self, id: ObjectId, parent: ParentRef) -> Result<()> {
        let old_world = self
            .world_transform(id)
            .ok_or_else(|| CoreError::ObjectNotFound(format!("world of missing node {id}")))?;
        let parent_world = match parent {
            ParentRef::Page(_) => Transform2D::IDENTITY,
            ParentRef::Object(host) => self.world_transform(host).ok_or_else(|| {
                CoreError::ObjectNotFound(format!("world of missing parent {host}"))
            })?,
        };
        let Some(inverse) = parent_world.inverse() else {
            return Err(CoreError::NonInvertibleTransform(format!(
                "cannot keep world under {parent:?}"
            )));
        };
        let local = inverse.concat(old_world);
        self.reparent(id, parent, Some(local))
    }

    fn reparent(
        &mut self,
        id: ObjectId,
        parent: ParentRef,
        local: Option<Transform2D>,
    ) -> Result<()> {
        if !self.nodes.contains_key(&id) {
            return Err(CoreError::ObjectNotFound(id.to_string()));
        }
        match parent {
            ParentRef::Page(page) => {
                if !self.pages.contains_key(&page) {
                    return Err(CoreError::UnknownPage(page.to_string()));
                }
            }
            ParentRef::Object(host) => {
                if host == id {
                    return Err(CoreError::CycleDetected(format!(
                        "{id} cannot parent itself"
                    )));
                }
                let Some(host_node) = self.nodes.get(&host) else {
                    return Err(CoreError::ObjectNotFound(host.to_string()));
                };
                if !matches!(host_node.item, SceneItem::Group(_)) {
                    return Err(CoreError::NotAContainer(host.to_string()));
                }
                if self.is_descendant_of(host, id) {
                    return Err(CoreError::CycleDetected(format!("{host} is inside {id}")));
                }
            }
        }
        // A clip/mask source keeps pointing at this node wherever it
        // moves, but the move must not create a reference cycle.
        if self.binding_cycle_after_move(id, parent) {
            return Err(CoreError::CycleDetected(format!(
                "moving {id} under {parent:?} closes a reference cycle"
            )));
        }
        self.detach(id);
        match parent {
            ParentRef::Page(page) => {
                self.pages
                    .get_mut(&page)
                    .ok_or_else(|| CoreError::UnknownPage(page.to_string()))?
                    .push(id);
            }
            ParentRef::Object(host) => {
                let Some(SceneItem::Group(children)) =
                    self.nodes.get_mut(&host).map(|host| &mut host.item)
                else {
                    unreachable!("container checked above");
                };
                children.push(id);
            }
        }
        let Some(node) = self.nodes.get_mut(&id) else {
            unreachable!("existence checked above");
        };
        node.parent = parent;
        if let Some(local) = local {
            if !transform_is_finite(&local) {
                return Err(CoreError::InvariantViolation(format!(
                    "reparent produced a non-finite local transform for {id}"
                )));
            }
            node.transform = local;
        }
        Ok(())
    }

    /// Reorder a child inside its container or page root list. The
    /// index clamps into range; z-order stays an explicit list.
    pub fn reorder_child(
        &mut self,
        parent: ParentRef,
        child: ObjectId,
        index: usize,
    ) -> Result<()> {
        let list = match parent {
            ParentRef::Page(page) => self
                .pages
                .get_mut(&page)
                .ok_or_else(|| CoreError::UnknownPage(page.to_string()))?,
            ParentRef::Object(host) => {
                let Some(host_node) = self.nodes.get_mut(&host) else {
                    return Err(CoreError::ObjectNotFound(host.to_string()));
                };
                match &mut host_node.item {
                    SceneItem::Group(children) => children,
                    _ => return Err(CoreError::NotAContainer(host.to_string())),
                }
            }
        };
        let Some(current) = list.iter().position(|item| *item == child) else {
            return Err(CoreError::ObjectNotFound(format!(
                "{child} is not a child of {parent:?}"
            )));
        };
        list.remove(current);
        list.insert(index.min(list.len()), child);
        Ok(())
    }

    /// Detach a node from its current owner without deleting it.
    fn detach(&mut self, id: ObjectId) {
        let Some(parent) = self.nodes.get(&id).map(|node| node.parent) else {
            return;
        };
        match parent {
            ParentRef::Page(page) => {
                if let Some(roots) = self.pages.get_mut(&page) {
                    roots.retain(|item| *item != id);
                }
            }
            ParentRef::Object(host) => {
                if let Some(host_node) = self.nodes.get_mut(&host) {
                    if let SceneItem::Group(children) = &mut host_node.item {
                        children.retain(|item| *item != id);
                    }
                }
            }
        }
    }

    /// Retrieves a reference to a node by ID.
    pub fn get_node(&self, id: ObjectId) -> Option<&SceneNode> {
        self.nodes.get(&id)
    }

    /// Retrieves a mutable reference to a node by ID.
    ///
    /// Non-structural edits (transform, flags, appearance) go through
    /// here. Structural ownership still belongs to the narrow
    /// operations; run [`SceneGraph::validate`] to catch bypasses.
    pub fn get_node_mut(&mut self, id: ObjectId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(&id)
    }

    /// Declared structural owner of a node.
    pub fn parent_of(&self, id: ObjectId) -> Option<ParentRef> {
        self.nodes.get(&id).map(|node| node.parent)
    }

    /// Ordered children of a group container. `None` when the node is
    /// missing or not a container; leaf items expose no children.
    pub fn children_of(&self, id: ObjectId) -> Option<&[ObjectId]> {
        self.nodes.get(&id)?.item.children()
    }

    /// Ancestors from the direct parent up to the page, cycle-guarded.
    pub fn ancestors(&self, id: ObjectId) -> Vec<ObjectId> {
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        let mut cursor = self.parent_of(id).and_then(|parent| match parent {
            ParentRef::Page(_) => None,
            ParentRef::Object(host) => Some(host),
        });
        while let Some(current) = cursor {
            if !seen.insert(current) {
                break;
            }
            out.push(current);
            cursor = self.parent_of(current).and_then(|parent| match parent {
                ParentRef::Page(_) => None,
                ParentRef::Object(host) => Some(host),
            });
        }
        out
    }

    /// Descendants depth-first in z-order, excluding the node itself.
    pub fn descendants(&self, id: ObjectId) -> Vec<ObjectId> {
        let mut out = Vec::new();
        let mut stack: Vec<ObjectId> = self.children_of(id).unwrap_or(&[]).to_vec();
        let mut seen = HashSet::new();
        while let Some(current) = stack.pop() {
            if !seen.insert(current) {
                continue;
            }
            out.push(current);
            if let Some(children) = self.children_of(current) {
                stack.extend(children.iter().copied());
            }
        }
        // Pop order is reverse z-order; restore document order.
        out.reverse();
        out
    }

    /// True when `id` sits inside `ancestor`'s subtree.
    pub fn is_descendant_of(&self, id: ObjectId, ancestor: ObjectId) -> bool {
        self.ancestors(id).contains(&ancestor)
    }

    /// Derived world transform: the product of every local transform
    /// from the page down to the node. `None` for a missing node or a
    /// broken ancestry chain. World transforms are derived state and
    /// are never persisted.
    pub fn world_transform(&self, id: ObjectId) -> Option<Transform2D> {
        let mut chain = vec![id];
        chain.extend(self.ancestors(id));
        let mut world = Transform2D::IDENTITY;
        for member in chain.into_iter().rev() {
            world = world.concat(self.nodes.get(&member)?.transform);
        }
        Some(world)
    }

    /// Whether moving `id` under `parent` would close a reference
    /// cycle through clip/mask bindings.
    fn binding_cycle_after_move(&self, id: ObjectId, parent: ParentRef) -> bool {
        let ParentRef::Object(host) = parent else {
            return false;
        };
        // Moving under a host that (transitively) depends on this node
        // through bindings would make the two reach each other.
        let mut stack = vec![host];
        let mut seen = HashSet::new();
        while let Some(current) = stack.pop() {
            if current == id {
                return true;
            }
            if !seen.insert(current) {
                continue;
            }
            let Some(node) = self.nodes.get(&current) else {
                continue;
            };
            stack.extend(binding_sources(node));
            // Hierarchy edges also carry reachability here: a host
            // inside the moved subtree already fails above, but its
            // own bindings still count.
            if let Some(children) = node.item.children() {
                stack.extend(children.iter().copied());
            }
        }
        false
    }

    /// Full structural validation: ownership on both sides, acyclic
    /// hierarchy and bindings, finite geometry and live references.
    pub fn validate(&self) -> Result<()> {
        if !self.pages.contains_key(&self.default_page) {
            return Err(CoreError::UnknownPage(self.default_page.to_string()));
        }
        for (page, roots) in &self.pages {
            let mut seen = HashSet::new();
            for root in roots {
                if !seen.insert(root) {
                    return Err(CoreError::InvariantViolation(format!(
                        "duplicate root {root} on page {page}"
                    )));
                }
                let Some(node) = self.nodes.get(root) else {
                    return Err(CoreError::DanglingReference(format!(
                        "page {page} lists missing node {root}"
                    )));
                };
                if node.parent != ParentRef::Page(*page) {
                    return Err(CoreError::InvariantViolation(format!(
                        "root {root} declares {parent:?}, not page {page}",
                        parent = node.parent,
                    )));
                }
            }
        }
        for node in self.nodes.values() {
            self.validate_node(node)?;
            match node.parent {
                ParentRef::Page(page) => {
                    let Some(roots) = self.pages.get(&page) else {
                        return Err(CoreError::UnknownPage(page.to_string()));
                    };
                    if !roots.contains(&node.id) {
                        return Err(CoreError::InvariantViolation(format!(
                            "node {} declares page {page} but is not listed there",
                            node.id,
                        )));
                    }
                }
                ParentRef::Object(host) => {
                    let Some(host_node) = self.nodes.get(&host) else {
                        return Err(CoreError::DanglingReference(format!(
                            "node {} parents missing object {host}",
                            node.id,
                        )));
                    };
                    let Some(children) = host_node.item.children() else {
                        return Err(CoreError::NotAContainer(host.to_string()));
                    };
                    if !children.contains(&node.id) {
                        return Err(CoreError::InvariantViolation(format!(
                            "node {} declares object parent {host} without being listed",
                            node.id,
                        )));
                    }
                }
            }
            if let Some(children) = node.item.children() {
                let mut seen = HashSet::new();
                for child in children {
                    if !seen.insert(child) {
                        return Err(CoreError::InvariantViolation(format!(
                            "duplicate child {child} in {}",
                            node.id,
                        )));
                    }
                    let Some(child_node) = self.nodes.get(child) else {
                        return Err(CoreError::DanglingReference(format!(
                            "{} lists missing child {child}",
                            node.id,
                        )));
                    };
                    if child_node.parent != ParentRef::Object(node.id) {
                        return Err(CoreError::InvariantViolation(format!(
                            "child {child} does not point back at {}",
                            node.id,
                        )));
                    }
                }
            }
            for source in binding_sources(node) {
                if !self.nodes.contains_key(&source) {
                    return Err(CoreError::DanglingReference(format!(
                        "node {} binds missing source {source}",
                        node.id,
                    )));
                }
                if source == node.id {
                    return Err(CoreError::CycleDetected(format!(
                        "node {} binds itself",
                        node.id
                    )));
                }
            }
        }
        self.check_acyclic()?;
        Ok(())
    }

    /// Per-node checks that do not need the whole graph: finite
    /// transform and opacity, plus a clip binding that names its host.
    fn validate_node(&self, node: &SceneNode) -> Result<()> {
        if !transform_is_finite(&node.transform) {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite transform on {}",
                node.id,
            )));
        }
        if !node.opacity.is_finite() {
            return Err(CoreError::InvariantViolation(format!(
                "non-finite opacity on {}",
                node.id,
            )));
        }
        if let Some(clip) = &node.clip {
            if clip.target != node.id {
                return Err(CoreError::InvariantViolation(format!(
                    "clip on {} names target {}",
                    node.id, clip.target,
                )));
            }
        }
        Ok(())
    }

    /// Reject any cycle across parent edges and clip/mask bindings.
    fn check_acyclic(&self) -> Result<()> {
        // 0 = unvisited, 1 = on the current path, 2 = done.
        let mut state: BTreeMap<ObjectId, u8> = BTreeMap::new();
        for id in self.nodes.keys() {
            if state.get(id).copied().unwrap_or(0) != 0 {
                continue;
            }
            let mut stack = vec![(*id, false)];
            while let Some((current, expanded)) = stack.pop() {
                if expanded {
                    state.insert(current, 2);
                    continue;
                }
                match state.get(&current).copied().unwrap_or(0) {
                    1 => {
                        return Err(CoreError::CycleDetected(format!(
                            "reference cycle through {current}"
                        )));
                    }
                    2 => continue,
                    _ => {}
                }
                state.insert(current, 1);
                stack.push((current, true));
                let Some(node) = self.nodes.get(&current) else {
                    continue;
                };
                if let ParentRef::Object(host) = node.parent {
                    stack.push((host, false));
                }
                for source in binding_sources(node) {
                    stack.push((source, false));
                }
            }
        }
        Ok(())
    }

    /// Inserts a node into the default page root order.
    ///
    /// Compatibility shim: new code uses [`SceneGraph::insert_root`],
    /// which verifies the declared parent instead of assigning it.
    /// Duplicate IDs replace the stored node, as before.
    pub fn insert_node(&mut self, mut node: SceneNode) {
        let id = node.id;
        node.parent = ParentRef::Page(self.default_page);
        self.nodes.insert(id, node);
        // One owner only: drop stale listings anywhere else first.
        for roots in self.pages.values_mut() {
            roots.retain(|item| *item != id);
        }
        for stored in self.nodes.values_mut() {
            if let SceneItem::Group(children) = &mut stored.item {
                children.retain(|item| *item != id);
            }
        }
        if let Some(roots) = self.pages.get_mut(&self.default_page) {
            if !roots.contains(&id) {
                roots.push(id);
            }
        }
    }

    /// Removes a node from the scene graph.
    ///
    /// Compatibility shim with the historical behavior: only the page
    /// root listing and the node itself are removed, so references
    /// held elsewhere can dangle. New code uses
    /// [`SceneGraph::remove_subtree`], which refuses to leave dangling
    /// mandatory references behind.
    pub fn remove_node(&mut self, id: ObjectId) -> Result<SceneNode> {
        if let Some(roots) = self.pages.get_mut(&self.default_page) {
            roots.retain(|&item_id| item_id != id);
        }
        self.nodes
            .remove(&id)
            .ok_or_else(|| CoreError::ObjectNotFound(id.to_string()))
    }

    /// Detaches an ID from the default page root order without removing
    /// its node. Compatibility helper for the default page; page-aware
    /// code edits root lists through narrow operations instead.
    pub fn unlist_root(&mut self, id: ObjectId) {
        if let Some(roots) = self.pages.get_mut(&self.default_page) {
            roots.retain(|&item_id| item_id != id);
        }
    }

    /// Move an ID already present in the default page root order to a
    /// new index, clamped into range. Root order is z-order; storage
    /// never decides it.
    pub fn place_root(&mut self, id: ObjectId, index: usize) {
        let Some(roots) = self.pages.get_mut(&self.default_page) else {
            return;
        };
        let Some(current) = roots.iter().position(|item| *item == id) else {
            return;
        };
        let removed = roots.remove(current);
        let at = index.min(roots.len());
        roots.insert(at, removed);
    }

    /// Returns the default page root ordering of objects.
    #[must_use]
    pub fn root_order(&self) -> &[ObjectId] {
        self.pages
            .get(&self.default_page)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Returns total number of nodes in the scene.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

/// Clip and mask sources of a node, for validation and cycle checks.
fn binding_sources(node: &SceneNode) -> Vec<ObjectId> {
    let mut sources = Vec::new();
    if let Some(clip) = &node.clip {
        sources.push(clip.source);
    }
    if let Some(mask) = &node.mask {
        sources.push(mask.source);
    }
    sources
}

/// True when every transform coefficient is finite.
fn transform_is_finite(transform: &Transform2D) -> bool {
    let Transform2D { a, b, c, d, tx, ty } = *transform;
    [a, b, c, d, tx, ty].iter().all(|v| v.is_finite())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crop::BindingSourceUse;

    fn page_of(scene: &SceneGraph) -> PageId {
        scene.default_page()
    }

    fn boxed(scene: &SceneGraph, name: &str) -> SceneNode {
        SceneNode::new_path(
            name,
            VectorPath::rect(0.0, 0.0, 10.0, 10.0),
            ParentRef::Page(page_of(scene)),
        )
    }

    #[test]
    fn test_scene_graph_insert_and_retrieval() {
        let mut scene = SceneGraph::new();
        let node = boxed(&scene, "Test Box");
        let id = node.id;

        scene.insert_node(node);
        assert_eq!(scene.len(), 1);
        assert!(scene.get_node(id).is_some());
        assert_eq!(scene.root_order(), &[id]);
        assert_eq!(scene.parent_of(id), Some(ParentRef::Page(page_of(&scene))));

        let removed = scene.remove_node(id).expect("removal succeeds");
        assert_eq!(removed.id, id);
        assert!(scene.is_empty());
    }

    #[test]
    fn insert_root_verifies_the_declared_parent() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        let mut node = boxed(&scene, "declared");
        node.parent = ParentRef::Page(PageId::new_v4());
        assert!(scene.insert_root(page, node).is_err());
    }

    #[test]
    fn insert_child_links_both_sides() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        let group = SceneNode {
            id: ObjectId::new_v4(),
            name: "group".to_string(),
            parent: ParentRef::Page(page),
            visible: true,
            locked: false,
            transform: Transform2D::IDENTITY,
            opacity: 1.0,
            fill: None,
            stroke: None,
            clip: None,
            mask: None,
            item: SceneItem::Group(Vec::new()),
        };
        let group_id = group.id;
        scene.insert_root(page, group).expect("insert group");
        let child = SceneNode::new_path(
            "child",
            VectorPath::rect(0.0, 0.0, 5.0, 5.0),
            ParentRef::Object(group_id),
        );
        let child_id = child.id;
        scene
            .insert_child(group_id, child, None)
            .expect("insert child");
        assert_eq!(scene.parent_of(child_id), Some(ParentRef::Object(group_id)));
        assert_eq!(scene.children_of(group_id), Some([child_id].as_slice()));
        assert_eq!(scene.ancestors(child_id), vec![group_id]);
        assert!(scene.is_descendant_of(child_id, group_id));
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn non_groups_reject_children() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        let path = boxed(&scene, "leaf");
        let path_id = path.id;
        scene.insert_root(page, path).expect("insert leaf");
        let child = SceneNode::new_path(
            "child",
            VectorPath::rect(0.0, 0.0, 1.0, 1.0),
            ParentRef::Object(path_id),
        );
        assert!(scene.insert_child(path_id, child, None).is_err());
    }

    #[test]
    fn reparent_keeping_local_moves_ownership_only() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        scene
            .insert_root(page, boxed(&scene, "group-a"))
            .expect("a");
        let group_a = scene.root_order()[0];
        // Turn the first root into a group container.
        scene.get_node_mut(group_a).expect("group").item = SceneItem::Group(Vec::new());
        let leaf = boxed(&scene, "leaf");
        let leaf_id = leaf.id;
        scene.insert_root(page, leaf).expect("leaf");
        scene
            .reparent_keeping_local(leaf_id, ParentRef::Object(group_a))
            .expect("reparent");
        assert_eq!(scene.parent_of(leaf_id), Some(ParentRef::Object(group_a)));
        assert_eq!(scene.page_roots(page), Some([group_a].as_slice()));
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn reparent_refuses_cycles() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        let group = SceneNode {
            id: ObjectId::new_v4(),
            name: "group".to_string(),
            parent: ParentRef::Page(page),
            visible: true,
            locked: false,
            transform: Transform2D::IDENTITY,
            opacity: 1.0,
            fill: None,
            stroke: None,
            clip: None,
            mask: None,
            item: SceneItem::Group(Vec::new()),
        };
        let group_id = group.id;
        scene.insert_root(page, group).expect("group");
        let child = SceneNode::new_path(
            "child",
            VectorPath::rect(0.0, 0.0, 1.0, 1.0),
            ParentRef::Object(group_id),
        );
        let child_id = child.id;
        scene.insert_child(group_id, child, None).expect("child");
        // A group cannot move under its own child.
        assert!(scene
            .reparent_keeping_local(group_id, ParentRef::Object(child_id))
            .is_err());
        // A node cannot parent itself.
        assert!(scene
            .reparent_keeping_local(group_id, ParentRef::Object(group_id))
            .is_err());
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn reparent_keeping_world_recomputes_the_local_transform() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        let mut holder = boxed(&scene, "holder");
        holder.transform = Transform2D::translation(10.0, 0.0);
        let holder_id = holder.id;
        scene.insert_root(page, holder).expect("holder");
        let leaf = boxed(&scene, "leaf");
        let leaf_id = leaf.id;
        scene.insert_root(page, leaf).expect("leaf");
        let before = scene.world_transform(leaf_id).expect("world");
        scene.get_node_mut(holder_id).expect("holder").item = SceneItem::Group(Vec::new());
        scene
            .reparent_keeping_world(leaf_id, ParentRef::Object(holder_id))
            .expect("reparent");
        let after = scene.world_transform(leaf_id).expect("world");
        assert_eq!(after, before);
        let local = scene.get_node(leaf_id).expect("leaf").transform;
        assert_eq!(local.tx, -10.0);
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn world_transform_composes_down_the_hierarchy() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        let mut group = boxed(&scene, "group");
        group.transform = Transform2D::translation(5.0, 0.0);
        let group_id = group.id;
        scene.insert_root(page, group).expect("group");
        scene.get_node_mut(group_id).expect("group").item = SceneItem::Group(Vec::new());
        let mut child = SceneNode::new_path(
            "child",
            VectorPath::rect(0.0, 0.0, 1.0, 1.0),
            ParentRef::Object(group_id),
        );
        child.transform = Transform2D::translation(0.0, 7.0);
        let child_id = child.id;
        scene.insert_child(group_id, child, None).expect("child");
        let world = scene.world_transform(child_id).expect("world");
        assert_eq!(
            world.transform_point(Point::new(0.0, 0.0)),
            Point::new(5.0, 7.0)
        );
    }

    #[test]
    fn remove_subtree_detaches_and_returns_parent_first() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        let group = SceneNode {
            id: ObjectId::new_v4(),
            name: "group".to_string(),
            parent: ParentRef::Page(page),
            visible: true,
            locked: false,
            transform: Transform2D::IDENTITY,
            opacity: 1.0,
            fill: None,
            stroke: None,
            clip: None,
            mask: None,
            item: SceneItem::Group(Vec::new()),
        };
        let group_id = group.id;
        scene.insert_root(page, group).expect("group");
        let child = SceneNode::new_path(
            "child",
            VectorPath::rect(0.0, 0.0, 1.0, 1.0),
            ParentRef::Object(group_id),
        );
        let child_id = child.id;
        scene.insert_child(group_id, child, None).expect("child");
        let removed = scene.remove_subtree(group_id).expect("remove");
        assert_eq!(
            removed.iter().map(|node| node.id).collect::<Vec<_>>(),
            vec![group_id, child_id]
        );
        assert!(scene.is_empty());
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn remove_subtree_refuses_live_bindings() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        let source = boxed(&scene, "source");
        let source_id = source.id;
        scene.insert_root(page, source).expect("source");
        let mut target = boxed(&scene, "target");
        target.clip = Some(
            crate::crop::ClipBinding::new(target.id, source_id, BindingSourceUse::BindingOnly)
                .expect("valid"),
        );
        let target_id = target.id;
        scene.insert_root(page, target).expect("target");
        assert!(scene.remove_subtree(source_id).is_err());
        // Removing the dependent first releases the source.
        scene.remove_subtree(target_id).expect("target goes first");
        assert_eq!(scene.remove_subtree(source_id).expect("source").len(), 1);
    }

    #[test]
    fn mutual_bindings_are_rejected_as_cycles() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        let first = boxed(&scene, "first");
        let first_id = first.id;
        scene.insert_root(page, first).expect("first");
        let mut second = boxed(&scene, "second");
        let second_id = second.id;
        second.mask = Some(MaskBinding {
            source: first_id,
            mode: MaskMode::Alpha,
        });
        scene.insert_root(page, second).expect("second");
        scene.get_node_mut(first_id).expect("first").mask = Some(MaskBinding {
            source: second_id,
            mode: MaskMode::Alpha,
        });
        assert!(scene.validate().is_err());
    }

    #[test]
    fn reorder_child_keeps_z_order_explicit() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        let first = boxed(&scene, "first");
        let first_id = first.id;
        let second = boxed(&scene, "second");
        let second_id = second.id;
        scene.insert_root(page, first).expect("first");
        scene.insert_root(page, second).expect("second");
        scene
            .reorder_child(ParentRef::Page(page), first_id, 5)
            .expect("reorder");
        assert_eq!(
            scene.page_roots(page),
            Some([second_id, first_id].as_slice())
        );
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn pages_keep_independent_root_lists() {
        let mut scene = SceneGraph::new();
        let first_page = scene.default_page();
        let second_page = scene.create_page();
        let first = boxed(&scene, "first");
        let first_id = first.id;
        scene.insert_root(first_page, first).expect("first");
        let mut second = boxed(&scene, "second");
        second.parent = ParentRef::Page(second_page);
        let second_id = second.id;
        scene.insert_root(second_page, second).expect("second");
        assert_eq!(scene.page_roots(first_page), Some([first_id].as_slice()));
        assert_eq!(scene.page_roots(second_page), Some([second_id].as_slice()));
        assert!(scene.validate().is_ok());
    }

    #[test]
    fn structural_serialization_round_trip() {
        let mut scene = SceneGraph::new();
        let page = scene.default_page();
        let node = boxed(&scene, "box");
        let id = node.id;
        scene.insert_root(page, node).expect("insert");
        let json = serde_json::to_string(&scene).expect("serializable");
        let back: SceneGraph = serde_json::from_str(&json).expect("deserializable");
        assert_eq!(back.len(), 1);
        assert_eq!(back.parent_of(id), Some(ParentRef::Page(page)));
        assert_eq!(back.default_page(), page);
        assert!(back.validate().is_ok());
    }
}
