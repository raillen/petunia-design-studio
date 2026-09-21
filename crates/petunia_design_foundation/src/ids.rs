//! Typed stable IDs. Never use a `Vec` index or pointer as identity.

use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! stable_id {
    ($name:ident) => {
        /// Stable namespaced entity identifier.
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub u64);

        impl $name {
            /// Creates an ID from a raw value. Prefer [`IdGenerator`] in app code.
            #[must_use]
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            /// Raw numeric value, for persistence and debugging only.
            #[must_use]
            pub const fn raw(self) -> u64 {
                self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", stringify!($name), self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}:{}", stringify!($name), self.0)
            }
        }

        impl std::str::FromStr for $name {
            type Err = crate::PetuniaError;

            fn from_str(text: &str) -> Result<Self, Self::Err> {
                let prefix = concat!(stringify!($name), ":");
                let digits = text.strip_prefix(prefix).ok_or_else(|| {
                    crate::PetuniaError::invalid_input(format!(
                        "expected `{}` prefix in `{text}`",
                        stringify!($name)
                    ))
                })?;
                let raw: u64 = digits.parse().map_err(|_| {
                    crate::PetuniaError::invalid_input(format!("invalid id `{text}`"))
                })?;
                Ok(Self(raw))
            }
        }
    };
}

stable_id!(ObjectId);
stable_id!(SurfaceId);
stable_id!(ResourceId);
stable_id!(StyleId);
stable_id!(EffectId);
stable_id!(TextStoryId);

/// Deterministic monotonic ID generator for tests and headless flows.
///
/// Production persistence must store the issued IDs; the generator itself is
/// not persisted in this MVP cut.
#[derive(Debug, Default)]
pub struct IdGenerator {
    next: u64,
}

impl IdGenerator {
    /// Starts a generator at 1. ID 0 is reserved as "none".
    #[must_use]
    pub fn new() -> Self {
        Self { next: 1 }
    }

    /// Starts a generator at an explicit value (at least 1).
    #[must_use]
    pub fn with_start(start: u64) -> Self {
        Self { next: start.max(1) }
    }

    /// Raises the counter so `raw` can never be issued again.
    ///
    /// Commands can introduce identities the generator never issued (a project
    /// loaded from disk, a plugin, a duplicate). Without observing them the
    /// next `next_object` would collide with an existing object.
    pub fn observe(&mut self, raw: u64) {
        self.next = self.next.max(raw.saturating_add(1));
    }

    /// Issues the next `ObjectId`.
    pub fn next_object(&mut self) -> ObjectId {
        let id = ObjectId(self.next);
        self.next += 1;
        id
    }

    /// Issues the next `SurfaceId`.
    pub fn next_surface(&mut self) -> SurfaceId {
        let id = SurfaceId(self.next);
        self.next += 1;
        id
    }

    /// Issues the next `ResourceId`.
    pub fn next_resource(&mut self) -> ResourceId {
        let id = ResourceId(self.next);
        self.next += 1;
        id
    }

    /// Issues the next `TextStoryId`.
    pub fn next_text_story(&mut self) -> TextStoryId {
        let id = TextStoryId(self.next);
        self.next += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable_across_display_roundtrip() {
        let id = ObjectId::new(42);
        let text = id.to_string();
        assert_eq!(text.parse::<ObjectId>().expect("roundtrip"), id);
    }

    #[test]
    fn generator_is_monotonic_and_never_zero() {
        let mut gen = IdGenerator::new();
        let a = gen.next_object().raw();
        let b = gen.next_object().raw();
        assert!(a >= 1);
        assert!(b > a);
    }

    #[test]
    fn wrong_prefix_is_rejected() {
        assert!("SurfaceId:7".parse::<ObjectId>().is_err());
    }
}
