#![forbid(unsafe_code)]

//! Executor-neutral job contracts: cancellation, progress, and task management for MVP.
//!
//! Background work declares cancellation behavior; dropping the owner must
//! not publish stale results. Workers accept immutable input and return values;
//! the owning application still publishes commands through its normal lane.

mod cancellation;
mod executor;
mod manager;

pub use cancellation::{CancellationToken, JobProgress};
pub use executor::{JobContext, JobExecutor, JobFailure, JobHandle, JobResult};
pub use manager::{JobInfo, JobManager, JobState};
