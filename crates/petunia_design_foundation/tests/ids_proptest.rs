//! Property invariants for stable IDs: roundtrip, monotonicity, rejection.

use petunia_design_foundation::{IdGenerator, ObjectId, SurfaceId};
use proptest::prelude::*;

proptest! {
    /// Display/parse roundtrip holds for every raw value.
    #[test]
    fn object_id_display_roundtrip(raw in any::<u64>()) {
        let id = ObjectId::new(raw);
        let text = id.to_string();
        let parsed: ObjectId = text.parse().expect("roundtrip");
        prop_assert_eq!(parsed, id);
    }

    /// Surface IDs roundtrip independently of object IDs.
    #[test]
    fn surface_id_display_roundtrip(raw in any::<u64>()) {
        let id = SurfaceId::new(raw);
        let text = id.to_string();
        let parsed: SurfaceId = text.parse().expect("roundtrip");
        prop_assert_eq!(parsed, id);
    }

    /// A sequence of N generated IDs is strictly increasing and never zero.
    #[test]
    fn generator_sequence_is_strictly_increasing(n in 1..64usize) {
        let mut gen = IdGenerator::new();
        let mut previous = 0u64;
        for _ in 0..n {
            let next = gen.next_object().raw();
            prop_assert!(next > previous, "not increasing: {previous} -> {next}");
            previous = next;
        }
    }

    /// Cross-type confusion is always rejected.
    #[test]
    fn cross_type_prefix_is_rejected(raw in any::<u64>()) {
        let text = format!("SurfaceId:{raw}");
        prop_assert!(text.parse::<ObjectId>().is_err());
    }
}
