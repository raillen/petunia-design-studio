//! Desktop edit drafts bind to a session and object, never to a selection index.
//! Only the edited property is compared with its opening value before commit.
use petunia_design_application::{session::SessionIdentity, Command};
use petunia_design_document::ShapeKind;
use petunia_design_foundation::{
    numeric::{parse_numeric_input, NumericFieldKind},
    ObjectId, PetuniaError,
};
use petunia_design_shell::PetuniaShell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKind {
    Name,
    Text,
    Transform,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EditValue {
    Name(String),
    Text(ShapeKind),
    Transform { bounds: [f64; 4], rotation: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ObjectEdit {
    pub session: SessionIdentity,
    pub object: ObjectId,
    pub original: EditValue,
}

#[derive(Debug)]
pub enum EditFailure {
    TargetChanged,
    PropertyChanged,
    InvalidName,
    Number {
        field: &'static str,
        reason: petunia_design_foundation::numeric::NumericParseError,
    },
    Command(PetuniaError),
}

impl ObjectEdit {
    pub fn capture(
        shell: &PetuniaShell,
        object: ObjectId,
        kind: EditKind,
    ) -> Result<Self, EditFailure> {
        let session = shell.bridge.session().ok_or(EditFailure::TargetChanged)?;
        let obj = session
            .find_object(object)
            .ok_or(EditFailure::TargetChanged)?;
        let original = match kind {
            EditKind::Name => EditValue::Name(obj.name.clone()),
            EditKind::Text => match &obj.shape {
                Some(shape @ ShapeKind::Text { .. }) => EditValue::Text(shape.clone()),
                _ => return Err(EditFailure::TargetChanged),
            },
            EditKind::Transform => EditValue::Transform {
                bounds: obj.bounds.ok_or(EditFailure::TargetChanged)?,
                rotation: obj.rotation,
            },
        };
        Ok(Self {
            session: session.identity(),
            object,
            original,
        })
    }

    pub fn apply(
        &self,
        shell: &mut PetuniaShell,
        text: &str,
        fields: [&str; 5],
    ) -> Result<bool, EditFailure> {
        let session = shell.bridge.session().ok_or(EditFailure::TargetChanged)?;
        if session.identity() != self.session {
            return Err(EditFailure::TargetChanged);
        }
        let kind = match self.original {
            EditValue::Name(_) => EditKind::Name,
            EditValue::Text(_) => EditKind::Text,
            EditValue::Transform { .. } => EditKind::Transform,
        };
        if Self::capture(shell, self.object, kind)?.original != self.original {
            return Err(EditFailure::PropertyChanged);
        }
        let command = match &self.original {
            EditValue::Name(original) => {
                let name = text.trim();
                if name.is_empty()
                    || name.chars().count() > 256
                    || name.chars().any(char::is_control)
                {
                    return Err(EditFailure::InvalidName);
                }
                if name == original {
                    return Ok(false);
                }
                Command::RenameObject {
                    id: self.object,
                    name: name.to_owned(),
                }
            }
            EditValue::Text(original) => {
                let mut shape = original.clone();
                let ShapeKind::Text { content, .. } = &mut shape else {
                    unreachable!()
                };
                if content == text {
                    return Ok(false);
                }
                *content = text.to_owned();
                Command::SetShape {
                    id: self.object,
                    shape: Some(shape),
                }
            }
            EditValue::Transform {
                bounds: original,
                rotation: old_rotation,
            } => {
                let mut bounds = [0.; 4];
                for (index, tag) in ["X", "Y", "W", "H"].into_iter().enumerate() {
                    bounds[index] = field_number(
                        fields[index],
                        tag,
                        if index < 2 {
                            NumericFieldKind::DistancePt
                        } else {
                            NumericFieldKind::PositiveDimensionPt
                        },
                        &["pt"],
                    )?;
                }
                let rotation =
                    field_number(fields[4], "°", NumericFieldKind::AngleDeg, &["deg", "°"])?
                        .to_radians();
                // Untouched fields retain the original radians, avoiding a
                // round-trip conversion that would create a spurious undo step.
                let rotation = if fields[4] == old_rotation.to_degrees().to_string() {
                    *old_rotation
                } else {
                    rotation
                };
                if &bounds == original && rotation == *old_rotation {
                    return Ok(false);
                }
                Command::SetBounds {
                    id: self.object,
                    bounds: Some(bounds),
                    rotation,
                }
            }
        };
        shell
            .bridge
            .submit_all("Edit object property", vec![command])
            .map_err(EditFailure::Command)?;
        Ok(true)
    }
}

/// This UI declares points and degrees. Other suffixes require a real unit
/// conversion and must not be stripped by the more general numeric parser.
fn field_number(
    raw: &str,
    field: &'static str,
    kind: NumericFieldKind,
    suffixes: &[&str],
) -> Result<f64, EditFailure> {
    let raw = raw.trim();
    let lower = raw.to_ascii_lowercase();
    let numeric = suffixes
        .iter()
        .find(|suffix| lower.ends_with(**suffix))
        .map_or(raw, |suffix| raw[..raw.len() - suffix.len()].trim());
    if !numeric.is_empty() && numeric.replace(',', ".").parse::<f64>().is_err() {
        return Err(EditFailure::Number {
            field,
            reason: petunia_design_foundation::numeric::NumericParseError::NotANumber,
        });
    }
    parse_numeric_input(numeric, kind).map_err(|reason| EditFailure::Number { field, reason })
}

/// A size step changes only size; font family, spacing, content and path stay.
pub fn step_font_size(
    shell: &mut PetuniaShell,
    target: &ObjectEdit,
    delta: f64,
) -> Result<(), EditFailure> {
    if shell.bridge.session().map(|s| s.identity()) != Some(target.session) {
        return Err(EditFailure::TargetChanged);
    }
    let current = ObjectEdit::capture(shell, target.object, EditKind::Text)?;
    let EditValue::Text(mut shape) = current.original else {
        return Err(EditFailure::TargetChanged);
    };
    let ShapeKind::Text { font_size, .. } = &mut shape else {
        unreachable!()
    };
    let next = (*font_size + delta).max(1.);
    if next == *font_size {
        return Ok(());
    }
    *font_size = next;
    shell
        .bridge
        .submit_all(
            "Set font size",
            vec![Command::SetShape {
                id: target.object,
                shape: Some(shape),
            }],
        )
        .map_err(EditFailure::Command)?;
    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use petunia_design_document::TextOnPathAttachment;

    pub fn fixture() -> (PetuniaShell, ObjectId, ObjectId) {
        let mut shell = PetuniaShell::new(800., 600.);
        shell.new_document("Editor").unwrap();
        let surface = shell.bridge.active_surface().unwrap();
        let path = shell.bridge.next_object_id().unwrap();
        let text = shell.bridge.next_object_id().unwrap();
        shell
            .bridge
            .submit_all(
                "Fixture",
                vec![
                    Command::CreateShapeObject {
                        surface,
                        id: path,
                        name: "Path".into(),
                        shape: ShapeKind::Path(petunia_design_geometry::GPath::rect(
                            petunia_design_geometry::GRect::new(10., 20., 210., 100.),
                            0.,
                            0.,
                        )),
                        bounds: Some([10., 20., 200., 80.]),
                        fill: None,
                        stroke: None,
                        stroke_width: 0.,
                    },
                    Command::CreateShapeObject {
                        surface,
                        id: text,
                        name: "Title".into(),
                        shape: ShapeKind::Text {
                            content: "First\nSecond".into(),
                            font_family: "Noto Sans".into(),
                            font_size: 22.,
                            line_height: 1.7,
                            letter_spacing: 2.5,
                            on_path: Some(TextOnPathAttachment::new(path, 0.1, 0.9)),
                        },
                        bounds: Some([30., 40., 180., 50.]),
                        fill: Some("ptnd.gray/900".into()),
                        stroke: None,
                        stroke_width: 0.,
                    },
                    Command::SetBounds {
                        id: text,
                        bounds: Some([30., 40., 180., 50.]),
                        rotation: 0.73,
                    },
                ],
            )
            .unwrap();
        shell.bridge.set_selection(vec![text]);
        (shell, text, path)
    }

    #[test]
    fn clearing_text_preserves_typography_path_and_undo_redo() {
        let (mut shell, text, path) = fixture();
        let draft = ObjectEdit::capture(&shell, text, EditKind::Text).unwrap();
        assert!(draft.apply(&mut shell, "", [""; 5]).unwrap());
        let obj = shell.bridge.session().unwrap().find_object(text).unwrap();
        let Some(ShapeKind::Text {
            content,
            font_family,
            font_size,
            line_height,
            letter_spacing,
            on_path,
        }) = &obj.shape
        else {
            panic!()
        };
        assert_eq!(content, "");
        assert_eq!(font_family, "Noto Sans");
        assert_eq!((*font_size, *line_height, *letter_spacing), (22., 1.7, 2.5));
        assert_eq!(on_path.unwrap().target, path);
        assert_eq!(obj.rotation, 0.73);
        shell.bridge.undo().unwrap();
        assert_eq!(
            ObjectEdit::capture(&shell, text, EditKind::Text)
                .unwrap()
                .original,
            draft.original
        );
        shell.bridge.redo().unwrap();
        assert!(
            matches!(shell.bridge.session().unwrap().find_object(text).unwrap().shape.as_ref(), Some(ShapeKind::Text { content, .. }) if content.is_empty())
        );
    }

    #[test]
    fn size_step_preserves_every_other_text_attribute() {
        let (mut shell, text, _) = fixture();
        let draft = ObjectEdit::capture(&shell, text, EditKind::Text).unwrap();
        step_font_size(&mut shell, &draft, 4.).unwrap();
        let EditValue::Text(mut expected) = draft.original else {
            panic!()
        };
        let ShapeKind::Text { font_size, .. } = &mut expected else {
            panic!()
        };
        *font_size = 26.;
        assert_eq!(
            shell
                .bridge
                .session()
                .unwrap()
                .find_object(text)
                .unwrap()
                .shape,
            Some(expected)
        );
    }

    #[test]
    fn transform_is_atomic_validates_units_and_keeps_invalid_drafts_out_of_history() {
        let (mut shell, text, _) = fixture();
        let draft = ObjectEdit::capture(&shell, text, EditKind::Transform).unwrap();
        let revision = shell.bridge.session().unwrap().current_revision();
        for fields in [
            ["4", "5", "0", "20", "90"],
            ["4", "5", "NaN", "20", "90"],
            ["4", "5", "100 px", "20", "90"],
            ["4", "5", "100", "20", "90 pt"],
        ] {
            assert!(matches!(
                draft.apply(&mut shell, "", fields),
                Err(EditFailure::Number { .. })
            ));
            assert_eq!(shell.bridge.session().unwrap().current_revision(), revision);
        }
        assert!(draft
            .apply(&mut shell, "", ["-4,5 pt", "5", "100 pt", "20", "90°"])
            .unwrap());
        let obj = shell.bridge.session().unwrap().find_object(text).unwrap();
        assert_eq!(obj.bounds, Some([-4.5, 5., 100., 20.]));
        assert_eq!(obj.rotation, std::f64::consts::FRAC_PI_2);
        shell.bridge.undo().unwrap();
        assert_eq!(
            ObjectEdit::capture(&shell, text, EditKind::Transform).unwrap(),
            draft
        );
        shell.bridge.redo().unwrap();
        assert_eq!(
            shell
                .bridge
                .session()
                .unwrap()
                .find_object(text)
                .unwrap()
                .bounds,
            Some([-4.5, 5., 100., 20.])
        );
    }

    #[test]
    fn editing_only_width_preserves_exact_rotation() {
        let (mut shell, text, _) = fixture();
        let draft = ObjectEdit::capture(&shell, text, EditKind::Transform).unwrap();
        let rotation = 0.73_f64.to_degrees().to_string();
        assert!(draft
            .apply(&mut shell, "", ["30", "40", "181", "50", &rotation])
            .unwrap());
        assert_eq!(
            shell
                .bridge
                .session()
                .unwrap()
                .find_object(text)
                .unwrap()
                .rotation,
            0.73
        );
    }

    #[test]
    fn draft_cannot_write_another_session_with_the_same_object_id() {
        let (mut shell, text, _) = fixture();
        let draft = ObjectEdit::capture(&shell, text, EditKind::Name).unwrap();
        shell.new_document("Other").unwrap();
        let surface = shell.bridge.active_surface().unwrap();
        shell
            .bridge
            .submit_all(
                "Other object",
                vec![Command::CreateObject {
                    surface,
                    id: text,
                    name: "Keep".into(),
                }],
            )
            .unwrap();
        assert!(matches!(
            draft.apply(&mut shell, "Wrong", [""; 5]),
            Err(EditFailure::TargetChanged)
        ));
        assert_eq!(
            shell
                .bridge
                .session()
                .unwrap()
                .find_object(text)
                .unwrap()
                .name,
            "Keep"
        );
    }

    #[test]
    fn conflicting_property_is_rejected_but_unrelated_edit_is_preserved() {
        let (mut shell, text, _) = fixture();
        let draft = ObjectEdit::capture(&shell, text, EditKind::Name).unwrap();
        shell
            .bridge
            .submit_all(
                "Opacity",
                vec![Command::SetOpacity {
                    id: text,
                    opacity: 0.4,
                }],
            )
            .unwrap();
        assert!(draft.apply(&mut shell, "Edited", [""; 5]).unwrap());
        assert_eq!(
            shell
                .bridge
                .session()
                .unwrap()
                .find_object(text)
                .unwrap()
                .opacity,
            0.4
        );
        assert!(matches!(
            draft.apply(&mut shell, "Overwrite", [""; 5]),
            Err(EditFailure::PropertyChanged)
        ));
        assert_eq!(
            shell
                .bridge
                .session()
                .unwrap()
                .find_object(text)
                .unwrap()
                .name,
            "Edited"
        );
    }

    #[test]
    fn unchanged_properties_do_not_create_history_and_names_are_validated() {
        let (mut shell, text, _) = fixture();
        let name = ObjectEdit::capture(&shell, text, EditKind::Name).unwrap();
        let revision = shell.bridge.session().unwrap().current_revision();
        assert!(!name.apply(&mut shell, " Title ", [""; 5]).unwrap());
        for invalid in ["  ".to_owned(), "Title\nother".to_owned(), "x".repeat(257)] {
            assert!(matches!(
                name.apply(&mut shell, &invalid, [""; 5]),
                Err(EditFailure::InvalidName)
            ));
        }
        let transform = ObjectEdit::capture(&shell, text, EditKind::Transform).unwrap();
        let angle = 0.73_f64.to_degrees().to_string();
        assert!(!transform
            .apply(&mut shell, "", ["30", "40", "180", "50", &angle])
            .unwrap());
        assert_eq!(shell.bridge.session().unwrap().current_revision(), revision);
    }

    #[test]
    fn text_drafts_are_owned_by_their_object_not_shared_with_other_content() {
        let (mut shell, text, _) = fixture();
        let second_id = shell.bridge.next_object_id().unwrap();
        let surface = shell.bridge.active_surface().unwrap();
        shell
            .bridge
            .submit_all(
                "Second text",
                vec![Command::CreateShapeObject {
                    surface,
                    id: second_id,
                    name: "Second".into(),
                    shape: ShapeKind::Text {
                        content: "Other".into(),
                        font_family: "Inter".into(),
                        font_size: 14.,
                        line_height: 1.2,
                        letter_spacing: 0.,
                        on_path: None,
                    },
                    bounds: Some([1., 1., 100., 20.]),
                    fill: None,
                    stroke: None,
                    stroke_width: 0.,
                }],
            )
            .unwrap();
        let first = ObjectEdit::capture(&shell, text, EditKind::Text).unwrap();
        let second = ObjectEdit::capture(&shell, second_id, EditKind::Text).unwrap();
        assert!(second.apply(&mut shell, "Second draft", [""; 5]).unwrap());
        assert_eq!(
            ObjectEdit::capture(&shell, text, EditKind::Text).unwrap(),
            first
        );
    }
}
