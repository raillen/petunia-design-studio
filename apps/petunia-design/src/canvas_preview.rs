//! Freya owns polling/upload only. Shaping, decoding and scene composition run
//! in the application worker; no toolkit type crosses that boundary.
use freya::prelude::*;
use freya_engine::prelude::{AlphaType, ColorSpace, ColorType, Data, Image, ImageInfo};
use petunia_design_application::preview::{PreviewController, PreviewFrame, PreviewRequest};
use petunia_design_geometry::GRect;
use petunia_design_shell::canvas::CanvasSnapshot;
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};

#[derive(Clone)]
pub struct PresentedPreview {
    pub image: Image,
    pub viewport: GRect,
}
#[derive(Clone, Default)]
struct Publication {
    request: Option<PreviewRequest>,
    frame: Option<Arc<PreviewFrame>>,
    error: Option<String>,
}
impl PartialEq for Publication {
    fn eq(&self, other: &Self) -> bool {
        self.request == other.request
            && self.error == other.error
            && match (&self.frame, &other.frame) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}
struct Upload {
    frame: Arc<PreviewFrame>,
    image: Image,
}

pub fn use_canvas_preview(
    snapshot: &CanvasSnapshot,
    channel: usize,
    soft_proof: bool,
    proof_settings: Option<petunia_design_color::IccProofSettings>,
) -> (Option<PresentedPreview>, Option<String>) {
    use_canvas_preview_inner(snapshot, channel, soft_proof, proof_settings, false)
}
pub fn use_canvas_preview_with_background(
    snapshot: &CanvasSnapshot,
    channel: usize,
    soft_proof: bool,
    transparent: bool,
) -> (Option<PresentedPreview>, Option<String>) {
    use_canvas_preview_inner(snapshot, channel, soft_proof, None, transparent)
}
fn use_canvas_preview_inner(
    snapshot: &CanvasSnapshot,
    channel: usize,
    soft_proof: bool,
    proof_settings: Option<petunia_design_color::IccProofSettings>,
    transparent: bool,
) -> (Option<PresentedPreview>, Option<String>) {
    let next = snapshot
        .preview_source
        .as_ref()
        .map(|source| {
            PreviewRequest::from_camera(source.clone(), &snapshot.camera, channel, soft_proof).map(
                |mut request| {
                    request.proof_settings = proof_settings.clone();
                    request.transparent_artboard = transparent;
                    request
                },
            )
        })
        .transpose();
    let input_error = next.as_ref().err().map(ToString::to_string);
    let next = next.ok().flatten();
    let mut desired = use_state(|| next.clone());
    if *desired.peek() != next {
        desired.set(next.clone());
    }
    let owner = use_hook(|| Rc::new(RefCell::new(PreviewController::new())));
    use_drop({
        let owner = owner.clone();
        move || {
            if let Ok(controller) = owner.borrow_mut().as_mut() {
                controller.clear();
            }
        }
    });
    let mut publication = use_state(Publication::default);
    use_future({
        let owner = owner.clone();
        move || {
            // Reading outside async subscribes this task to request replacement.
            let next = desired.read().clone();
            let owner = owner.clone();
            async move {
                if let Ok(controller) = owner.borrow_mut().as_mut() {
                    controller.request(next.clone());
                }
                loop {
                    let (current, pending) = {
                        let mut owner = owner.borrow_mut();
                        match owner.as_mut() {
                            Ok(controller) => {
                                controller.poll();
                                (
                                    Publication {
                                        request: next.clone(),
                                        frame: controller.frame(),
                                        error: controller.failure().map(ToString::to_string),
                                    },
                                    controller.is_pending(),
                                )
                            }
                            Err(error) => (
                                Publication {
                                    request: next.clone(),
                                    frame: None,
                                    error: Some(error.to_string()),
                                },
                                false,
                            ),
                        }
                    };
                    if *publication.peek() != current {
                        publication.set(current);
                    }
                    if !pending {
                        break;
                    }
                    timer(Duration::from_millis(16)).await;
                }
            }
        }
    });
    let uploads = use_hook(|| Rc::new(RefCell::new(None::<Upload>)));
    let published = publication.read();
    let frame = published.frame.as_ref().filter(|frame| {
        next.as_ref().is_some_and(|next| {
            next.source.id() == frame.request.source.id()
                && next.channel == frame.request.channel
                && next.soft_proof == frame.request.soft_proof
                && next.proof_settings == frame.request.proof_settings
                && next.transparent_artboard == frame.request.transparent_artboard
        })
    });
    let mut cache = uploads.borrow_mut();
    let mut error = input_error.or_else(|| {
        (published.request == next)
            .then(|| published.error.clone())
            .flatten()
    });
    if let Some(frame) = frame {
        if error.is_none() && !frame.warnings.is_empty() {
            error = Some(frame.warnings.join(" · "));
        }
        if cache
            .as_ref()
            .is_none_or(|upload| !Arc::ptr_eq(&upload.frame, frame))
        {
            let pixels = &frame.pixels;
            let info = ImageInfo::new(
                (pixels.width as i32, pixels.height as i32),
                ColorType::RGBA8888,
                AlphaType::Unpremul,
                ColorSpace::new_srgb(),
            );
            #[allow(deprecated)]
            let image = Image::from_raster_data(
                &info,
                Data::new_copy(&pixels.data),
                pixels.width as usize * 4,
            );
            *cache = image.map(|image| Upload {
                frame: frame.clone(),
                image,
            });
            if cache.is_none() {
                error = Some("preview image upload unavailable".into());
            }
        }
    } else {
        *cache = None;
    }
    let preview = cache.as_ref().map(|upload| PresentedPreview {
        image: upload.image.clone(),
        viewport: upload.frame.request.render.viewport,
    });
    (preview, error)
}
