# Política de dependências

O Petunia não busca “zero dependências”. Busca **dependências mínimas, especializadas e justificadas**.

Reimplementar um algoritmo complexo apenas para reduzir o número de crates pode aumentar tamanho, bugs e custo de manutenção.

## Critério de entrada

Uma dependência nova deve responder satisfatoriamente:

1. Qual problema concreto resolve?
2. O problema é difícil o bastante para justificar uma crate?
3. Já existe funcionalidade equivalente na stack?
4. A API pode ficar encapsulada?
5. O projeto é mantido?
6. A licença é compatível?
7. O custo de compilação/binário/memória é aceitável?
8. A segurança é adequada ao tipo de entrada que processa?
9. A substituição futura é possível sem contaminar o domínio?

## Preferência por bibliotecas focadas

Preferir uma biblioteca pequena que resolve um problema claramente delimitado a um framework que redefine a arquitetura inteira.

Exemplo:

```text
kurbo          → curvas e geometria
i_overlay      → boolean/topologia
rustybuzz      → shaping
fontdue        → rasterização de glifos
rayon          → paralelismo de dados
```

Cada uma entra atrás de uma fronteira Petunia.

## Mapa técnico atual

Dependências e papéis já definidos pela arquitetura:

| Área | Dependência / direção | Status |
|---|---|---|
| Bézier/matemática geométrica | `kurbo` | definido |
| Boolean/topologia | `i_overlay` | definido |
| Spatial index | `rstar` | definido |
| Matemática de cor | `palette` | definido |
| ICC/CMM | Little CMS 2 via binding Rust | definido |
| Text shaping | `rustybuzz` | definido |
| Grapheme segmentation | `unicode-segmentation` | definido |
| BiDi | `unicode-bidi` | definido |
| Line break | `unicode-linebreak` | definido |
| Font metadata/outlines | `ttf-parser` | definido |
| Hyphenation | `hypher` atrás de adapter | definido |
| Glyph raster coverage | `fontdue` | definido |
| Bitmap codecs | `image` | definido |
| Data parallelism | `rayon` | definido |
| Persistência DTO | `serde` | definido |
| Identidade | `uuid` | definido |
| Errors | `thiserror` | definido |
| UI | Qt/QML via CXX-Qt | definido |
| Plugin runtime WASM | runtime concreto | medir no milestone de plugins |

O runtime WASM fica propositalmente aberto até o milestone porque tamanho de binário, startup, sandbox e throughput precisam ser medidos. Essa abertura não muda o Host API ou o formato de plugin.

## Dependência não vira modelo de domínio

Evitar:

```rust
pub struct Document {
    pub shape: external_crate::SomeShape,
}
```

quando esse tipo amarra o formato persistente e a API pública a uma biblioteca substituível.

Preferir:

```rust
pub struct VectorPath {
    // representação canônica Petunia
}
```

com conversão no Engine.

## Dependências sobrepostas

Não adotar duas bibliotecas para a mesma responsabilidade sem um motivo explícito.

Quando uma crate nova substitui outra:

```text
avaliar
→ implementar adapter
→ provar paridade
→ migrar
→ remover dependência antiga
```

“Talvez usemos depois” não é integração.

## Features

Desabilitar features desnecessárias quando a crate permitir.

Isso reduz:

- tempo de compilação;
- superfície de ataque;
- tamanho de binário;
- dependências transitivas.

## Git dependencies e forks

Dependência por commit Git ou fork exige:

- revisão fixada;
- motivo documentado;
- diferenças em relação ao upstream;
- plano de atualização ou saída.

Evitar depender de branch móvel.

## Avaliação contínua

Dependências não são decisões eternas.

Reavaliar quando:

- projeto fica abandonado;
- CVE relevante aparece;
- API necessária exige hacks;
- outra solução reduz complexidade substancialmente;
- custo medido deixa de ser aceitável.

Trocar só porque apareceu uma biblioteca mais nova não é motivo suficiente.
