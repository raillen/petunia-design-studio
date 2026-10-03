//! Real workers exercised by synchronization, without throughput assertions.
use petunia_design_jobs::{JobExecutor, JobFailure, JobHandle, JobManager, JobState};
use std::sync::mpsc;
use std::time::{Duration, Instant};

fn result<T>(handle: &JobHandle<T>, revision: u64) -> Result<T, JobFailure> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(value) = handle.try_result(revision)? {
            return Ok(value);
        }
        assert!(Instant::now() < deadline, "worker failed to finish");
        std::thread::yield_now();
    }
}

#[test]
fn worker_executes_and_returns_revision_tagged_value() {
    let manager = JobManager::default();
    let executor = JobExecutor::new(2, 8, manager.clone()).unwrap();
    let owner = std::thread::current().id();
    let handle = executor
        .submit("geometry", 7, move |context| {
            assert_ne!(std::thread::current().id(), owner);
            context.check_cancelled()?;
            context.report_progress(50);
            Ok(42)
        })
        .unwrap();
    assert_eq!(result(&handle, 7).unwrap(), 42);
    assert_eq!(manager.list_jobs()[0].state, JobState::Completed);
    assert_eq!(manager.list_jobs()[0].percent, 100);
    executor.shutdown_and_join();
}

#[test]
fn stale_result_cannot_be_published_as_a_current_revision() {
    let executor = JobExecutor::new(1, 4, JobManager::new()).unwrap();
    let handle = executor.submit("snapshot", 3, |_| Ok(42)).unwrap();
    assert_eq!(
        result(&handle, 4),
        Err(JobFailure::StaleRevision {
            source_revision: 3,
            current_revision: 4
        })
    );
    executor.shutdown_and_join();
}

#[test]
fn pending_queue_is_bounded_without_registering_rejected_jobs() {
    let manager = JobManager::new();
    let executor = JobExecutor::new(1, 1, manager.clone()).unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let running = executor
        .submit("running", 0, move |_| {
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok(1)
        })
        .unwrap();
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let queued = executor.submit("queued", 0, |_| Ok(2)).unwrap();
    assert!(matches!(
        executor.submit("overflow", 0, |_| Ok(3)),
        Err(JobFailure::QueueFull)
    ));
    assert_eq!(manager.list_jobs().len(), 2);
    assert_eq!(manager.list_jobs()[1].state, JobState::Queued);
    release_tx.send(()).unwrap();
    assert_eq!(result(&running, 0).unwrap(), 1);
    assert_eq!(result(&queued, 0).unwrap(), 2);
    executor.shutdown_and_join();
}

#[test]
fn cancelled_pending_work_never_calls_its_task() {
    let executor = JobExecutor::new(1, 2, JobManager::new()).unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let running = executor
        .submit("barrier", 0, move |_| {
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok(())
        })
        .unwrap();
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let queued = executor
        .submit("cancelled", 0, |_| -> Result<(), JobFailure> {
            panic!("cancelled task ran")
        })
        .unwrap();
    queued.cancel();
    release_tx.send(()).unwrap();
    result(&running, 0).unwrap();
    assert_eq!(queued.try_result(0), Err(JobFailure::Cancelled));
    executor.shutdown_and_join();
}

#[test]
fn panic_marks_failure_and_worker_remains_available() {
    let manager = JobManager::new();
    let executor = JobExecutor::new(1, 4, manager.clone()).unwrap();
    let bad = executor
        .submit("panic", 0, |_| -> Result<(), JobFailure> {
            panic!("fixture")
        })
        .unwrap();
    assert_eq!(result(&bad, 0), Err(JobFailure::Panicked));
    assert_eq!(manager.list_jobs()[0].state, JobState::Failed);
    let good = executor.submit("next", 0, |_| Ok(9)).unwrap();
    assert_eq!(result(&good, 0).unwrap(), 9);
    executor.shutdown_and_join();
}

#[test]
fn dropping_owner_signals_running_worker_and_keeps_cancel_terminal() {
    let manager = JobManager::new();
    let executor = JobExecutor::new(1, 4, manager.clone()).unwrap();
    let (started_tx, started_rx) = mpsc::channel();
    let (cancel_tx, cancel_rx) = mpsc::channel();
    let handle = executor
        .submit("cooperative", 0, move |context| {
            started_tx.send(()).unwrap();
            while !context.cancellation().is_cancelled() {
                std::thread::yield_now();
            }
            cancel_tx.send(()).unwrap();
            context.check_cancelled()
        })
        .unwrap();
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    drop(handle);
    cancel_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    executor.shutdown_and_join();
    assert_eq!(manager.list_jobs()[0].state, JobState::Cancelled);
}

#[test]
fn invalid_executor_configuration_returns_an_error() {
    assert!(matches!(
        JobExecutor::new(0, 4, JobManager::new()),
        Err(JobFailure::InvalidConfiguration)
    ));
    assert!(matches!(
        JobExecutor::new(1, 0, JobManager::new()),
        Err(JobFailure::InvalidConfiguration)
    ));
}

#[test]
fn cancelling_queued_work_releases_admission_before_worker_is_available() {
    let executor = JobExecutor::new(1, 1, JobManager::new()).unwrap();
    let (started_tx, started_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let active = executor
        .submit("active", 1, move |_| {
            started_tx.send(()).unwrap();
            release_rx.recv().unwrap();
            Ok(())
        })
        .unwrap();
    started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let obsolete = executor.submit("obsolete", 1, |_| Ok(1u32)).unwrap();
    drop(obsolete);
    let fresh = executor.submit("fresh", 1, |_| Ok(2u32)).unwrap();
    release_tx.send(()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(value) = fresh.try_result(1).unwrap() {
            assert_eq!(value, 2);
            break;
        }
        assert!(Instant::now() < deadline);
        std::thread::yield_now();
    }
    drop(active);
    executor.shutdown_and_join();
}
