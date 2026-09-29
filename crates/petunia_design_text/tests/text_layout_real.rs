//! F7.2: layout religado no shaper real (sem estimativa `0.55 * size`).
//!
//! Estes testes FALHAM sobre o layout heuristico e PASSAM apos religar
//! `TextLayout` nos advances/métricas de `TypeSystem`:
//! - (a) wrap usa advances reais: "WWWW" quebra diferente de "iiii";
//! - (b) `bounds`/`baseline` vêm de ascent/descent reais da fonte;
//! - (c) segundo `layout` do mesmo story+width não re-shapeia (cache).

use petunia_design_foundation::TextStoryId;
use petunia_design_text::{
    CharacterStyle, CharacterStyleRun, TextLayout, TextOffset, TextStory, TypeSystem,
};

const FAMILY: &str = "Noto Sans";
const SIZE: f64 = 32.0;

fn styled_story(id: u64, content: &str) -> TextStory {
    let mut story = TextStory::with_content(TextStoryId::new(id), content);
    let end = TextOffset::new(story.content.len());
    story.char_runs = vec![CharacterStyleRun {
        start: TextOffset::new(0),
        end,
        style: CharacterStyle {
            font_family: FAMILY.to_string(),
            font_size: SIZE,
            ..CharacterStyle::default()
        },
    }];
    story
}

fn total_advance(text: &str) -> f32 {
    TypeSystem::shape_run(text, FAMILY, SIZE as f32)
        .iter()
        .map(|g| g.advance_px)
        .sum()
}

#[test]
fn wrap_uses_real_advances_not_fixed_factor() {
    let adv_w = total_advance("W");
    let adv_i = total_advance("i");
    assert!(
        adv_i < adv_w,
        "guarda do teste: fonte proporcional deve medir i < W, i={adv_i} W={adv_w}"
    );

    // Largura que comporta "iiii" folgada mas força "WWWW" a quebrar.
    let max_width = f64::from(adv_w) * 2.5;
    assert!(
        f64::from(adv_i) * 4.0 < max_width,
        "guarda do teste: iiii deve caber em {max_width}px, i={adv_i}"
    );

    let narrow = TextLayout::layout(&styled_story(11, "iiii"), Some(max_width));
    let wide = TextLayout::layout(&styled_story(12, "WWWW"), Some(max_width));

    assert_eq!(
        narrow.line_count, 1,
        "iiii deve caber em uma linha de {max_width}px"
    );
    assert!(
        wide.line_count > 1,
        "WWWW deve quebrar em {max_width}px com advances reais, linhas={}",
        wide.line_count
    );
    assert_ne!(
        wide.lines[0].text, narrow.lines[0].text,
        "pontos de quebra devem diferir entre larguras reais distintas"
    );
    let wide_w = wide.lines[0].bounds.x1 - wide.lines[0].bounds.x0;
    let narrow_w = narrow.lines[0].bounds.x1 - narrow.lines[0].bounds.x0;
    assert!(
        wide_w > narrow_w,
        "linha de W deve medir mais que linha de i: {wide_w} vs {narrow_w}"
    );
}

#[test]
fn baseline_and_bounds_come_from_real_font_metrics() {
    let story = styled_story(21, "Hello");
    let layout = TextLayout::layout(&story, None);
    assert_eq!(layout.line_count, 1);

    let metrics = TypeSystem::font_metrics(FAMILY, SIZE as f32);
    let line = &layout.lines[0];
    assert!(
        (line.baseline_y - f64::from(metrics.ascent_px)).abs() < 1e-6,
        "baseline deve ser o ascent real ({}), obtido {}",
        metrics.ascent_px,
        line.baseline_y
    );
    let height = line.bounds.y1 - line.bounds.y0;
    assert!(
        height >= f64::from(metrics.ascent_px + metrics.descent_px) - 1e-6,
        "altura da linha deve cobrir ascent+descent reais, altura={height}"
    );
    assert!(
        (f64::from(metrics.ascent_px) - SIZE * 0.8).abs() > 0.5,
        "guarda do teste: ascent real deve diferir da heuristica size*0.8"
    );
}

#[test]
fn second_layout_of_same_story_does_not_reshape() {
    TextLayout::clear_layout_cache();
    let story = styled_story(31, "cache-probe WW WW iiii\nsecond line here");
    TypeSystem::reset_shape_count();

    let first = TextLayout::layout(&story, Some(200.0));
    let after_first = TypeSystem::shape_count();
    assert!(
        after_first > 0,
        "primeiro layout deve passar pelo shaper ao menos uma vez"
    );

    let second = TextLayout::layout(&story, Some(200.0));
    let after_second = TypeSystem::shape_count();
    assert_eq!(
        after_second, after_first,
        "segundo layout do mesmo story+width não deve re-shapear \
         (cache): {after_first} -> {after_second}"
    );
    assert_eq!(first, second, "layout cacheado deve ser idêntico");
}
