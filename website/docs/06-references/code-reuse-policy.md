# Política de reutilização direta de código open source

**Decisão aprovada pelo mantenedor em 2026-10-08:** o Petunia Design Studio **pode copiar, adaptar, portar, incorporar como crate, fazer vendor ou criar forks de código aberto** de VectorCraft, PhotoCraft e LightCraft, quando isso reduzir esforço e risco de maneira verificável. **Não existe preferência obrigatória por reescrever código que já funciona.** A autorização é técnica e de produto; cada uso continua condicionado à licença efetiva do material e à compatibilidade com a arquitetura Petunia.

**Escopo:** esta política complementa [Dependências](#/docs/00-philosophy/dependency-policy.md), [Matriz de integração](#/docs/06-references/integration-matrix.md) e [Protocolo para agentes](#/docs/06-references/agent-research-protocol.md). Ela não aprova dependências arbitrárias, não comprova qualidade de componentes upstream e não autoriza copiar marcas/ativos ou artefatos de licença incompatível.

## 1. Regra principal

> **Reutilizar antes de reimplementar, quando o código for licenciado para isso, estiver tecnicamente adequado e houver evidências suficientes de correção.** Copiar código bom, com proveniência e testes, é desejável; duplicar um algoritmo difícil apenas por preferência estética não é.

Para cada funcionalidade, comparar as alternativas em ordem **de menor custo total de manutenção e integração**, não em ordem dogmática:
1. **Dependência direta de crate estável/publicada:** preferida quando API, licença, versionamento e dependências transitivas são adequados; encapsular atrás de adapter.
2. **Copiar módulo/rotinas selecionadas com attribution:** preferida quando a unidade algorítmica é pequena, útil, isolável e mantida mais facilmente no próprio Petunia que como dependência.
3. **Vendor de crate ou fork por commit:** quando vários módulos coesos são necessários, APIs upstream conflitam com o domínio Petunia ou precisamos de patches/controle de atualização; documentar delta e upstream.
4. **Portar/adaptar algoritmo:** quando conceitos e testes servem, mas tipos, unidades, gerenciamento de cor, política de erros ou modelo autoral diferem.
5. **Reimplementar somente o necessário:** quando nenhuma opção anterior atende ao contrato Petunia de modo sustentável, seguro e juridicamente viável.

**Não há proibição genérica a copiar blocos inteiros, algoritmos, tests, funções ou crates** elegíveis. Existe proibição de copiar sem verificar licenças, notices, origem de terceiros, proveniência e implicações arquiteturais.

## 2. Permissões por origem

| Origem | Licença declarada no workspace | Reuso de código | Cuidados específicos |
|---|---|---|---|
| [VectorCraft](https://github.com/storytold/vectorcraft/tree/5f92f5eb7fd194826aab2ad0844e474f13990938) | MIT **OR** Apache-2.0 | **Permitido** para arquivos/crates efetivamente sob essas licenças | Bibliotecas transitivas, notices, logos/artefatos, dependência de `linesweeper`, semântica de Path |
| [PhotoCraft](https://github.com/storytold/photocraft/tree/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7) | MIT **OR** Apache-2.0 | **Permitido** para arquivos/crates efetivamente sob essas licenças | `NOTICE`, APIs de PixelFormat/PSDs, color blending, atribuição de ícones/fontes/fixtures |
| [LightCraft](https://github.com/storytold/lightcraft/tree/832b8ada51af1e18ddc2eba87fbfe0485d150fc7) | MIT **OR** Apache-2.0, com exceções | **Permitido** onde a licença por arquivo/crate autoriza | `crates/segment`: Apache-2.0 **somente**; pesos SAM 3: **licença diferente, não OSI**, tratar separadamente |

**MIT OR Apache-2.0:** quando a fonte for efetivamente dual-licensed, podemos receber o código nos termos da opção aplicável e compatível, observadas as obrigações dessa opção. Não assumir que cada arquivo, fixture, modelo, imagem, font, biblioteca transitiva ou asset de marca compartilhe a licença raiz.

**MIT:** manter aviso de copyright e texto/aviso de licença exigidos em cópias e distribuições. **Apache-2.0:** manter texto da licença e avisos relevantes, preservar NOTICE quando aplicável e indicar alterações em arquivos modificados; observar demais requisitos da licença e eventual patent grant. Ações concretas dependem da forma de distribuição. Ao combinar proveniências, fazer revisão dos termos, não apagar cabeçalhos e não dar novo licenciamento a partes de terceiros sem autorização.

**Sem equivalência automática:** poder ler um repositório público no GitHub **não** torna toda a sua árvore open source; verificar arquivo específico e histórico quando houver código importado de outra origem. Código derivado permanece derivado mesmo após mudar identificadores e formatação. Nunca alegar implementação "clean-room original" para trechos copiados.

## 3. Critérios objetivos de escolha: copiar, vendor ou depender

Cada candidato recebe um `ReuseAssessment` registrado na PR:

| Critério | Pergunta antes da cópia |
|---|---|
| Legal/proveniência | Licença exata, notices, autoria e terceiros foram verificados? |
| Grau de maturidade | Há testes relevantes, bugs conhecidos, commits estáveis, corpus e evidência de comportamento? |
| Unidade e isolamento | Consigo copiar 1–3 módulos ou preciso do grafo inteiro de crates upstream? |
| Arquitectura | A integração preserva Core/Engine/RenderModel/Render/UI e evita egui fora de UI? |
| Modelo de dados | IDs, NodeKind, FillRule, ColorProfile, unidades e persistência têm adapter claro? |
| Dependências | Compilação, versões Rust, size, `unsafe`, libs transitivas e segurança são aceitáveis? |
| Performance | Resultado medido com mesmas fixtures vale o custo de adaptação? |
| Manutenção | Quem mantém o fork/vendored patch e como importamos correções upstream? |
| Recuperação | Result/Cancel/Undo, stale revision e fallbacks estão previstos? |
| Experiência | ActionIds/teclado/a11y/preview garantem nossa filosofia, mesmo que upstream seja diferente? |

**Critério de decisão:** copiar/adaptar quando o custo total estimado (port + testes + licença + manutenção) for menor do que implementar do zero **e** o contrato Petunia continuar correto. Não exigir reescrita cosmética de código válido. Tampouco importar um subsistema grande inteiro para evitar um adapter de dez linhas.

## 4. Candidatos prioritários a aproveitamento efetivo

Estes são **candidatos**, não imports realizados nem APIs testadas no ambiente Petunia:

| Referência / arquivo ou pasta | Abordagem preferida | Principal adaptação |
|---|---|---|
| VC [`crates/pathops/src/edit.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/pathops/src/edit.rs), `fit.rs` | **Copiar/portar algoritmo de Simplify e Bézier refit**, conservando testes elegíveis | Converter `PathData`/Anchor para `VectorPath`/NodeId, tolerância e degenerados |
| VC [`crates/pathops/src/boolean.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/pathops/src/boolean.rs), `planar.rs` | Avaliar **vendor de módulo/algoritmo de arrangement e face-walk** | Dependência `linesweeper` vs `i_overlay`, open paths, provenance e FillRules |
| VC [`crates/tools/src/pen.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/tools/src/pen.rs), `direct.rs` | Copiar helpers matemáticos/gestos **pontuais** se desacoplados; adaptar ToolController | Política híbrida Petunia, Bend explícito, Alt opcional, Qt |
| VC [`crates/effects/src/stroke/width.rs`](https://github.com/storytold/vectorcraft/blob/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/effects/src/stroke/width.rs) | Avaliar cópia do cálculo de outline/width | Perfil por arc length, source editável, RenderSnapshot |
| PC [`crates/raster/src/lib.rs`](https://github.com/storytold/photocraft/blob/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/raster/src/lib.rs) | **Candidato forte a cópia/adaptação do storage COW esparso** | `PixelFormat`, validação, masks/default pixel, autoral PTND |
| PC [`crates/paint/src/`](https://github.com/storytold/photocraft/tree/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/paint/src) | Copiar/portar partes de brush dynamics, dabs, tips e tile renderer | Normalização de input Petunia, OneEuro, units e flow/opacity |
| PC [`crates/compose/src/`](https://github.com/storytold/photocraft/tree/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/compose/src) | Adaptar CPU compositor **como referência algorítmica**; cópia pontual se compatível | Display-RGB upstream vs color pipeline/documento Petunia |
| PC [`crates/psd/src/`](https://github.com/storytold/photocraft/tree/5ecc860ac1abf8fa7e3f218c0474cdcce49fa6c7/crates/psd/src) | **Avaliar crate/dependência ou vendor de parser/writer**, em vez de recriar PSD | Segurança, I/O adapter, unknown blocks e roundtrip corpus |
| LC [`crates/preview/src/`](https://github.com/storytold/lightcraft/tree/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/preview/src) | **Candidato forte a reutilizar/adaptar cache LRU/disco e JobPool** | `PreviewKey`, cache invalidation, color space e generation |
| LC [`crates/pipeline/src/`](https://github.com/storytold/lightcraft/tree/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/pipeline/src) | Copiar/portar etapas de develop isoladas com CPU oracle | Profile, color science, stage cache e pipeline de composição |
| VC/PC [`crates/plugins/src/`](https://github.com/storytold/vectorcraft/tree/5f92f5eb7fd194826aab2ad0844e474f13990938/crates/plugins/src) | Adaptar sandbox, budgets e validações ou reutilizar crate pública | ABI/Host API versionada, capabilities, diferentes objetos e tiles |
| LC [`crates/segment/src/`](https://github.com/storytold/lightcraft/tree/832b8ada51af1e18ddc2eba87fbfe0485d150fc7/crates/segment/src) | **Somente estudo/experimento condicionado** | Apache-only code, pesos SAM License não-OSI, consentimento e hardware |

**Não copiar de imediato:** modelos de documento integrais, widget code egui, strings de identificação de Live Paint por nome de Group, índices de knot como IDs autorais, configurações de cor incompatíveis e formatos de arquivo substitutos de PTND. Partes algorítmicas internas desses módulos ainda podem ser reaproveitadas com adapters adequados.

## 5. Fluxo de cópia responsável

```text
Petunia feature + expected behavior
→ locate exact upstream file/commit + tests
→ verify license/NOTICE/dependencies and third-party provenance
→ choose direct crate / copied routines / vendor / fork / algorithm port
→ record ReuseAssessment + how to update upstream
→ bring licensed code + original notices; document modifications
→ isolate behind Petunia-owned adapter
→ adjust types, units, colors, IDs, errors, cancellation, security
→ run upstream tests applicable + new Petunia regression/roundtrip/property tests
→ check perf, headless, UI/keyboard, Undo/Redo, preview/commit
→ commit with provenance and update implementation matrix
```

### Registro mínimo exigido de código incorporado

Manter uma entrada de proveniência por import ou grupo de arquivos, por exemplo `third_party/PROVENANCE.md` ou cabeçalho de integração na própria crate, incluindo:

```yaml
component: "Petunia geometry simplify"
upstream_repo: "https://github.com/storytold/vectorcraft"
upstream_commit: "5f92f5eb7fd194826aab2ad0844e474f13990938"
upstream_files: ["crates/pathops/src/edit.rs", "crates/pathops/src/fit.rs"]
reuse_mode: "adapted-source" # direct-dependency | copied-source | vendored | fork | translated
license_expression: "MIT OR Apache-2.0" # conferir os arquivos reais
notices: ["upstream LICENSE-MIT / LICENSE-APACHE / NOTICE as applicable"]
changes: ["adapted to Petunia VectorPath/NodeId and error model"]
owner_crate: "petunia-engine"
adapter_contract: "Engine Geometry"
upstream_update_policy: "re-audit diff pinned SHA before sync"
evidence: "link de PR, cargo test results e testes de regressão"
```

**Este bloco é um exemplo de registro, não afirma que um import ocorreu.** Quando houver vendor/fork, manter patches/diff legíveis e updates controlados; não remover cabeçalhos de copyright/licença das rotinas copiadas. O caminho específico de provenance deve ser definido na primeira integração e seguido consistentemente.

## 6. Relação com a filosofia do projeto

- A filosofia Petunia incentiva **reuso responsável e pragmático**, não purismo ou NIH ("not invented here").
- Um algoritmo robusto pode ser aproveitado **literalmente** mesmo que a API de seu domínio precise de adapter.
- Reaproveitar **testes, fixtures sintéticas e benchmarks licenciados** é tão importante quanto copiar a função.
- A **GUI permanece Qt/QML via CXX-Qt**; importar matemática de um ToolController egui não significa portar egui.
- `petunia-core` permanece única fonte de verdade; código upstream não pode impor outros IDs/SceneGraph ou serialização.
- Decisões e priorização de ferramentas continuam próprias do Petunia, ainda que implementação derive de terceiros.

## 7. Critérios de aceite

Um trecho copiado só se torna **ADOTADO** após licença/proveniência registrada, build e testes efetivamente executados no Petunia, integração tipada e revisão técnica. `CODE_IDENTIFIED`, `LICENSE_REVIEWED`, `PORT_IN_PROGRESS`, `TESTED`, `ADOPTED` são estágios diferentes. Nenhum código foi copiado **por esta documentação**.

[Índice](#/docs/06-references/index.md) · [Matriz de integração](#/docs/06-references/integration-matrix.md) · [Protocolo dos agentes](#/docs/06-references/agent-research-protocol.md).
