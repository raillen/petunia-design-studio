# ADR-0005 — Avaliação não destrutiva

**Status:** Accepted

## Contexto

Effects, booleans live, crop, trace, shapes e adjustments precisam continuar editáveis sem substituir source por caches/materializações.

## Decisão

Persistir Source + Operations/parameters e avaliar:

~~~text
Geometry
→ Appearance
→ Post-Paint Effects
→ Clip/Mask
→ Composite
→ Output
~~~

Resultados são Derived State. DAGs de dependência são permitidas desde que acíclicas.

## Materialização

Somente Commands explícitos como Expand, Bake, Rasterize, Flatten, Convert to Curves e Trim Pixels podem trocar intenção live por resultado materializado.

## Alternativas rejeitadas

**Destructive-by-default:** perde intenção e torna History a única forma de recuperar source.

**Cache persistido como verdade:** cria stale state e acopla documento ao backend.

**Effect name + HashMap:** perde typing, migration e validação.

## Consequências

Toda operação live documenta fase, input/output, parâmetros, bounds, ROI, tolerância, failure behavior, determinismo e materialização equivalente.

## Referências

[Não destrutibilidade](#/docs/00-architecture/non-destructive.md)
