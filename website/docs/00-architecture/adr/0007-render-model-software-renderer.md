# ADR-0007 — Render Model + software reference renderer

**Status:** Accepted

## Contexto

Engine precisa entregar geometria/texto/effects avaliados ao Render sem Render depender de Engine. Também precisamos de resultado previsível para export, CI e comparação de futuros backends GPU.

## Decisão

Criar `petunia-render-model` como crate de contrato imutável.

Backend principal/reference da v0.1:

~~~text
tiled software renderer em Rust
~~~

Vector pipeline:

~~~text
evaluated path
→ adaptive flatten
→ edge binning
→ deterministic scan conversion
→ subpixel coverage
→ linear-premultiplied compositor
~~~

## Alternativas rejeitadas

**GPU-first:** aumenta superfície de plataforma antes de existir referência semântica.

**Render receber SceneGraph:** força Render a executar regras de Engine.

**Engine depender de petunia-render:** cria acoplamento invertido.

## Consequências

- headless é nativo;
- CPU renderer define golden/reference behavior;
- GPU futuro obedece ao mesmo contrato;
- RenderGraph e caches são Derived State;
- render tile size é parâmetro medido;
- overlays ficam fora do documento.

## Referências

[Render Model](#/docs/03-render/render-model.md)  
[Pipeline](#/docs/03-render/pipeline.md)  
[Composição](#/docs/03-render/compositing-effects.md)
