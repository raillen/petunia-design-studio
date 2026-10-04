//! The Navigator reuses the authoritative composition worker, including raster,
//! text and effects; view gestures never modify the document.
use crate::{
    actions::run_action_token, canvas_preview, studio_widgets::StudioButton, theme,
    ui_state::UiShell,
};
use freya::prelude::*;
use freya_engine::prelude::{FilterMode, Paint, PaintStyle, Rect as SkRect};
use petunia_design_application::view_camera::ViewportCamera;
use petunia_design_geometry::{GPoint, GRect};

#[derive(Clone, PartialEq)]
pub struct NavigatorTab(pub UiShell);

impl Component for NavigatorTab {
    fn render(&self) -> impl IntoElement {
        let ui = &self.0;
        let mut size = use_state(|| (288.0, 180.0));
        let mut dragging = use_state(|| false);
        let mut snapshot = ui.shell.read().canvas_snapshot();
        let main_camera = snapshot.camera.clone();
        let zoom = main_camera.zoom;
        let bounds = snapshot.surface.as_ref().map(|s| s.bounds);
        let (width, height) = *size.read();
        let mut camera = ViewportCamera::new(width, height);
        if let Some([x, y, w, h]) = bounds {
            camera.fit_rect(GRect::new(x, y, x + w, y + h), 8.);
        }
        snapshot.camera = camera.clone();
        let proof = snapshot
            .preview_source
            .as_ref()
            .and_then(|source| source.surface_snapshot().cmyk_profile.clone())
            .zip(ui.monitor_profile.read().clone())
            .map(|(proof, monitor)| petunia_design_color::IccProofSettings {
                proof,
                monitor,
                options: *ui.proof_options.read(),
                proof_intent: ui.proof_options.read().intent,
            });
        let (preview, error) = canvas_preview::use_canvas_preview(
            &snapshot,
            *ui.channel_view.read(),
            *ui.soft_proof.read(),
            proof,
        );
        let focus_id = use_a11y();
        let focused = use_focus(focus_id);
        let accent = ui.accent.read().value;
        let frame_key = preview.as_ref().map(|p| p.image.unique_id());
        let camera_key = (
            camera.pan_x.to_bits(),
            camera.pan_y.to_bits(),
            camera.zoom.to_bits(),
            zoom.to_bits(),
        );
        let render_camera = camera.clone();
        let render = RenderCallback::new(move |context: &mut CanvasContext| {
            let canvas = context.canvas;
            canvas.clear(theme::SURFACE_WORKSPACE);
            let mut paint = Paint::default();
            paint.set_anti_alias(true);
            if let Some(preview) = &preview {
                let v = preview.viewport;
                let a = render_camera.doc_to_screen(GPoint::new(v.x0, v.y0));
                let b = render_camera.doc_to_screen(GPoint::new(v.x1, v.y1));
                canvas.draw_image_rect_with_sampling_options(
                    &preview.image,
                    None,
                    SkRect::new(a.x as f32, a.y as f32, b.x as f32, b.y as f32),
                    FilterMode::Linear,
                    &paint,
                );
            }
            let visible = main_camera.visible_doc_rect();
            let a = render_camera.doc_to_screen(GPoint::new(visible.x0, visible.y0));
            let b = render_camera.doc_to_screen(GPoint::new(visible.x1, visible.y1));
            paint.set_style(PaintStyle::Stroke);
            paint.set_stroke_width(1.5);
            paint.set_color(accent);
            canvas.draw_rect(
                SkRect::new(a.x as f32, a.y as f32, b.x as f32, b.y as f32),
                &paint,
            );
        });
        let down_ui = ui.clone();
        let move_ui = ui.clone();
        let key_ui = ui.clone();
        let down_camera = camera.clone();
        let move_camera = camera;
        rect()
            .direction(Direction::Vertical)
            .width(Size::fill())
            .spacing(theme::SPACE_2)
            .child(
                rect()
                    .direction(Direction::Horizontal)
                    .content(Content::Flex)
                    .width(Size::fill())
                    .cross_align(Alignment::Center)
                    .spacing(theme::SPACE_1)
                    .child(
                        label()
                            .width(Size::flex(1.))
                            .text(format!("{:.1}%", zoom * 100.))
                            .color(theme::TEXT_PRIMARY),
                    )
                    .children(
                        [
                            (
                                "zoom_out",
                                theme::ICON_ZOOM_OUT,
                                "ptnd.action.view.zoom_out",
                            ),
                            ("zoom_in", theme::ICON_ZOOM_IN, "ptnd.action.view.zoom_in"),
                            ("fit", theme::ICON_FIT, "ptnd.action.view.fit_surface"),
                        ]
                        .into_iter()
                        .map(|(key, icon, action)| {
                            let mut shell = ui.shell;
                            StudioButton::new(ui, ui.studio_text(key))
                                .icon(icon)
                                .width(Size::px(32.))
                                .on_press(move |_| {
                                    let _ = run_action_token(&mut shell.write(), action);
                                })
                        }),
                    )
                    .child({
                        let mut shell = ui.shell;
                        StudioButton::new(ui, "100%")
                            .text("100%")
                            .on_press(move |_| {
                                let _ = run_action_token(
                                    &mut shell.write(),
                                    "ptnd.action.view.zoom_100",
                                );
                            })
                    }),
            )
            .child(
                rect()
                    .width(Size::fill())
                    .height(Size::px(180.))
                    .a11y_id(focus_id)
                    .a11y_focusable(bounds.is_some())
                    .a11y_role(AccessibilityRole::Image)
                    .a11y_alt(ui.studio_text("navigator_hint"))
                    .border(
                        Border::new()
                            .fill(if focused() == Focus::Keyboard {
                                accent
                            } else {
                                theme::BORDER_SUBTLE
                            })
                            .width(2.)
                            .alignment(BorderAlignment::Inner),
                    )
                    .on_sized(move |event: Event<SizedEventData>| {
                        size.set_if_modified((
                            event.area.width() as f64,
                            event.area.height() as f64,
                        ));
                    })
                    .on_pointer_down(move |event: Event<PointerEventData>| {
                        event.stop_propagation();
                        if !event.is_primary() {
                            return;
                        }
                        dragging.set(true);
                        center_view(&down_ui, &down_camera, event.element_location(), bounds);
                    })
                    .on_pointer_move(move |event: Event<PointerEventData>| {
                        if *dragging.peek() {
                            event.stop_propagation();
                            center_view(&move_ui, &move_camera, event.element_location(), bounds);
                        }
                    })
                    .on_mouse_up(move |_| dragging.set(false))
                    .on_pointer_leave(move |_| dragging.set(false))
                    .on_key_down(move |event: Event<KeyboardEventData>| {
                        let delta = match event.key {
                            Key::Named(NamedKey::ArrowLeft) => Some((32., 0.)),
                            Key::Named(NamedKey::ArrowRight) => Some((-32., 0.)),
                            Key::Named(NamedKey::ArrowUp) => Some((0., 32.)),
                            Key::Named(NamedKey::ArrowDown) => Some((0., -32.)),
                            _ => None,
                        };
                        if let Some((x, y)) = delta {
                            event.stop_propagation();
                            event.prevent_default();
                            let mut shell = key_ui.shell;
                            let mut shell = shell.write();
                            let mut camera = shell.view_camera();
                            camera.pan(x, y);
                            shell.set_view_camera(camera);
                        }
                    })
                    .child(
                        canvas(render)
                            .key((
                                frame_key,
                                camera_key,
                                crate::studio_widgets::canvas_render_epoch(),
                            ))
                            .width(Size::fill())
                            .height(Size::fill()),
                    ),
            )
            .child(
                label()
                    .text(ui.studio_text("navigator_hint"))
                    .font_size(theme::CAPTION_SIZE)
                    .color(theme::TEXT_SECONDARY),
            )
            .children(error.into_iter().map(|reason| {
                label()
                    .text(reason)
                    .font_size(theme::CAPTION_SIZE)
                    .color(theme::TEXT_ERROR)
            }))
    }
}

fn center_view(
    ui: &UiShell,
    navigator: &ViewportCamera,
    point: freya::prelude::CursorPoint,
    bounds: Option<[f64; 4]>,
) {
    let Some([x, y, w, h]) = bounds else {
        return;
    };
    let target = navigator.screen_to_doc(GPoint::new(point.x, point.y));
    let mut shell = ui.shell;
    let mut shell = shell.write();
    let mut camera = shell.view_camera();
    camera.pan_x = camera.viewport_width / 2. - target.x.clamp(x, x + w) * camera.zoom;
    camera.pan_y = camera.viewport_height / 2. - target.y.clamp(y, y + h) * camera.zoom;
    shell.set_view_camera(camera);
}
