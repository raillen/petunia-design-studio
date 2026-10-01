use petunia_design_application::{Command, CommandRequest, DocumentSession};
use petunia_design_document::{Document, DocumentMutator, DocumentObject, ShapeKind};
use petunia_design_foundation::{ObjectId, SurfaceId};
use petunia_design_geometry::GPoint;

fn rectangle_session() -> (DocumentSession, ObjectId) {
    let mut document = Document::new();
    let id = ObjectId::new(2);
    let mut object = DocumentObject::new(id, "rectangle");
    object.bounds = Some([10.0, 10.0, 20.0, 20.0]);
    object.shape = Some(ShapeKind::Rectangle {
        corner_radii: [0.0; 4],
    });
    let mut mutator = DocumentMutator::new(&mut document);
    mutator.add_surface(SurfaceId::new(1), "page").unwrap();
    mutator.add_object(SurfaceId::new(1), object).unwrap();
    (DocumentSession::with_document("restored", document), id)
}

#[test]
fn restored_document_is_hittable_before_its_first_edit() {
    let (session, id) = rectangle_session();
    assert_eq!(session.current_revision(), 0);
    assert_eq!(
        session.spatial_candidates_point(GPoint::new(20.0, 20.0), 0.1),
        vec![id]
    );
    assert!(session
        .spatial_candidates_point(GPoint::new(100.0, 100.0), 0.1)
        .is_empty());
}

#[test]
fn refreshing_bounds_cannot_validate_old_flattened_geometry() {
    for world in [false, true] {
        let (mut session, id) = rectangle_session();
        for tolerance in [0.1, 0.5] {
            if world {
                session.cached_world_polygons(id, tolerance).unwrap();
            } else {
                session.cached_polygons(id, tolerance).unwrap();
            }
        }
        session
            .execute_command(CommandRequest::new(Command::SetBounds {
                id,
                bounds: Some([100.0, 10.0, 20.0, 20.0]),
                rotation: 0.0,
            }))
            .unwrap();
        assert_eq!(session.cached_bounds(id).unwrap()[0], 100.0);
        assert_eq!(session.cached_world_bounds(id).unwrap()[0], 100.0);
        for tolerance in [0.5, 0.1] {
            let polygons = if world {
                session.cached_world_polygons(id, tolerance)
            } else {
                session.cached_polygons(id, tolerance)
            }
            .unwrap();
            assert!(polygons.iter().flatten().all(|point| point.x >= 100.0));
        }
    }
}

#[test]
fn cached_compound_path_hits_preserve_holes_in_both_frames() {
    use petunia_design_geometry::{GPath, GRect};
    let mut document = Document::new();
    let id = ObjectId::new(2);
    let mut outer = GPath::rect(GRect::new(100.0, 100.0, 200.0, 200.0), 0.0, 0.0);
    let inner = GPath::rect(GRect::new(125.0, 125.0, 175.0, 175.0), 0.0, 0.0);
    for verb in inner.verbs {
        outer.push(verb).unwrap();
    }
    let mut object = DocumentObject::new(id, "Donut");
    object.bounds = Some([100.0, 100.0, 100.0, 100.0]);
    object.shape = Some(ShapeKind::Path(outer));
    let mut m = DocumentMutator::new(&mut document);
    m.add_surface(SurfaceId::new(1), "Page").unwrap();
    m.add_object(SurfaceId::new(1), object).unwrap();
    let session = DocumentSession::with_document("Compound", document);
    for point in [GPoint::new(150.0, 150.0), GPoint::new(110.0, 110.0)] {
        let expected = session
            .find_object(id)
            .unwrap()
            .to_path()
            .contains_point(point, 0.25);
        assert_eq!(session.cached_hit(id, point, 0.25), expected);
        assert_eq!(session.cached_world_hit(id, point, 0.25), expected);
    }
    assert!(!session.cached_world_hit(id, GPoint::new(150.0, 150.0), 0.25));
}
