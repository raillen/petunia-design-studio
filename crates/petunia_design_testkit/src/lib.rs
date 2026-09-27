#![forbid(unsafe_code)]

//! Deterministic headless fixtures for the canvas and interaction gauntlets.

use petunia_design_application::create_shape_commands;
use petunia_design_application::view_camera::ViewportCamera;
use petunia_design_application::Command;
use petunia_design_document::{ContainerRole, ShapeKind};
use petunia_design_foundation::{ObjectId, PetuniaError, SurfaceId};
use petunia_design_geometry::GPoint;
use petunia_design_shell::shell::PetuniaShell;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixtureOptions {
    pub object_count: usize,
    pub group_size: usize,
    pub nested_groups: bool,
    pub rotation_period: usize,
    pub viewport_width: f64,
    pub viewport_height: f64,
}

impl Default for FixtureOptions {
    fn default() -> Self {
        Self {
            object_count: 500,
            group_size: 0,
            nested_groups: false,
            rotation_period: 17,
            viewport_width: 4096.0,
            viewport_height: 4096.0,
        }
    }
}

#[derive(Debug)]
pub struct SceneFixture {
    pub shell: PetuniaShell,
    pub surface_id: SurfaceId,
    pub root_ids: Vec<ObjectId>,
    pub all_ids: Vec<ObjectId>,
    anchors: Vec<(ObjectId, GPoint)>,
    digest: u64,
}

impl SceneFixture {
    pub fn first_object_id(&self) -> Option<ObjectId> {
        self.root_ids.first().copied()
    }

    pub fn anchor(&self, id: ObjectId) -> Option<GPoint> {
        self.anchors
            .iter()
            .find(|(candidate, _)| *candidate == id)
            .map(|(_, point)| *point)
    }

    pub fn digest_hex(&self) -> String {
        format!("{:016x}", self.digest)
    }
}

pub fn build_scene(options: FixtureOptions) -> Result<SceneFixture, PetuniaError> {
    if options.object_count == 0 {
        return Err(PetuniaError::invalid_input(
            "canvas fixture must contain at least one object",
        ));
    }
    if !options.viewport_width.is_finite() || !options.viewport_height.is_finite() {
        return Err(PetuniaError::invalid_input(
            "canvas fixture viewport must be finite",
        ));
    }

    let mut shell = PetuniaShell::new(options.viewport_width, options.viewport_height);
    shell.new_document("P07-G05 Deterministic Canvas Fixture")?;
    let surface_id = shell
        .bridge
        .active_surface()
        .ok_or_else(|| PetuniaError::invalid_input("canvas fixture has no active surface"))?;
    shell.set_view_camera(ViewportCamera::new(
        options.viewport_width,
        options.viewport_height,
    ));

    let mut root_ids = Vec::with_capacity(options.object_count);
    let mut all_ids = Vec::with_capacity(options.object_count);
    let mut commands = Vec::with_capacity(options.object_count.saturating_mul(5));
    for index in 0..options.object_count {
        let id = ObjectId::new(index as u64 + 1);
        let column = (index % 50) as f64;
        let row = (index / 50) as f64;
        let bounds = [32.0 + column * 32.0, 24.0 + row * 24.0, 24.0, 18.0];
        let rotation =
            if options.rotation_period > 0 && index > 0 && index % options.rotation_period == 0 {
                ((index / options.rotation_period) % 8) as f64 * std::f64::consts::FRAC_PI_4
            } else {
                0.0
            };
        commands.extend(create_shape_commands(
            surface_id,
            id,
            format!("Fixture Object {index}"),
            ShapeKind::Rectangle {
                corner_radii: [0.0; 4],
            },
            Some(bounds),
            Some("ptnd.blue/500".to_string()),
            None,
        ));
        if rotation != 0.0 {
            commands.push(Command::SetBounds {
                id,
                bounds: Some(bounds),
                rotation,
            });
        }
        root_ids.push(id);
        all_ids.push(id);
    }
    shell
        .bridge
        .submit_all("Build deterministic canvas fixture", commands)?;

    let mut next_id = options.object_count as u64 + 1;
    if options.group_size > 1 {
        let level_ids = group_level(
            &mut shell,
            surface_id,
            root_ids.clone(),
            options.group_size,
            &mut next_id,
            &mut all_ids,
        )?;
        if options.nested_groups {
            group_level(
                &mut shell,
                surface_id,
                level_ids,
                options.group_size,
                &mut next_id,
                &mut all_ids,
            )?;
        }
    }

    let anchors = root_ids
        .iter()
        .map(|id| {
            let session = shell
                .bridge
                .session()
                .ok_or_else(|| PetuniaError::invalid_input("fixture session disappeared"))?;
            let object = session.document().find_object(*id).ok_or_else(|| {
                PetuniaError::not_found(format!("fixture object `{id}` disappeared"))
            })?;
            let [_x, _y, width, height] = object.bounds.ok_or_else(|| {
                PetuniaError::invalid_input(format!("fixture object `{id}` has no bounds"))
            })?;
            let transform = session
                .document()
                .world_transform_checked(*id)
                .map_err(|error| PetuniaError::invalid_input(error.to_string()))?;
            Ok((*id, transform.apply(GPoint::new(width / 2.0, height / 2.0))))
        })
        .collect::<Result<Vec<_>, PetuniaError>>()?;

    let digest = fixture_digest(options, &root_ids, &anchors);
    Ok(SceneFixture {
        shell,
        surface_id,
        root_ids,
        all_ids,
        anchors,
        digest,
    })
}

fn group_level(
    shell: &mut PetuniaShell,
    surface_id: SurfaceId,
    ids: Vec<ObjectId>,
    group_size: usize,
    next_id: &mut u64,
    all_ids: &mut Vec<ObjectId>,
) -> Result<Vec<ObjectId>, PetuniaError> {
    let mut next_level = Vec::with_capacity(ids.len());
    for chunk in ids.chunks(group_size.max(1)) {
        if chunk.len() < 2 {
            next_level.push(chunk[0]);
            continue;
        }
        let group_id = ObjectId::new(*next_id);
        *next_id = next_id.saturating_add(1);
        shell
            .bridge
            .group_objects(surface_id, group_id, chunk.to_vec(), ContainerRole::Group)?;
        all_ids.push(group_id);
        next_level.push(group_id);
    }
    Ok(next_level)
}

fn fixture_digest(
    options: FixtureOptions,
    root_ids: &[ObjectId],
    anchors: &[(ObjectId, GPoint)],
) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    let mut mix = |value: u64| {
        digest ^= value;
        digest = digest.wrapping_mul(0x1000_0000_01b3);
    };
    mix(options.object_count as u64);
    mix(options.group_size as u64);
    mix(options.nested_groups as u64);
    mix(options.rotation_period as u64);
    for id in root_ids {
        mix(id.raw());
    }
    for (id, point) in anchors {
        mix(id.raw());
        mix(point.x.to_bits());
        mix(point.y.to_bits());
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::*;
    use petunia_design_application::interaction::{
        NormalizedPointerEvent, PointerButton, PointerPhase, SemanticModifiers,
    };

    #[test]
    fn fixture_drives_select_transform_preview() {
        let mut fixture = build_scene(FixtureOptions {
            object_count: 4,
            rotation_period: 2,
            ..FixtureOptions::default()
        })
        .expect("fixture");
        let id = fixture.first_object_id().expect("object");
        let anchor = fixture.anchor(id).expect("anchor");
        let camera = fixture.shell.view_camera();
        let down = NormalizedPointerEvent::new(
            PointerPhase::Down,
            PointerButton::Primary,
            camera.doc_to_screen(anchor),
            anchor,
            SemanticModifiers::default(),
        );
        fixture
            .shell
            .handle_pointer_event(&down)
            .expect("select down");
        let target = petunia_design_geometry::GPoint::new(anchor.x + 8.0, anchor.y);
        let move_event = NormalizedPointerEvent::new(
            PointerPhase::Move,
            PointerButton::Primary,
            camera.doc_to_screen(target),
            target,
            SemanticModifiers {
                disable_snap: true,
                ..SemanticModifiers::default()
            },
        );
        fixture
            .shell
            .handle_pointer_event(&move_event)
            .expect("select move");
        assert!(fixture
            .shell
            .tools
            .select_tool()
            .transform_preview()
            .is_some());
        let cancel = NormalizedPointerEvent::new(
            PointerPhase::Cancel,
            PointerButton::Primary,
            camera.doc_to_screen(target),
            target,
            SemanticModifiers::default(),
        );
        fixture
            .shell
            .handle_pointer_event(&cancel)
            .expect("select cancel");
    }

    #[test]
    fn fixture_is_deterministic_for_equal_options() {
        let options = FixtureOptions {
            object_count: 8,
            rotation_period: 2,
            ..FixtureOptions::default()
        };
        let first = build_scene(options).expect("first fixture");
        let second = build_scene(options).expect("second fixture");
        assert_eq!(first.digest_hex(), second.digest_hex());
        assert_eq!(first.root_ids, second.root_ids);
        assert_eq!(first.anchors, second.anchors);
    }

    #[test]
    fn nested_fixture_preserves_root_world_anchors() {
        let options = FixtureOptions {
            object_count: 8,
            group_size: 4,
            nested_groups: true,
            rotation_period: 2,
            ..FixtureOptions::default()
        };
        let fixture = build_scene(options).expect("fixture");
        let root = fixture.first_object_id().expect("root object");
        let session = fixture.shell.bridge.session().expect("session");
        let object = session.document().find_object(root).expect("root");
        assert!(object.parent.is_some());
        assert!(fixture.anchor(root).is_some());
    }
}
