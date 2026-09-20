//! Structured diagnostics. Every failure names what broke and why.

use std::fmt;

/// Machine-readable diagnostic code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticCode(pub &'static str);

impl fmt::Display for DiagnosticCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

/// Human-facing diagnostic attached to an [`AubrietaError`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
}

impl Diagnostic {
    #[must_use]
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code: DiagnosticCode(code),
            message: message.into(),
        }
    }
}

/// Typed domain error. Libraries return this; binaries may wrap it.
#[derive(Debug, thiserror::Error)]
pub enum AubrietaError {
    /// Caller supplied data that violates a contract.
    #[error("invalid input [{code}]: {message}")]
    InvalidInput {
        code: DiagnosticCode,
        message: String,
    },
    /// Referenced entity does not exist.
    #[error("not found [{code}]: {message}")]
    NotFound {
        code: DiagnosticCode,
        message: String,
    },
    /// Capability is unavailable; carries the disabled reason.
    #[error("capability unavailable [{code}]: {message}")]
    CapabilityUnavailable {
        code: DiagnosticCode,
        message: String,
    },
    /// Persistence or IO failure with context.
    #[error("io [{code}]: {message}")]
    Io {
        code: DiagnosticCode,
        message: String,
    },
}

impl AubrietaError {
    /// Builds an invalid-input error explaining the violated contract.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            code: DiagnosticCode("aubrieta.invalid-input"),
            message: message.into(),
        }
    }

    /// Builds a not-found error naming the missing entity.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound {
            code: DiagnosticCode("aubrieta.not-found"),
            message: message.into(),
        }
    }

    /// Builds a capability-unavailable error with the disabled reason.
    pub fn capability_unavailable(message: impl Into<String>) -> Self {
        Self::CapabilityUnavailable {
            code: DiagnosticCode("aubrieta.capability-unavailable"),
            message: message.into(),
        }
    }

    /// Builds an IO error with context.
    pub fn io(message: impl Into<String>) -> Self {
        Self::Io {
            code: DiagnosticCode("aubrieta.io"),
            message: message.into(),
        }
    }

    /// Machine-readable code for telemetry and tests.
    #[must_use]
    pub fn code(&self) -> &DiagnosticCode {
        match self {
            Self::InvalidInput { code, .. }
            | Self::NotFound { code, .. }
            | Self::CapabilityUnavailable { code, .. }
            | Self::Io { code, .. } => code,
        }
    }
}
