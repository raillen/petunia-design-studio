//! Final-phase regressions for ADR-012. These mount the production Rust shell.
use crate::*;
use freya_testing::prelude::*;
use petunia_design_application::Command;
use petunia_design_document::ShapeKind;
use petunia_design_foundation::ObjectId;
use petunia_design_shell::Locale;
use std::{cell::RefCell, rc::Rc, time::Duration};

fn mount(width: f32, height: f32, locale: Locale) -> (TestingRunner, UiShell) {
    let observed = Rc::new(RefCell::new(None));
    let output = observed.clone();
    let (mut runner, ()) = TestingRunner::new(
        move || {
            use_init_theme(theme::petunia_theme);
            let shell = use_state(|| {
                let mut shell = PetuniaShell::new(1280., 800.);
                shell.bridge.set_locale(locale.clone());
                file_workflows::create_configured_document(
                    &mut shell,
                    "Petunia · Studio",
                    [800., 560.],
                    0.,
                    0.,
                )
                .unwrap();
                let surface = shell.bridge.active_surface().unwrap();
                for (index, (name, bounds, color, shape)) in [
                    (
                        "Background",
                        [0., 0., 800., 560.],
                        "#EADFCB",
                        ShapeKind::Rectangle {
                            corner_radii: [0.; 4],
                        },
                    ),
                    (
                        "Coral rectangle",
                        [120., 110., 230., 300.],
                        "#F06C8D",
                        ShapeKind::Rectangle {
                            corner_radii: [18.; 4],
                        },
                    ),
                    (
                        "Blue ellipse",
                        [300., 80., 330., 330.],
                        "#354BA6",
                        ShapeKind::Ellipse,
                    ),
                    (
                        "Title",
                        [85., 435., 600., 65.],
                        "#21232B",
                        ShapeKind::Text {
                            content: "PETUNIA / DESIGN".into(),
                            font_family: "DejaVu Sans".into(),
                            font_size: 38.,
                            line_height: 1.2,
                            letter_spacing: 2.,
                            on_path: None,
                        },
                    ),
                ]
                .into_iter()
                .enumerate()
                {
                    let id = shell.bridge.next_object_id().unwrap();
                    shell
                        .bridge
                        .submit_all(
                            "Fixture",
                            vec![Command::CreateShapeObject {
                                surface,
                                id,
                                name: name.into(),
                                shape,
                                bounds: Some(bounds),
                                fill: Some(color.into()),
                                stroke: Some("#21232B".into()),
                                stroke_width: if index == 0 { 0. } else { 3. },
                            }],
                        )
                        .unwrap();
                }
                shell.bridge.clear_selection();
                shell
            });
            let ui = UiShell::fresh(shell);
            ui.autofit_documents.clone().set(true);
            output.replace(Some(ui.clone()));
            desktop_studio(ui)
        },
        (width, height).into(),
        |_| {},
        1.,
    );
    runner.sync_and_update();
    let ui = observed.borrow().clone().unwrap();
    (runner, ui)
}
fn click_label(runner: &mut TestingRunner, title: &str) {
    let area = runner
        .find(|node, el| {
            Label::try_downcast(el)
                .filter(|label| label.text == title)
                .map(|_| node)
        })
        .unwrap_or_else(|| panic!("Missing label {title}"))
        .layout()
        .area;
    runner.click_cursor((f64::from(area.center().x), f64::from(area.center().y)));
    runner.sync_and_update();
}
fn click_control(runner: &mut TestingRunner, title: &str) {
    let area = runner
        .find(|node, el| {
            Rect::try_downcast(el)
                .filter(|r| r.accessibility.builder.label() == Some(title))
                .map(|_| node)
        })
        .unwrap_or_else(|| panic!("Missing control {title}"))
        .layout()
        .area;
    runner.click_cursor((f64::from(area.center().x), f64::from(area.center().y)));
    runner.sync_and_update();
}
fn replace_input(runner: &mut TestingRunner, index: usize, text: &str) {
    let mut nodes = runner.find_many(|node, el| {
        Rect::try_downcast(el)
            .filter(|r| {
                matches!(
                    r.accessibility.builder.role(),
                    AccessibilityRole::TextInput | AccessibilityRole::MultilineTextInput
                )
            })
            .map(|_| node)
    });
    nodes.sort_by(|a, b| a.layout().area.min_y().total_cmp(&b.layout().area.min_y()));
    let area = nodes[index].layout().area;
    let point = (f64::from(area.min_x() + 20.), f64::from(area.min_y() + 14.));
    runner.move_cursor(point);
    // Native focus requests are committed between down and up. The headless
    // runner needs an explicit turn before the previous input's outside click.
    runner.press_cursor(point);
    runner.sync_and_update();
    runner.release_cursor(point);
    runner.sync_and_update();
    runner.send_event(PlatformEvent::Keyboard {
        name: KeyboardEventName::KeyDown,
        key: Key::Character("a".into()),
        code: Code::KeyA,
        modifiers: Modifiers::CONTROL,
    });
    runner.sync_and_update();
    runner.press_key(Key::Named(NamedKey::Backspace));
    if !text.is_empty() {
        runner.write_text(text);
    }
    runner.sync_and_update();
}
fn capture(runner: &mut TestingRunner, name: &str) {
    let deadline = std::time::Instant::now()
        + if name.contains("document") {
            Duration::from_millis(100)
        } else {
            Duration::from_secs(8)
        };
    loop {
        runner.poll(Duration::from_millis(16), Duration::from_millis(80));
        let bytes = runner.render();
        let pixels = petunia_design_io::import_raster(&bytes, 16 * 1024 * 1024).unwrap();
        if pixels
            .data
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| {
                p[..3]
                    .iter()
                    .zip([53u8, 75, 166])
                    .all(|(a, b)| a.abs_diff(b) <= 1)
            })
            .count()
            > 500
            || std::time::Instant::now() > deadline
        {
            break;
        }
    }
    if let Some(directory) = std::env::var_os("PETUNIA_UI_EVIDENCE_DIR") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        runner.render_to_file(directory.join(format!("studio-{name}.png")));
    }
}
fn fixture_ids(ui: &UiShell) -> Vec<ObjectId> {
    ui.shell
        .peek()
        .bridge
        .query_layers()
        .rows
        .iter()
        .map(|r| r.id)
        .collect()
}

#[test]
fn production_studio_fits_and_keeps_controls_in_en_and_pt_at_four_sizes() {
    for (width, height) in [(900., 600.), (1024., 768.), (1280., 800.), (1600., 1000.)] {
        for locale in [Locale::EnUs, Locale::PtBr] {
            let (mut runner, ui) = mount(width, height, locale.clone());
            for key in [
                "layers",
                "properties",
                "colors",
                "history",
                "navigator",
                "tasks",
                "swatches",
            ] {
                let title = ui.studio_text(key);
                let node = runner
                    .find(|node, el| {
                        Label::try_downcast(el)
                            .filter(|l| l.text == title)
                            .map(|_| node)
                    })
                    .unwrap();
                let a = node.layout().area;
                assert!(
                    a.min_x() >= 0. && a.max_x() <= width && a.max_y() <= height,
                    "{title}: {a:?}"
                );
                assert!(a.height() < 25.);
            }
            let s = ui.shell.peek();
            let camera = s.view_camera();
            let a = camera.doc_to_screen(GPoint::new(0., 0.));
            let b = camera.doc_to_screen(GPoint::new(800., 560.));
            assert!(
                a.x >= 35.
                    && a.y >= 35.
                    && b.x <= camera.viewport_width - 35.
                    && b.y <= camera.viewport_height - 35.
            );
            drop(s);
            capture(
                &mut runner,
                &format!(
                    "{}-{width:.0}x{height:.0}",
                    if locale == Locale::PtBr { "pt" } else { "en" }
                ),
            );
            let bytes = runner.render();
            let pixels = petunia_design_io::import_raster(&bytes, 16 * 1024 * 1024).unwrap();
            assert!(
                pixels
                    .data
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .filter(|p| p[..3]
                        .iter()
                        .zip([53u8, 75, 166])
                        .all(|(a, b)| a.abs_diff(b) <= 1))
                    .count()
                    > 500,
                "worker must display the actual ellipse"
            );
        }
    }
}
#[test]
fn studio_live_locale_change_relabels_panels_and_keeps_tab_state() {
    let (mut runner, mut ui) = mount(1280., 800., Locale::EnUs);
    click_label(&mut runner, &ui.studio_text("navigator"));
    ui.shell.write().bridge.set_locale(Locale::PtBr);
    runner.sync_and_update();
    assert_eq!(*ui.dock_tab.peek(), 4);
    assert!(runner
        .find(|_, el| Label::try_downcast(el).filter(|l| l.text == "Navegador"))
        .is_some());
    assert!(runner
        .find(|_, el| Label::try_downcast(el).filter(|l| l.text == "Navigator"))
        .is_none());
}
#[test]
fn studio_hide_restore_and_collapse_preserve_document_and_width() {
    let (mut runner, ui) = mount(900., 600., Locale::EnUs);
    let before = ui.shell.peek().bridge.session().unwrap().document().clone();
    ui.dock_width.clone().set(440.);
    runner.sync_and_update();
    click_control(&mut runner, &ui.studio_text("hide_studio"));
    assert!(!*ui.right_studio_open.peek());
    click_control(&mut runner, &ui.studio_text("show_studio"));
    assert!(*ui.right_studio_open.peek());
    assert_eq!(*ui.dock_width.peek(), 440.);
    click_control(&mut runner, &ui.studio_text("collapse"));
    assert!(!*ui.studio_upper_open.peek());
    assert_eq!(
        ui.shell.peek().bridge.session().unwrap().document(),
        &before
    );
    click_control(&mut runner, &ui.studio_text("expand"));
    assert!(*ui.studio_upper_open.peek());
}
#[test]
fn studio_new_session_fits_once_and_tab_return_restores_camera() {
    let (mut runner, mut ui) = mount(1280., 800., Locale::EnUs);
    let mut camera = ui.shell.peek().view_camera();
    camera.set_zoom(1.7);
    camera.pan(37., -29.);
    ui.shell.write().set_view_camera(camera.clone());
    ui.shell.write().new_document("Second").unwrap();
    runner.sync_and_update();
    let second = ui.shell.peek().view_camera();
    assert!(second.zoom < 1. && second.viewport_width > 300.);
    ui.shell.write().bridge.switch_session(0).unwrap();
    runner.sync_and_update();
    assert_eq!(ui.shell.peek().view_camera(), camera);
}
#[test]
fn vector_color_preserves_width_locked_objects_and_atomic_undo() {
    let (_runner, mut ui) = mount(1280., 800., Locale::EnUs);
    let ids = fixture_ids(&ui);
    let before = ui.shell.peek().bridge.session().unwrap().document().clone();
    ui.shell.write().bridge.set_selection(vec![ids[0], ids[1]]);
    studio::apply_color(&ui, "#57C784", false);
    let s = ui.shell.peek();
    for id in &ids[..2] {
        let o = s.bridge.session().unwrap().find_object(*id).unwrap();
        assert_eq!(o.stroke.as_deref(), Some("#57C784"));
        assert_eq!(
            o.stroke_width,
            before.find_object(*id).unwrap().stroke_width
        );
    }
    drop(s);
    ui.shell.write().bridge.undo().unwrap();
    assert_eq!(
        ui.shell.peek().bridge.session().unwrap().document(),
        &before
    );
    ui.shell
        .write()
        .bridge
        .submit_all(
            "Lock",
            vec![Command::SetLocked {
                id: ids[0],
                locked: true,
            }],
        )
        .unwrap();
    let old = ui
        .shell
        .peek()
        .bridge
        .session()
        .unwrap()
        .find_object(ids[0])
        .unwrap()
        .fill
        .clone();
    studio::apply_color(&ui, "#FFFFFF", true);
    assert_eq!(
        ui.shell
            .peek()
            .bridge
            .session()
            .unwrap()
            .find_object(ids[0])
            .unwrap()
            .fill,
        old
    );
    assert_eq!(
        ui.shell
            .peek()
            .bridge
            .session()
            .unwrap()
            .find_object(ids[1])
            .unwrap()
            .fill
            .as_deref(),
        Some("#FFFFFF")
    );
}
#[test]
fn pixel_color_changes_actual_brush_clears_ink_and_preserves_alpha() {
    let (_runner, mut ui) = mount(1280., 800., Locale::EnUs);
    ui.set_persona(petunia_design_application::surfaces::PERSONA_PHOTO.to_string());
    let before = ui.shell.peek().bridge.session().unwrap().document().clone();
    let brush = ui.shell.write().tools.photo_brush_tool().brush_settings();
    let mut settings = brush;
    settings.color[3] = 0.37;
    settings.ink = Some([0.2, 0.3, 0.4, 0.5]);
    ui.shell
        .write()
        .tools
        .photo_brush_tool_mut()
        .set_brush_settings(settings);
    studio::apply_color(&ui, "#4C8DE8", true);
    let brush = ui.shell.peek().tools.photo_brush_tool().brush_settings();
    assert_eq!(brush.ink, None);
    assert_eq!(brush.color[3], 0.37);
    assert!((brush.color[0] - 76. / 255.).abs() < 1e-6);
    assert_eq!(
        ui.shell.peek().bridge.session().unwrap().document(),
        &before
    );
}
#[test]
fn layer_collapse_is_session_scoped_and_never_deletes_descendants() {
    let (mut runner, mut ui) = mount(1280., 800., Locale::EnUs);
    let ids = fixture_ids(&ui);
    ui.shell.write().bridge.set_selection(vec![ids[1], ids[2]]);
    ui.shell
        .write()
        .bridge
        .dispatch_action(petunia_design_application::ActionRequest::without_payload(
            petunia_design_application::ActionId::new("ptnd.action.object.group"),
        ))
        .unwrap();
    runner.sync_and_update();
    let group = ui.shell.peek().bridge.selection().selected_ids[0];
    let before = ui.shell.peek().bridge.session().unwrap().document().clone();
    click_control(&mut runner, &ui.studio_text("collapse_layer"));
    let session = ui.shell.peek().bridge.session().unwrap().identity();
    assert!(ui.collapsed_layers.peek().contains(&(session, group)));
    assert!(runner
        .find(
            |_, el| Rect::try_downcast(el).filter(|r| r.accessibility.builder.role()
                == AccessibilityRole::TreeItem
                && r.accessibility.builder.label() == Some("Coral rectangle"))
        )
        .is_none());
    assert_eq!(
        ui.shell.peek().bridge.session().unwrap().document(),
        &before
    );
    ui.shell.write().new_document("Second").unwrap();
    runner.sync_and_update();
    ui.shell.write().bridge.switch_session(0).unwrap();
    runner.sync_and_update();
    assert!(ui.collapsed_layers.peek().contains(&(session, group)));
    click_control(&mut runner, &ui.studio_text("expand_layer"));
    assert!(runner
        .find(|_, el| Rect::try_downcast(el)
            .filter(|r| r.accessibility.builder.label() == Some("Coral rectangle")))
        .is_some());
}
#[test]
fn navigator_contains_actual_ellipse_and_pans_without_mutating_document() {
    let (mut runner, mut ui) = mount(1280., 800., Locale::EnUs);
    ui.dock_tab.set(4);
    runner.sync_and_update();
    capture(&mut runner, "navigator");
    let area = runner
        .find(|node, el| {
            Rect::try_downcast(el)
                .filter(|r| r.accessibility.builder.role() == AccessibilityRole::Image)
                .map(|_| node)
        })
        .unwrap()
        .layout()
        .area;
    let deadline = std::time::Instant::now() + Duration::from_secs(8);
    loop {
        runner.poll(Duration::from_millis(16), Duration::from_millis(80));
        let bytes = runner.render();
        let pixels = petunia_design_io::import_raster(&bytes, 16 * 1024 * 1024).unwrap();
        let mut count = 0;
        for y in area.min_y() as usize..area.max_y() as usize {
            for x in area.min_x() as usize..area.max_x() as usize {
                let i = (y * pixels.width as usize + x) * 4;
                if pixels.data[i..i + 3]
                    .iter()
                    .zip([53u8, 75, 166])
                    .all(|(a, b)| a.abs_diff(b) <= 1)
                {
                    count += 1;
                }
            }
        }
        if count > 150 {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "thumbnail worker did not publish"
        );
    }
    capture(&mut runner, "navigator");
    let bytes = runner.render();
    let pixels = petunia_design_io::import_raster(&bytes, 16 * 1024 * 1024).unwrap();
    let mut count = 0;
    for y in area.min_y() as usize..area.max_y() as usize {
        for x in area.min_x() as usize..area.max_x() as usize {
            let i = (y * pixels.width as usize + x) * 4;
            if pixels.data[i..i + 3]
                .iter()
                .zip([53u8, 75, 166])
                .all(|(a, b)| a.abs_diff(b) <= 1)
            {
                count += 1;
            }
        }
    }
    assert!(count > 150, "navigator must render blue ellipse pixels");
    let before = ui.shell.peek().bridge.session().unwrap().document().clone();
    let old = ui.shell.peek().view_camera();
    runner.click_cursor((f64::from(area.min_x() + 25.), f64::from(area.min_y() + 25.)));
    runner.sync_and_update();
    let new = ui.shell.peek().view_camera();
    assert_eq!(new.zoom, old.zoom);
    assert_ne!((new.pan_x, new.pan_y), (old.pan_x, old.pan_y));
    assert_eq!(
        ui.shell.peek().bridge.session().unwrap().document(),
        &before
    );
}
#[test]
fn new_document_invalid_input_creates_no_tab_then_exact_values_commit() {
    let (mut runner, ui) = mount(1024., 768., Locale::PtBr);
    ui.new_doc_open.clone().set(true);
    runner.sync_and_update();
    runner.poll(Duration::from_millis(16), Duration::from_millis(200));
    replace_input(&mut runner, 1, "0");
    capture(&mut runner, "new-document-invalid-pt");
    click_label(&mut runner, &ui.text("create"));
    assert_eq!(ui.shell.peek().bridge.sessions().len(), 1);
    assert!(*ui.new_doc_open.peek());
    replace_input(&mut runner, 0, "Documento exato");
    replace_input(&mut runner, 1, "612,5 pt");
    replace_input(&mut runner, 2, "400");
    replace_input(&mut runner, 3, "3,5");
    replace_input(&mut runner, 4, "12 pt");
    assert!(
        runner
            .find(|_, el| Label::try_downcast(el).filter(|l| l.text.starts_with("Largura:")))
            .is_none(),
        "editing clears the previous validation error"
    );
    capture(&mut runner, "new-document-exact-pt");
    click_label(&mut runner, &ui.text("create"));
    let s = ui.shell.peek();
    assert_eq!(s.bridge.sessions().len(), 2);
    let session = s.bridge.session().unwrap();
    assert_eq!(session.title(), "Documento exato");
    let surface = session.surface(session.active_surface().unwrap()).unwrap();
    assert_eq!(surface.dimensions, [612.5, 400.]);
    assert_eq!(
        surface.bleed,
        petunia_design_document::surface_metadata::Bleed::uniform(3.5)
    );
    assert_eq!(
        surface.margins,
        petunia_design_document::surface_metadata::Margins::uniform(12.)
    );
}
#[test]
fn focused_inspector_button_activates_by_keyboard_without_deleting_canvas() {
    let (mut runner, ui) = mount(1280., 800., Locale::EnUs);
    let before = ui.shell.peek().bridge.session().unwrap().document().clone();
    let id = runner
        .find(|_, el| {
            Rect::try_downcast(el)
                .filter(|r| {
                    r.accessibility.builder.label() == Some(ui.studio_text("navigator").as_str())
                })
                .map(|r| r.accessibility.a11y_id.unwrap())
        })
        .unwrap();
    runner.run_in(|| id.request_focus());
    runner.sync_and_update();
    runner.press_key(Key::Named(NamedKey::Enter));
    assert_eq!(*ui.dock_tab.peek(), 4);
    runner.press_key(Key::Named(NamedKey::Delete));
    assert_eq!(
        ui.shell.peek().bridge.session().unwrap().document(),
        &before
    );
}
#[test]
fn color_hex_and_hsv_accept_valid_triplets_and_reject_partial_input() {
    assert_eq!(studio::parse_hex(" #4c8De8 "), Some([76, 141, 232]));
    for bad in ["4C8DE8", "#FFF", "#GG0000", "#FFFFFF00", "#ééé"] {
        assert!(studio::parse_hex(bad).is_none());
    }
    for rgb in [
        [1., 0., 0.],
        [0., 1., 0.],
        [0., 0., 1.],
        [0., 0., 0.],
        [1., 1., 1.],
        [0.15, 0.7, 0.92],
    ] {
        let [h, s, v] = studio::rgb_to_hsv(rgb);
        let out = studio::hsv_to_rgb(h, s, v);
        for (a, b) in rgb.into_iter().zip(out) {
            assert!((a - b).abs() < 1e-5);
        }
    }
    assert_eq!(studio::rgb_token([-1., 2., 0.5]), "#00FF80");
}
#[test]
fn studio_tokens_meet_text_contrast_and_preserve_semantic_outline_fallback() {
    fn luminance(color: Color) -> f64 {
        let linear = |v: u8| {
            let v = f64::from(v) / 255.;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r()) + 0.7152 * linear(color.g()) + 0.0722 * linear(color.b())
    }
    for foreground in [
        theme::TEXT_PRIMARY,
        theme::TEXT_SECONDARY,
        theme::TEXT_TERTIARY,
    ] {
        for background in [
            theme::SURFACE_CHROME,
            theme::SURFACE_PANEL,
            theme::SURFACE_SELECTED,
        ] {
            let a = luminance(foreground);
            let b = luminance(background);
            assert!((a.max(b) + 0.05) / (a.min(b) + 0.05) >= 4.5);
        }
    }
    for accent in theme::ACCENTS {
        assert!(
            (luminance(accent.value) + 0.05) / (luminance(theme::TEXT_ON_ACCENT) + 0.05) >= 4.5
        );
    }
    for icon in [
        theme::ICON_UNDO,
        theme::ICON_REDO,
        theme::ICON_FIT,
        theme::ICON_EYE_OFF,
        theme::ICON_ROTATE,
    ] {
        assert_eq!(icon.bytes(theme::IconStyle::Filled), icon.outline);
    }
}

fn color_pixels_in(runner: &mut TestingRunner, bounds: [usize; 4], rgb: [u8; 3]) -> usize {
    let bytes = runner.render();
    let pixels = petunia_design_io::import_raster(&bytes, 16 * 1024 * 1024).unwrap();
    let mut count = 0;
    for y in bounds[1]..bounds[3].min(pixels.height as usize) {
        for x in bounds[0]..bounds[2].min(pixels.width as usize) {
            let i = (y * pixels.width as usize + x) * 4;
            if pixels.data[i..i + 3]
                .iter()
                .zip(rgb)
                .all(|(a, b)| a.abs_diff(b) <= 1)
            {
                count += 1;
            }
        }
    }
    count
}

#[test]
fn small_window_two_column_rail_and_all_docks_preserve_editable_viewport() {
    let (mut runner, mut ui) = mount(900., 600., Locale::PtBr);
    ui.tool_rail.write().columns = ui_state::RailColumns::Two;
    ui.left_dock_open.set(true);
    ui.left_dock_width.set(480.);
    ui.dock_width.set(480.);
    ui.bottom_dock_open.set(true);
    ui.bottom_dock_height.set(420.);
    runner.poll(Duration::from_millis(16), Duration::from_millis(100));
    let camera = ui.shell.peek().view_camera();
    assert!(camera.viewport_width >= 240., "{:?}", camera);
    assert!(
        (240. ..=260.).contains(&camera.viewport_height),
        "bottom dock must reserve space: {:?}",
        camera
    );
    assert_eq!(*ui.left_dock_width.peek(), 480.);
    assert_eq!(*ui.dock_width.peek(), 480.);
    assert_eq!(*ui.bottom_dock_height.peek(), 420.);
    let mut xs = runner.find_many(|node, el| {
        Rect::try_downcast(el)
            .filter(|r| {
                r.accessibility.builder.role() == AccessibilityRole::Button
                    && node.layout().area.min_x() < 112.
                    && node.layout().area.min_y() > 135.
            })
            .map(|_| node.layout().area.min_x().round() as i32)
    });
    xs.sort_unstable();
    xs.dedup();
    assert!(xs.len() >= 2, "tool buttons must occupy both columns");
    capture(&mut runner, "two-columns-all-docks-900-pt");
}

#[test]
fn hex_editor_keeps_invalid_draft_and_applies_exact_vector_color() {
    let (mut runner, mut ui) = mount(1600., 1000., Locale::EnUs);
    let id = fixture_ids(&ui)[2];
    ui.shell.write().bridge.set_selection(vec![id]);
    ui.studio_upper_open.set(false);
    ui.dock_tab.set(2);
    runner.sync_and_update();
    let before = ui.shell.peek().bridge.session().unwrap().document().clone();
    replace_input(&mut runner, 0, "#GG0000");
    click_label(&mut runner, &ui.text("color_apply_fill"));
    assert_eq!(
        ui.shell.peek().bridge.session().unwrap().document(),
        &before
    );
    replace_input(&mut runner, 0, "#57C784");
    capture(&mut runner, "hex-editor-en");
    click_label(&mut runner, &ui.text("color_apply_fill"));
    assert_eq!(
        ui.shell
            .peek()
            .bridge
            .session()
            .unwrap()
            .find_object(id)
            .unwrap()
            .fill
            .as_deref(),
        Some("#57C784")
    );
    ui.shell.write().bridge.undo().unwrap();
    assert_eq!(
        ui.shell.peek().bridge.session().unwrap().document(),
        &before
    );
}

#[test]
fn favorite_swatches_add_real_current_color_once_and_survive_panel_remount() {
    let (mut runner, mut ui) = mount(1280., 800., Locale::EnUs);
    let id = fixture_ids(&ui)[2];
    ui.shell.write().bridge.set_selection(vec![id]);
    let color = ui
        .shell
        .peek()
        .bridge
        .session()
        .unwrap()
        .find_object(id)
        .unwrap()
        .fill
        .clone()
        .unwrap();
    ui.studio_upper_tab.set(1);
    runner.sync_and_update();
    click_control(&mut runner, &ui.studio_text("favorites"));
    click_control(&mut runner, &ui.studio_text("add_selected_color"));
    click_control(&mut runner, &ui.studio_text("add_selected_color"));
    assert_eq!(ui.favorite_swatches.peek().len(), 1);
    assert_eq!(ui.favorite_swatches.peek()[0].1, color);
    ui.right_studio_open.set(false);
    runner.sync_and_update();
    ui.right_studio_open.set(true);
    runner.sync_and_update();
    click_control(&mut runner, &ui.studio_text("favorites"));
    assert_eq!(ui.favorite_swatches.peek()[0].1, color);
    capture(&mut runner, "favorites-en");
}

#[test]
fn preferences_reset_real_workspace_and_palette_handles_no_results() {
    let (mut runner, mut ui) = mount(1024., 768., Locale::PtBr);
    let before = ui.shell.peek().bridge.session().unwrap().document().clone();
    ui.tool_rail.write().columns = ui_state::RailColumns::Two;
    ui.right_studio_open.set(false);
    ui.left_dock_open.set(true);
    ui.bottom_dock_open.set(true);
    ui.dock_width.set(480.);
    ui.customize_open.set(true);
    runner.sync_and_update();
    click_control(&mut runner, &ui.studio_text("preferences_general"));
    capture(&mut runner, "preferences-general-pt");
    click_label(&mut runner, &ui.studio_text("reset_workspace"));
    assert_eq!(ui.tool_rail.peek().columns, ui_state::RailColumns::One);
    assert!(*ui.right_studio_open.peek());
    assert!(!*ui.left_dock_open.peek() && !*ui.bottom_dock_open.peek());
    assert_eq!(*ui.dock_width.peek(), 320.);
    runner.press_key(Key::Named(NamedKey::Escape));
    ui.palette_open.set(true);
    runner.sync_and_update();
    replace_input(&mut runner, 0, "zzzz_unmatched_command");
    assert!(runner
        .find(|_, el| Label::try_downcast(el).filter(|l| l.text == ui.studio_text("no_commands")))
        .is_some());
    capture(&mut runner, "palette-empty-pt");
    runner.press_key(Key::Named(NamedKey::Escape));
    assert!(!*ui.palette_open.peek());
    assert_eq!(
        ui.shell.peek().bridge.session().unwrap().document(),
        &before
    );
}

#[test]
fn cached_canvas_publishes_selection_overlays_without_document_revision() {
    let (mut runner, mut ui) = mount(1280., 800., Locale::EnUs);
    capture(&mut runner, "selection-baseline");
    let bounds = [56, 135, 954, 775];
    assert_eq!(color_pixels_in(&mut runner, bounds, [183, 122, 255]), 0);
    let before = ui.shell.peek().bridge.session().unwrap().document().clone();
    let id = fixture_ids(&ui)[2];
    ui.shell.write().bridge.set_selection(vec![id]);
    runner.sync_and_update();
    runner.sync_and_update();
    assert!(
        color_pixels_in(&mut runner, bounds, [183, 122, 255]) > 25,
        "selection must republish immediately"
    );
    ui.shell.write().bridge.clear_selection();
    runner.sync_and_update();
    runner.sync_and_update();
    assert_eq!(
        color_pixels_in(&mut runner, bounds, [183, 122, 255]),
        0,
        "cleared handles must disappear"
    );
    assert_eq!(
        ui.shell.peek().bridge.session().unwrap().document(),
        &before
    );
}

#[test]
fn pixel_studio_exposes_real_painting_tools_and_context_colors() {
    let (mut runner, ui) = mount(1280., 800., Locale::EnUs);
    click_control(&mut runner, &ui.studio_text("pixel_studio"));
    assert_eq!(
        ui.persona.peek().as_str(),
        petunia_design_application::surfaces::PERSONA_PHOTO
    );
    let available = ui
        .tool_rail
        .peek()
        .groups(true)
        .iter()
        .flat_map(|g| g.tools.iter())
        .copied()
        .collect::<Vec<_>>();
    for tool in [
        ToolKind::PixelPaintBrush,
        ToolKind::PixelEraser,
        ToolKind::PixelFill,
    ] {
        assert!(available.contains(&tool));
    }
    ui.activate_tool(ToolKind::PixelPaintBrush);
    runner.sync_and_update();
    studio::apply_color(&ui, "#57C784", true);
    runner.sync_and_update();
    let rgb = ui
        .shell
        .peek()
        .tools
        .photo_brush_tool()
        .brush_settings()
        .color;
    assert!((rgb[0] - 87. / 255.).abs() < 0.001);
    assert!(runner
        .find(|_, el| Label::try_downcast(el).filter(|l| l.text.starts_with("Hardness")))
        .is_some());
    assert!(runner
        .find(|_, el| Label::try_downcast(el).filter(|l| l.text.starts_with("Opacity")))
        .is_some());
    capture(&mut runner, "pixel-brush-en");
}

#[test]
fn splitter_drag_captures_outside_edge_and_keyboard_resize_preserves_document() {
    let (mut runner, ui) = mount(1280., 800., Locale::EnUs);
    let before = ui.shell.peek().bridge.session().unwrap().document().clone();
    let (area, id) = runner
        .find(|node, el| {
            Rect::try_downcast(el)
                .filter(|r| {
                    r.accessibility.builder.role() == AccessibilityRole::Splitter
                        && r.accessibility.builder.label()
                            == Some(ui.studio_text("resize_studio").as_str())
                })
                .map(|r| (node.layout().area, r.accessibility.a11y_id.unwrap()))
        })
        .unwrap();
    let point = (f64::from(area.center().x), f64::from(area.center().y));
    runner.press_cursor(point);
    runner.sync_and_update();
    runner.move_cursor((point.0 - 100., point.1));
    assert_eq!(*ui.dock_width.peek(), 420.);
    runner.release_cursor((point.0 - 100., point.1));
    runner.sync_and_update();
    runner.move_cursor((point.0 - 160., point.1));
    assert_eq!(*ui.dock_width.peek(), 420., "release must end the drag");
    runner.run_in(|| id.request_focus());
    runner.sync_and_update();
    runner.press_key(Key::Named(NamedKey::ArrowRight));
    assert_eq!(*ui.dock_width.peek(), 412.);
    runner.press_key(Key::Named(NamedKey::Delete));
    assert_eq!(
        ui.shell.peek().bridge.session().unwrap().document(),
        &before
    );
}
