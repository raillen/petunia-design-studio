//! Poll-only Freya adapter for the application histogram worker.
use freya::prelude::*;
use petunia_design_application::histogram::{Histogram, HistogramController, HistogramRequest};
use std::{cell::RefCell, rc::Rc, sync::Arc, time::Duration};
#[derive(Clone, Default)]
struct Publication {
    request: Option<HistogramRequest>,
    histogram: Option<Arc<Histogram>>,
    status: Option<String>,
}
impl PartialEq for Publication {
    fn eq(&self, other: &Self) -> bool {
        self.request == other.request
            && self.status == other.status
            && match (&self.histogram, &other.histogram) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
    }
}
pub fn use_histogram(next: Option<HistogramRequest>) -> (Option<Arc<Histogram>>, Option<String>) {
    let mut desired = use_state(|| next.clone());
    if *desired.peek() != next {
        desired.set(next.clone());
    }
    let owner = use_hook(|| Rc::new(RefCell::new(HistogramController::new())));
    use_drop({
        let owner = owner.clone();
        move || {
            if let Ok(controller) = owner.borrow_mut().as_mut() {
                controller.request(None);
            }
        }
    });
    let mut publication = use_state(Publication::default);
    use_future({
        let owner = owner.clone();
        move || {
            let next = desired.read().clone();
            let owner = owner.clone();
            async move {
                if let Ok(controller) = owner.borrow_mut().as_mut() {
                    controller.request(next.clone());
                }
                loop {
                    let (current, pending) = match owner.borrow_mut().as_mut() {
                        Ok(controller) => {
                            controller.poll();
                            (
                                Publication {
                                    request: next.clone(),
                                    histogram: controller.result(),
                                    status: controller.failure().map(ToString::to_string),
                                },
                                controller.is_pending(),
                            )
                        }
                        Err(error) => (
                            Publication {
                                request: next.clone(),
                                histogram: None,
                                status: Some(error.to_string()),
                            },
                            false,
                        ),
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
    let published = publication.read();
    if published.request == next {
        (published.histogram.clone(), published.status.clone())
    } else {
        (None, None)
    }
}
