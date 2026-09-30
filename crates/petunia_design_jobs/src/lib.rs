#![forbid(unsafe_code)]

//! Executor-neutral job contracts: cancellation, progress, and task management for MVP.
//!
//! Background work declares cancellation behavior; dropping the owner must
//! not leave dangling jobs. This MVP cut is synchronous and deterministic.

mod cancellation;
mod manager;

pub use cancellation::{CancellationToken, JobProgress};
pub use manager::{JobInfo, JobManager, JobState};
