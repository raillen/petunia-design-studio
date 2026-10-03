//! ICC swatches are calculated by a bounded worker from actual profile bytes.
use freya::prelude::*;
use petunia_design_color::{IccProfile, IccTransformOptions};
use petunia_design_jobs::{JobExecutor, JobFailure, JobManager};
use std::{rc::Rc, time::Duration};
#[derive(Clone, PartialEq)]
struct Request {
    profile: IccProfile,
    ink: [f32; 4],
}
#[derive(Clone, Default, PartialEq)]
struct Publication {
    request: Option<Request>,
    color: Option<[u8; 3]>,
    error: Option<String>,
}
pub fn use_cmyk_swatch(
    profile: Option<IccProfile>,
    ink: [f32; 4],
) -> (Option<[u8; 3]>, Option<String>) {
    let next = profile.map(|profile| Request { profile, ink });
    let mut desired = use_state(|| next.clone());
    if *desired.peek() != next {
        desired.set(next.clone());
    }
    let owner = use_hook(|| Rc::new(JobExecutor::new(1, 1, JobManager::new())));
    let mut publication = use_state(Publication::default);
    use_future({
        let owner = owner.clone();
        move || {
            let request = desired.read().clone();
            let owner = owner.clone();
            async move {
                let Some(request) = request else {
                    publication.set(Publication::default());
                    return;
                };
                let result = match owner.as_ref() {
                    Err(error) => Err(error.clone()),
                    Ok(executor) => loop {
                        let input = request.clone();
                        match executor.submit("icc-swatch", 0, move |context| {
                            context.check_cancelled()?;
                            let rgb = petunia_design_color::icc::cmyk_to_rgb(
                                &input.profile,
                                &IccProfile::srgb()
                                    .map_err(|e| JobFailure::Failed(e.to_string()))?,
                                &[input.ink],
                                IccTransformOptions::default(),
                            )
                            .map_err(|e| JobFailure::Failed(e.to_string()))?;
                            context.check_cancelled()?;
                            Ok(rgb[0].map(|v| (v.clamp(0., 1.) * 255.).round() as u8))
                        }) {
                            Ok(handle) => {
                                break loop {
                                    match handle.try_result(0) {
                                        Ok(Some(value)) => break Ok(value),
                                        Err(error) => break Err(error),
                                        Ok(None) => timer(Duration::from_millis(16)).await,
                                    }
                                }
                            }
                            Err(JobFailure::QueueFull) => timer(Duration::from_millis(16)).await,
                            Err(error) => break Err(error),
                        }
                    },
                };
                publication.set(match result {
                    Ok(color) => Publication {
                        request: Some(request),
                        color: Some(color),
                        error: None,
                    },
                    Err(error) => Publication {
                        request: Some(request),
                        color: None,
                        error: Some(error.to_string()),
                    },
                });
            }
        }
    });
    let published = publication.read();
    if published.request != next {
        return (None, None);
    }
    (published.color, published.error.clone())
}
