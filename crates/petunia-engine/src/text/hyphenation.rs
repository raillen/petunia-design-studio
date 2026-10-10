//! Pattern-based, deterministic hyphenation behind a Petunia interface.
/// Return UTF-8 byte offsets inside a word. Unsupported BCP 47 primary
/// languages return no opportunities; source text is never normalized.
pub trait HyphenationProvider {
    fn opportunities(&self, word: &str, language: &str) -> Vec<usize>;
}
#[derive(Debug, Clone, Copy, Default)]
pub struct PatternHyphenation;
impl HyphenationProvider for PatternHyphenation {
    fn opportunities(&self, word: &str, language: &str) -> Vec<usize> {
        let primary = language
            .split('-')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        let Ok(code) = primary.as_bytes().try_into() else {
            return Vec::new();
        };
        let Some(language) = hypher::Lang::from_iso(code) else {
            return Vec::new();
        };
        let mut offset = 0;
        hypher::hyphenate(word, language)
            .filter_map(|syllable| {
                offset += syllable.len();
                (offset < word.len()).then_some(offset)
            })
            .collect()
    }
}
