# ADR-0002 — Path canônico Line + Cubic

**Status:** Accepted

## Contexto

Boolean, offset, simplify, hit-test e edição por nodes ficam muito mais complexos quando o formato persistente precisa tratar Line, Quadratic, Cubic, Arc e outras primitivas como casos equivalentes.

## Decisão

`VectorPath` persiste somente Line e Cubic Bézier.

Quadratic é convertida exatamente para Cubic. Elliptical Arc e generators permanecem paramétricos enquanto forem source própria; quando materializados, produzem Line/Cubic.

## Alternativas rejeitadas

**Persistir Quad:** não adiciona capacidade geométrica e aumenta branches no kernel.

**Persistir Arc dentro de VectorPath:** mistura geometria materializada com intenção paramétrica.

**Flat polyline como formato nativo:** perde precisão e editabilidade Bézier.

## Consequências

- De Casteljau é operação canônica de split;
- Node Tool possui modelo uniforme;
- adapters import/export fazem conversões;
- booleans podem flatten internamente sem alterar source;
- Shape/Generator continuam modelos próprios até Convert to Curves.

## Referências

[path.rs](#/docs/01-core/path.md)  
[shape.rs](#/docs/01-core/shape.md)  
[Geometry Engine](#/docs/02-engine/geometry.md)
