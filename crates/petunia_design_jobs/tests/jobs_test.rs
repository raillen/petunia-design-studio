use petunia_design_jobs::{JobManager, JobState};

#[test]
fn test_job_lifecycle_spawn_progress_complete() {
    let manager = JobManager::new();
    let (id, token) = manager.spawn_job("Exporting SVG");
    assert_eq!(manager.list_jobs().len(), 1);
    let job = &manager.list_jobs()[0];
    assert_eq!(job.id, id);
    assert_eq!(job.percent, 0);
    assert_eq!(job.state, JobState::Running);
    assert!(!token.is_cancelled());

    manager.update_progress(id, 45);
    assert_eq!(manager.list_jobs()[0].percent, 45);

    manager.complete_job(id);
    assert_eq!(manager.list_jobs()[0].state, JobState::Completed);
    assert_eq!(manager.list_jobs()[0].percent, 100);
}

#[test]
fn test_job_cancellation() {
    let manager = JobManager::new();
    let (id, token) = manager.spawn_job("Rasterizing FX");
    assert!(!token.is_cancelled());

    manager.cancel_job(id);
    assert!(token.is_cancelled());
    assert_eq!(manager.list_jobs()[0].state, JobState::Cancelled);
}

#[test]
fn test_clear_completed_jobs() {
    let manager = JobManager::new();
    let (id1, _) = manager.spawn_job("Job 1");
    let (id2, _) = manager.spawn_job("Job 2");
    manager.complete_job(id1);
    assert_eq!(manager.list_jobs().len(), 2);
    manager.clear_completed();
    assert_eq!(manager.list_jobs().len(), 1);
    assert_eq!(manager.list_jobs()[0].id, id2);
}
