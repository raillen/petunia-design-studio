//! Artboard output preview uses the same immutable CPU composition as PNG.
//! Screen fitting changes sampling density, never document geometry or output DPI.
use crate::{canvas_preview, theme, ui_state::UiShell};
use freya::prelude::*;
use freya_engine::prelude::{Color, FilterMode, Paint, Rect as SkRect};
use petunia_design_application::{preview::PreviewSource, session::SessionIdentity};
use petunia_design_foundation::SurfaceId;
use petunia_design_shell::canvas::{CanvasOverlays, CanvasSnapshot, ViewportCamera};
use std::sync::Arc;
#[derive(Clone, PartialEq)]
pub struct ExportPreview(pub UiShell);
impl Component for ExportPreview {
    fn render(&self) -> impl IntoElement {
        let shell = self.0.shell.read();
        let input = shell.bridge.session().and_then(|session| {
            let surface = session
                .active_surface()
                .and_then(|id| session.document().surface(id).ok())?;
            Some((session.identity(), session.current_revision(), surface))
        });
        let mut source =
            use_state(|| None::<(SessionIdentity, u64, SurfaceId, Arc<PreviewSource>)>);
        let mut camera = ViewportCamera::new(500., 200.);
        if let Some((identity, revision, surface)) = input {
            let key = (identity, revision, surface.id);
            if source
                .peek()
                .as_ref()
                .is_none_or(|prior| (prior.0, prior.1, prior.2) != key)
            {
                source.set(Some((
                    identity,
                    revision,
                    surface.id,
                    PreviewSource::capture(surface, revision),
                )))
            }
            let [x, y, w, h] = surface.bounds();
            camera.fit_rect(petunia_design_geometry::GRect::new(x, y, x + w, y + h), 0.);
        } else if source.peek().is_some() {
            source.set(None)
        }
        let snapshot = CanvasSnapshot {
            revision: input.map_or(0, |(_, revision, _)| revision),
            preview_source: source.read().as_ref().map(|s| s.3.clone()),
            camera,
            overlays: CanvasOverlays::default(),
            surface: None,
            objects: Vec::new(),
        };
        drop(shell);
        let (preview, error) =
            canvas_preview::use_canvas_preview_with_background(&snapshot, 0, false, true);
        let frame_key = preview.as_ref().map(|p| p.image.unique_id());
        let on_render = RenderCallback::new(move |context: &mut CanvasContext| {
            let canvas = context.canvas;
            let mut paint = Paint::default();
            for row in 0..20 {
                for col in 0..50 {
                    paint.set_color(if (row + col) % 2 == 0 {
                        Color::from_rgb(224, 224, 224)
                    } else {
                        Color::from_rgb(250, 250, 250)
                    });
                    canvas.draw_rect(
                        SkRect::new(
                            (col * 10) as f32,
                            (row * 10) as f32,
                            ((col + 1) * 10) as f32,
                            ((row + 1) * 10) as f32,
                        ),
                        &paint,
                    );
                }
            }
            if let Some(preview) = preview.as_ref() {
                canvas.draw_image_rect_with_sampling_options(
                    &preview.image,
                    None,
                    SkRect::new(0., 0., 500., 200.),
                    FilterMode::Linear,
                    &Paint::default(),
                );
            }
        });
        rect()
            .direction(Direction::Vertical)
            .spacing(theme::SPACE_1)
            .child(label().text(self.0.text("export_preview")))
            .child(
                canvas(on_render)
                    .key((frame_key, crate::studio_widgets::canvas_render_epoch()))
                    .width(Size::px(500.))
                    .height(Size::px(200.)),
            )
            .children(
                error
                    .into_iter()
                    .map(|error| label().text(error).color(theme::TEXT_ERROR)),
            )
    }
}
