//! Scene graph tree, scene nodes, and appearance models.

use crate::color::ColorRgba;
use crate::crop::ClipBinding;
use crate::error::{CoreError, Result};
use crate::generated::{GeneratedVectorObject, TraceObject};
use crate::id::ObjectId;
use crate::math::Transform2D;
use crate::path::VectorPath;
use crate::raster::{ImageObject, PixelLayer};
use crate::shape::ParametricShape;
use crate::symbols::SymbolInstance;
use crate::text::TextObject;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Appearance fill definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Fill {
    Solid(ColorRgba),
}

/// Appearance stroke definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub color: ColorRgba,
    pub width: f64,
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

/// An individual object in the scene graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: ObjectId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub transform: Transform2D,
    pub opacity: f32,
    pub fill: Option<Fill>,
    pub stroke: Option<Stroke>,
    pub clip: Option<ClipBinding>,
    pub item: SceneItem,
}

impl SceneNode {
    #[must_use]
    pub fn new_path(name: impl Into<String>, path: VectorPath) -> Self {
        Self {
            id: ObjectId::new_v4(),
            name: name.into(),
            visible: true,
            locked: false,
            transform: Transform2D::IDENTITY,
            opacity: 1.0,
            fill: Some(Fill::Solid(ColorRgba::BLACK)),
            stroke: None,
            clip: None,
            item: SceneItem::Path(path),
        }
    }
}

/// Hierarchical scene graph containing all objects and their ordering.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SceneGraph {
    nodes: HashMap<ObjectId, SceneNode>,
    root_order: Vec<ObjectId>,
}

impl SceneGraph {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_order: Vec::new(),
        }
    }

    /// Inserts a node into the root order.
    pub fn insert_node(&mut self, node: SceneNode) {
        let id = node.id;
        self.nodes.insert(id, node);
        if !self.root_order.contains(&id) {
            self.root_order.push(id);
        }
    }

    /// Retrieves a reference to a node by ID.
    pub fn get_node(&self, id: ObjectId) -> Option<&SceneNode> {
        self.nodes.get(&id)
    }

    /// Retrieves a mutable reference to a node by ID.
    pub fn get_node_mut(&mut self, id: ObjectId) -> Option<&mut SceneNode> {
        self.nodes.get_mut(&id)
    }

    /// Removes a node from the scene graph.
    pub fn remove_node(&mut self, id: ObjectId) -> Result<SceneNode> {
        self.root_order.retain(|&item_id| item_id != id);
        self.nodes
            .remove(&id)
            .ok_or_else(|| CoreError::ObjectNotFound(id.to_string()))
    }

    /// Detaches an ID from the root order without removing its node.
    /// Used when a node becomes a child of a group: it stays stored,
    /// but only roots define z-order.
    pub fn unlist_root(&mut self, id: ObjectId) {
        self.root_order.retain(|&item_id| item_id != id);
    }

    /// Returns the root ordering of objects.
    #[must_use]
    pub fn root_order(&self) -> &[ObjectId] {
        &self.root_order
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_graph_insert_and_retrieval() {
        let mut scene = SceneGraph::new();
        let path = VectorPath::rect(0.0, 0.0, 50.0, 50.0);
        let node = SceneNode::new_path("Test Box", path);
        let id = node.id;

        scene.insert_node(node);
        assert_eq!(scene.len(), 1);
        assert!(scene.get_node(id).is_some());
        assert_eq!(scene.root_order(), &[id]);

        let removed = scene.remove_node(id).expect("removal succeeds");
        assert_eq!(removed.id, id);
        assert!(scene.is_empty());
    }
}
