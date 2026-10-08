# ADR-0008 — Little CMS 2 como CMM

**Status:** Accepted

## Contexto

`palette` resolve matemática de cor, mas não substitui gerenciamento ICC profissional, intents, BPC, proofing e transforms de perfis.

## Decisão

Usar **Little CMS 2** através da crate Rust `lcms2`, encapsulada por adapter do Color Management Engine.

Tipos/handles do CMM não atravessam APIs de domínio.

## Alternativas rejeitadas

**Implementar ICC/CMM próprio:** risco numérico e manutenção desproporcionais.

**Tratar palette como CMM:** não cobre o contrato necessário.

**Converter tudo silenciosamente para sRGB:** destrói intenção de CMYK/Spot/profiles.

## Consequências

- Assign e Convert são Commands distintos;
- transforms são cacheados por profile/content + intent + BPC + format;
- profile inválido produz erro;
- soft proof é View State;
- compositing usa contexto RGB linear derivado;
- Spot continua identidade autoral separada.

## Referências

[Color Management](#/docs/02-engine/color-management.md)  
[color.rs](#/docs/01-core/color.md)
