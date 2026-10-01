use petunia_design_document::resolve_color_to_rgb;

#[test]
fn malformed_unicode_hex_colors_never_panic() {
    for token in ["#€abc", "#éabcd", "#💐ab", "#€", "#éa"] {
        let color = resolve_color_to_rgb(token);
        assert!(color.iter().all(|value| value.is_finite()));
    }
    assert_eq!(resolve_color_to_rgb("#ff0000"), [1.0, 0.0, 0.0]);
    assert_eq!(resolve_color_to_rgb("#f00"), [1.0, 0.0, 0.0]);
}
