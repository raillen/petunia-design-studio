//! System environment inspection service contracts and headless adapter (09.17).

use crate::error::PlatformError;
use serde::{Deserialize, Serialize};

/// Snapshot of host system preferences and display metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SystemEnvironment {
    /// Preferred locales reported by operating system (e.g. `["en-US", "pt-BR"]`).
    pub preferred_locales: Vec<String>,
    /// Dark mode preference.
    pub dark_mode: bool,
    /// High contrast accessibility flag.
    pub high_contrast: bool,
    /// Primary monitor HiDPI scaling factor.
    pub scale_factor: f64,
}

impl Default for SystemEnvironment {
    fn default() -> Self {
        Self {
            preferred_locales: vec!["en-US".to_string(), "pt-BR".to_string()],
            dark_mode: false,
            high_contrast: false,
            scale_factor: 1.0,
        }
    }
}

/// Abstract contract for querying system environment state.
pub trait EnvironmentService: Send + Sync {
    /// Queries the current environment snapshot.
    fn get_environment(&self) -> Result<SystemEnvironment, PlatformError>;
}

/// Headless environment provider.
#[derive(Debug, Default, Clone)]
pub struct HeadlessEnvironment {
    env: SystemEnvironment,
}

impl HeadlessEnvironment {
    /// Creates a headless environment with defaults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets custom environment snapshot.
    #[must_use]
    pub fn with_environment(mut self, env: SystemEnvironment) -> Self {
        self.env = env;
        self
    }
}

impl EnvironmentService for HeadlessEnvironment {
    fn get_environment(&self) -> Result<SystemEnvironment, PlatformError> {
        Ok(self.env.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_env_defaults() {
        let svc = HeadlessEnvironment::new();
        let env = svc.get_environment().unwrap();
        assert_eq!(env.scale_factor, 1.0);
        assert!(!env.dark_mode);
        assert!(env.preferred_locales.contains(&"en-US".to_string()));
    }
}
