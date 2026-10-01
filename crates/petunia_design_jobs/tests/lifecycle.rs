use petunia_design_jobs::{JobManager, JobState};

#[test]
fn cancellation_is_terminal_even_when_worker_completes_late() {
    let jobs = JobManager::new();
    let (id, token) = jobs.spawn_job("Work");
    jobs.update_progress(id, 30);
    jobs.cancel_job(id);
    assert!(token.is_cancelled());
    jobs.complete_job(id);
    jobs.fail_job(id);
    jobs.update_progress(id, 100);
    let info = jobs.list_jobs().pop().unwrap();
    assert_eq!(info.state, JobState::Cancelled);
    assert_eq!(info.percent, 30);
}

#[test]
fn completed_and_failed_jobs_cannot_change_terminal_state() {
    let jobs = JobManager::new();
    let (completed, token) = jobs.spawn_job("Complete");
    jobs.complete_job(completed);
    jobs.cancel_job(completed);
    jobs.fail_job(completed);
    assert!(!token.is_cancelled());
    let (failed, _) = jobs.spawn_job("Fail");
    jobs.fail_job(failed);
    jobs.complete_job(failed);
    jobs.cancel_job(failed);
    let list = jobs.list_jobs();
    assert_eq!(list[0].state, JobState::Completed);
    assert_eq!(list[1].state, JobState::Failed);
}
