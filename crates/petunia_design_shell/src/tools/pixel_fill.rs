//! A cancellable fill worker owns only a disposable immutable-source draft.
//! UI polling publishes one command transaction after revision/identity guards.
use super::PhotoBrushSettings;
use crate::bridge::PetuniaDesignGuiBridge;
use petunia_design_application::{
    interaction::{NormalizedPointerEvent, PointerButton, PointerPhase},
    raster_edit::{RasterBrush, RasterStroke},
};
use petunia_design_document::ChangeSet;
use petunia_design_foundation::PetuniaError;
use petunia_design_jobs::{JobExecutor, JobHandle, JobManager};

#[derive(Default)]
pub struct PixelFillTool {
    executor: Option<JobExecutor>,
    pending: Option<JobHandle<RasterStroke>>,
    source: Option<(
        petunia_design_application::session::SessionIdentity,
        u64,
        Option<petunia_design_foundation::SurfaceId>,
    )>,
}
impl std::fmt::Debug for PixelFillTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PixelFillTool")
            .field("pending", &self.pending.is_some())
            .finish()
    }
}
impl PixelFillTool {
    pub fn cancel(&mut self) {
        self.pending = None;
        self.source = None;
    }
    pub fn is_pending(&self) -> bool {
        self.pending.is_some()
    }
    pub fn on_pointer_event(
        &mut self,
        event: &NormalizedPointerEvent,
        bridge: &mut PetuniaDesignGuiBridge,
        settings: PhotoBrushSettings,
    ) -> Result<ChangeSet, PetuniaError> {
        if event.phase == PointerPhase::Cancel {
            self.cancel();
            return Ok(ChangeSet::empty());
        }
        if event.phase != PointerPhase::Down || event.button != PointerButton::Primary {
            return Ok(ChangeSet::empty());
        }
        self.cancel();
        let session = bridge
            .session()
            .ok_or_else(|| PetuniaError::invalid_input("no fill session"))?;
        let mut draft = RasterStroke::begin(
            session,
            RasterBrush {
                radius: 1.0,
                hardness: 1.0,
                flow: 1.0,
                opacity: settings.opacity,
                color: settings.color,
                ink: settings.ink,
                erase: false,
            },
        )?;
        let revision = session.current_revision();
        self.source = Some((session.identity(), revision, session.active_surface()));
        if self.executor.is_none() {
            self.executor = Some(
                JobExecutor::new(1, 1, JobManager::new())
                    .map_err(|e| PetuniaError::invalid_input(e.to_string()))?,
            );
        }
        let point = event.doc_pos;
        self.pending = Some(
            self.executor
                .as_ref()
                .expect("executor created")
                .submit("pixel-fill", revision, move |context| {
                    context.check_cancelled()?;
                    draft
                        .flood_fill(point, 0.0, &|| context.cancellation().is_cancelled())
                        .map_err(|e| petunia_design_jobs::JobFailure::Failed(e.to_string()))?;
                    context.check_cancelled()?;
                    Ok(draft)
                })
                .map_err(|e| PetuniaError::invalid_input(e.to_string()))?,
        );
        Ok(ChangeSet::empty())
    }
    pub fn poll(&mut self, bridge: &mut PetuniaDesignGuiBridge) -> Result<ChangeSet, PetuniaError> {
        let Some(handle) = &self.pending else {
            return Ok(ChangeSet::empty());
        };
        let Some(session) = bridge.session() else {
            self.cancel();
            return Ok(ChangeSet::empty());
        };
        if self.source
            != Some((
                session.identity(),
                session.current_revision(),
                session.active_surface(),
            ))
        {
            self.cancel();
            return Err(PetuniaError::invalid_input(
                "pixel fill cancelled because its document changed",
            ));
        }
        match handle.try_result(session.current_revision()) {
            Ok(None) => Ok(ChangeSet::empty()),
            Ok(Some(draft)) => {
                self.pending = None;
                bridge.commit_raster_stroke(draft)
            }
            Err(error) => {
                self.pending = None;
                Err(PetuniaError::invalid_input(error.to_string()))
            }
        }
    }
}
