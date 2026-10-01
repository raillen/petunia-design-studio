//! Regression sources for the final MVP gate; execution is deferred.
//! Positive glyph cases require the Linux DejaVu font fixture installed by setup.
use petunia_design_text::{prepare_text, TextFrameSpec, TextRenderError};
use std::sync::Arc;
fn spec(text: &str) -> TextFrameSpec {
    TextFrameSpec {
        content: text.into(),
        family: "DejaVu Sans".into(),
        font_size: 24.0,
        line_height: 1.2,
        letter_spacing: 0.0,
        width: 400.0,
        weight: 400, italic: false, alignment: Default::default(), wrap: true,
    }
}
#[test]
fn prepared_outline_is_real_ink_with_shared_cache_ownership() {
    let s = spec("Petunia outline fixture");
    let a = prepare_text(&s, &|| false).unwrap();
    let b = prepare_text(&s, &|| false).unwrap();
    assert!(Arc::ptr_eq(&a, &b));
    assert!(!a.outline().is_empty());
    assert!(a.outline().is_finite());
    assert!(!a.glyphs().is_empty());
    assert!(a
        .glyphs()
        .iter()
        .all(|g| !g.font_family.is_empty() && g.glyph_id != 0));
}
#[test]
fn ligature_clusters_preserve_original_byte_ranges() {
    let s = spec("fi");
    let text = prepare_text(&s, &|| false).unwrap();
    assert_eq!(text.glyphs().len(), 1);
    assert_eq!(text.glyphs()[0].source_range, [0, 2]);
    assert_eq!(s.content, "fi");
}
#[test]
fn mixed_direction_text_keeps_valid_utf8_source_clusters() {
    let s = spec("abc אבג");
    let text = prepare_text(&s, &|| false).unwrap();
    assert!(text.glyphs().iter().any(|g| g.rtl));
    assert!(text.glyphs().iter().any(|g| !g.rtl));
    for g in text.glyphs() {
        assert!(s
            .content
            .get(g.source_range[0]..g.source_range[1])
            .is_some());
    }
}
#[test]
fn paragraph_offsets_handle_crlf_cr_and_lfcr() {
    let s = spec("a\r\nb\rc\n\rd");
    let text = prepare_text(&s, &|| false).unwrap();
    let mut starts: Vec<_> = text.glyphs().iter().map(|g| g.source_range[0]).collect();
    starts.sort_unstable();
    assert_eq!(starts, vec![0, 3, 5, 8]);
    assert_eq!(text.line_count(), 4);
}
#[test]
fn wrapping_and_line_height_change_flow_without_mutating_source() {
    let mut s = spec("one two three four five six seven eight");
    let wide = prepare_text(&s, &|| false).unwrap();
    s.width = 70.0;
    let narrow = prepare_text(&s, &|| false).unwrap();
    assert!(narrow.line_count() > wide.line_count());
    s.line_height = 2.0;
    let spaced = prepare_text(&s, &|| false).unwrap();
    assert!(spaced.flow_height() > narrow.flow_height());
    assert_eq!(s.content, "one two three four five six seven eight");
}
#[test]
fn tracking_is_converted_from_document_points_to_em() {
    let mut s = spec("ABC");
    let a = prepare_text(&s, &|| false).unwrap();
    s.letter_spacing = 3.0;
    let b = prepare_text(&s, &|| false).unwrap();
    let delta = b.glyphs()[1].origin.x - a.glyphs()[1].origin.x;
    assert!((delta - 3.0).abs() < 0.02);
}
#[test]
fn whitespace_and_empty_text_do_not_invent_frame_coverage() {
    for s in [spec(""), spec(" \t\n ")] {
        let text = prepare_text(&s, &|| false).unwrap();
        assert!(text.outline().is_empty());
        assert!(text.ink_bounds().is_none());
    }
}
#[test]
fn cancellation_also_rejects_warm_cache_results() {
    let s = spec("warm cancellation");
    prepare_text(&s, &|| false).unwrap();
    assert!(matches!(
        prepare_text(&s, &|| true),
        Err(TextRenderError::Cancelled)
    ));
}
#[test]
fn invalid_metrics_fail_before_system_font_preparation() {
    for value in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
        let mut s = spec("metrics");
        s.font_size = value;
        assert!(matches!(
            prepare_text(&s, &|| false),
            Err(TextRenderError::Invalid(_))
        ));
    }
    let mut s = spec("metrics");
    s.width = 0.0;
    assert!(prepare_text(&s, &|| false).is_err());
    s.width = 100.0;
    s.letter_spacing = f64::NAN;
    assert!(prepare_text(&s, &|| false).is_err());
}
#[test]
fn oversized_text_and_family_receive_resource_reasons() {
    let mut s = spec(&"a".repeat(65_537));
    assert!(matches!(
        prepare_text(&s, &|| false),
        Err(TextRenderError::Limit(_))
    ));
    s.content.clear();
    s.family = "x".repeat(1025);
    assert!(matches!(
        prepare_text(&s, &|| false),
        Err(TextRenderError::Limit(_))
    ));
}
