#![forbid(unsafe_code)]

//! Executor-neutral job contracts: cancellation and progress for MVP.
//!
//! Background work declares cancellation behavior; dropping the owner must
//! not leave dangling jobs. This MVP cut is synchronous and deterministic.

mod cancellation;

pub use cancellation::{CancellationToken, JobProgress};
