use aubrieta_color::{
    Cmyk, ColorManagementProvider, ColorValue, DefaultColorManagementProvider,
    PreserveNumbersPolicy, ProofContext, Srgb,
};
use proptest::prelude::*;

proptest! {
    #[test]
    fn cmyk_preserve_all_is_pure_identity(
        c in 0.0f32..=1.0f32,
        m in 0.0f32..=1.0f32,
        y in 0.0f32..=1.0f32,
        k in 0.0f32..=1.0f32,
    ) {
        let input = Cmyk { c, m, y, k };
        let output = PreserveNumbersPolicy::PreserveAll.apply(input, "US Web Coated (SWOP) v2");
        prop_assert_eq!(output, input);
    }

    #[test]
    fn preserve_black_only_never_modifies_k(
        c in 0.0f32..=1.0f32,
        m in 0.0f32..=1.0f32,
        y in 0.0f32..=1.0f32,
        k in 0.0f32..=1.0f32,
    ) {
        let input = Cmyk { c, m, y, k };
        let output = PreserveNumbersPolicy::PreserveBlackOnly.apply(input, "FOGRA39");
        prop_assert_eq!(output.k, k);
    }

    #[test]
    fn soft_proof_always_produces_finite_valid_srgb(
        r in 0.0f32..=1.0f32,
        g in 0.0f32..=1.0f32,
        b in 0.0f32..=1.0f32,
        gamut_warn in proptest::bool::ANY,
    ) {
        let provider = DefaultColorManagementProvider;
        let ctx = ProofContext::for_profile("US Web Coated (SWOP) v2").with_gamut_warning(gamut_warn);
        let color = ColorValue::Rgb(Srgb::clamped(r, g, b));

        let (preview, _status) = provider.soft_proof(&color, &ctx);

        prop_assert!(preview.r.is_finite());
        prop_assert!(preview.g.is_finite());
        prop_assert!(preview.b.is_finite());
        prop_assert!((0.0..=1.0).contains(&preview.r));
        prop_assert!((0.0..=1.0).contains(&preview.g));
        prop_assert!((0.0..=1.0).contains(&preview.b));
    }
}
