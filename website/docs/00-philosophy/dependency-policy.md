# Política de dependências

O Petunia não busca “zero dependências”. Busca **dependências mínimas, especializadas e justificadas**. Também **não busca zero código de terceiros**: copiar ou adaptar algoritmos e módulos open source compatíveis é permitido e incentivado quando reduz retrabalho, risco e custo total de manutenção.

Reimplementar um algoritmo complexo apenas para reduzir o número de crates pode aumentar tamanho, bugs e custo de manutenção.

## Reutilização de código open source — decisão de 2026-10-08

É permitido incorporar código licenciado de terceiros por **dependência direta, cópia de funções/módulos, vendor, fork ou port/adaptação**, incluindo fontes dos projetos [VectorCraft, PhotoCraft e LightCraft](#/docs/06-references/index.md). **Não exigir reimplementação apenas para o código parecer original ao Petunia.** Escolher a estratégia que mantenha a qualidade, segurança, autoria identificada, atualizações e integração mais simples.

- **Copiar é permitido, não é sempre obrigatório:** comparar import direto versus manutenção de código vendored e API do adapter. Evitar duplicar uma biblioteca inteira quando um algoritmo pequeno resolve a demanda.
- **Código copiado continua tendo proveniência externa:** registrar repo/commit/caminhos, licença precisa, copyright/NOTICE, alterações realizadas, dependências e procedimento de atualização. Não apagar headers nem chamar de clean-room original o código derivado.
- **Licenças não são universais por repositório:** verificar arquivo/crate e todos os terceiros. Licença de código não abrange automaticamente fontes, fotos, logos, marcas, modelos e pesos. LightCraft `segment` tem licença Apache-2.0 only e os pesos SAM 3 têm licença externa própria.
- **Arquitetura fechada continua sendo autoridade:** Core persistente PTND, IDs tipados, Engine, RenderModel/Render e Qt/QML não são alterados apenas para acomodar estrutura estrangeira. Adaptar data/coordinate/color/undo/preview, não importar SceneGraph ou GUI de outro produto.
- **Teste e segurança são mandatórios:** compilar, verificar no ambiente Petunia, testar regressões, licenças/transitives e comportamento de falhas antes de marcar implementação concluída.

Critérios, candidatos e registro de proveniência: [Política de Reutilização Direta](#/docs/06-references/code-reuse-policy.md). Esta permissão não dispensa decisão de dependência/ADR se a integração estrutural mudar a arquitetura.

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
