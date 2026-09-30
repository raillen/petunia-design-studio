//! F7.1 text foundation: real shaping metrics over system fonts.
//!
//! These tests fail before `fonts.rs` exists and pass after.

use petunia_design_text::fonts::{ShapedGlyph, TypeSystem};

const FAMILY: &str = "Noto Sans";
const SIZE_PX: f32 = 32.0;

fn total_advance(glyphs: &[ShapedGlyph]) -> f32 {
    glyphs.iter().map(|g| g.advance_px).sum()
}

fn require_no_tofu(label: &str, script: &str, glyphs: &[ShapedGlyph]) {
    assert!(
        !glyphs.is_empty(),
        "shape_run returned no glyphs for {label}; cannot prove {script} coverage"
    );
    let tofu = glyphs.iter().filter(|g| g.glyph_id == 0).count();
    assert_eq!(
        tofu, 0,
        "silent .notdef for {label}: missing system coverage for {script}, \
         install a font covering it instead of ignoring tofu"
    );
}

#[test]
fn proportional_advances_differ() {
    let narrow = TypeSystem::shape_run("i", FAMILY, SIZE_PX);
    let wide = TypeSystem::shape_run("W", FAMILY, SIZE_PX);
    assert!(
        !narrow.is_empty() && !wide.is_empty(),
        "shape_run returned no glyphs; cannot compare proportional advances"
    );
    let narrow_adv = total_advance(&narrow);
    let wide_adv = total_advance(&wide);
    assert!(
        narrow_adv > 0.0 && wide_adv > 0.0,
        "advances must be positive, got i={narrow_adv} W={wide_adv}"
    );
    assert!(
        narrow_adv < wide_adv,
        "proportional font must shape i narrower than W at the same size, \
         got i={narrow_adv} W={wide_adv}"
    );
}

#[test]
fn fallback_chain_resolves_per_script() {
    let latin = TypeSystem::shape_run("a", FAMILY, SIZE_PX);
    require_no_tofu("Latin \"a\"", "Latin", &latin);
    let arabic = TypeSystem::shape_run("\u{0634}", FAMILY, SIZE_PX);
    require_no_tofu("Arabic SHEEN (U+0634)", "Arabic", &arabic);

    let latin_families: Vec<&str> = latin.iter().map(|g| g.fallback_family.as_str()).collect();
    let arabic_families: Vec<&str> = arabic.iter().map(|g| g.fallback_family.as_str()).collect();
    assert!(
        latin_families.iter().all(|f| !f.is_empty())
            && arabic_families.iter().all(|f| !f.is_empty()),
        "fallback must report a real family name, got Latin={latin_families:?} \
         Arabic={arabic_families:?}"
    );
    assert!(
        !latin_families.iter().any(|f| arabic_families.contains(f)),
        "fallback chain must resolve per-script fonts for Latin vs Arabic, \
         got Latin={latin_families:?} Arabic={arabic_families:?}"
    );
}

#[test]
fn multilingual_shapes_without_tofu() {
    // Latin "a" + Arabic SHEEN (U+0634) + CJK IDEOGRAPHIC COMMA (U+3001,
    // CJK Symbols and Punctuation block: the host has no Han-ideograph font,
    // so U+3001 exercises the CJK fallback path through installed Noto fonts).
    let text = "a\u{0634}\u{3001}";
    let glyphs = TypeSystem::shape_run(text, FAMILY, SIZE_PX);
    require_no_tofu(
        "multilingual \"a + SHEEN + IDEOGRAPHIC COMMA\"",
        "Latin/Arabic/CJK",
        &glyphs,
    );
    assert!(
        total_advance(&glyphs) > 0.0,
        "multilingual shaping must produce positive advances"
    );
}
