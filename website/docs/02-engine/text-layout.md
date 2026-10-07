# Text + Layout Engine

Texto possui três problemas diferentes: shaping, line layout e page/layout flow.

## Shaping

O workspace já usa `rustybuzz`. Shaping transforma caracteres em glyphs considerando script, ligatures, features e direção.

Output derivado:

```rust
pub struct GlyphRun {
    pub font: ResolvedFontId,
    pub glyphs: Vec<PositionedGlyph>,
    pub source_range: TextRange,
}
```

Glyph IDs não são persistidos como conteúdo autoral.

## Unicode e bidi

Shaping sozinho não resolve bidi, grapheme segmentation e line breaking. Planejar módulos específicos para:
- script detection
- BiDi
- grapheme boundaries
- line-break opportunities
- hyphenation por idioma.

## Font resolution

`FontRef` do documento → resolver fonte instalada/embutida → fallback. Missing font não deve reescrever o documento automaticamente.

## Frame text

Layout recebe frame geometry + paragraph style + shaped runs e produz linhas.

Decisões:
- overflow
- inset/padding
- vertical alignment
- columns
- baseline grid
- keep-with-next
- widow/orphan futuramente.

## Text on path

O Core referencia path + offset/orientation. Engine calcula arc length e posiciona glyphs pela tangente.

## Text wrap

Wrap around object usa bounds/contour avaliado do objeto, padding e política de side. É Layout Engine, não TextObject.

## Linked frames

Resolver flow de frame A → B → C; detectar cycles; invalidar downstream quando texto ou geometry muda.

## Tables e data merge

Table é modelo de documento; cálculo de column/row sizes é Layout Engine. Data merge produz instâncias/páginas via operação explícita e auditável.

## Render

Text Engine entrega glyph positions e font references; Render cuida de glyph atlas/vector outlines/antialias. UI nunca posiciona glyph manualmente.
