//! Capability registry: a missing capability is a normal state with a
//! disabled reason, never a panic. No lateral feature-to-feature access.

use std::collections::BTreeMap;

use petunia_design_foundation::PetuniaError;

/// Availability of one capability.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CapabilityState {
    /// Ready for use.
    Available,
    /// Present but unusable; carries the user-facing disabled reason.
    Disabled { reason: String },
}

/// Static description of one capability contribution.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityInfo {
    /// Stable capability ID, e.g. `ptnd.export.pdf`.
    pub id: String,
    /// Which crate/module provides it.
    pub provided_by: String,
    /// Current state.
    pub state: CapabilityState,
}

impl CapabilityInfo {
    /// Declares an available capability.
    #[must_use]
    pub fn available(id: impl Into<String>, provided_by: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            provided_by: provided_by.into(),
            state: CapabilityState::Available,
        }
    }

    /// Declares a disabled capability with an explicit reason.
    #[must_use]
    pub fn disabled(
        id: impl Into<String>,
        provided_by: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            provided_by: provided_by.into(),
            state: CapabilityState::Disabled {
                reason: reason.into(),
            },
        }
    }
}

/// Registry of provided capabilities and their states.
#[derive(Debug, Default)]
pub struct CapabilityRegistry {
    entries: BTreeMap<String, CapabilityInfo>,
}

impl CapabilityRegistry {
    /// Creates an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Registers or replaces one capability.
    pub fn register(&mut self, info: CapabilityInfo) {
        self.entries.insert(info.id.clone(), info);
    }

    /// Requires an available capability, else returns the disabled reason.
    pub fn require(&self, id: &str) -> Result<&CapabilityInfo, PetuniaError> {
        let info = self.entries.get(id).ok_or_else(|| {
            PetuniaError::capability_unavailable(format!("capability `{id}` is not registered"))
        })?;
        match &info.state {
            CapabilityState::Available => Ok(info),
            CapabilityState::Disabled { reason } => Err(PetuniaError::capability_unavailable(
                format!("capability `{id}` is disabled: {reason}"),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_capability_is_an_error_not_a_panic() {
        let registry = CapabilityRegistry::new();
        let err = registry
            .require("ptnd.export.pdf")
            .expect_err("missing");
        assert!(err.to_string().contains("not registered"));
    }

    #[test]
    fn disabled_capability_carries_reason() {
        let mut registry = CapabilityRegistry::new();
        registry.register(CapabilityInfo::disabled(
            "ptnd.export.pdf",
            "petunia_design_io",
            "PDF adapter not compiled in this build",
        ));
        let err = registry
            .require("ptnd.export.pdf")
            .expect_err("disabled");
        assert!(err.to_string().contains("not compiled"));
    }
}
