# 09.32 — ADR de Espaços de Coordenadas, Geometria Mundial e Migração

<aside>

**Status:** `ACCEPTED_V1` para o contrato de coordenadas/avaliação; a implementação está em execução na goal `P07-G05`. A fatia domain/world-geometry e um snapshot Freya toolkit-neutral já existem; o renderer vetorial/texto completo e o baseline de performance release permanecem abertos.

**Escopo:** `V1_REQUIRED` como fundação de vetores, hierarquia, seleção, canvas, exportação e evidência de performance.

**Supersede:** nada. A implementação atual possuía uma convenção ambígua entre `to_path()` e placement; este ADR torna essa relação explícita antes de qualquer migração.

</aside>

# Pergunta de decisão

Como Aubrieta deve representar geometria de objetos, placement, hierarquia, projeção de viewport e dados derivados para que uma única verdade canônica seja visível a ferramentas, hit-test, culling, renderização, exportação e automação?

# Contexto

O modelo v1 atual armazena `bounds`, `rotation`, `shape`, `parent` e `children`. As fábricas de formas paramétricas criam paths cujos pontos incluem a origem dos bounds, enquanto `local_transform()` é documentado como `T(bounds_origin) * R(rotation)`. `world_transform()` compõe esses transforms pela cadeia de pais. consumidores leem independentemente `bounds`, `to_path()`, `evaluated_path()` e rotação.

Isso permite três falhas:

- aplicar um transform mundial a um path que já contém a origem de placement, ou aplicar placement uma segunda vez;
- posicionar um filho usando bounds local como se fossem bounds mundiais;
- usar o AABB base sem rotação ou hierarquia para culling, hit-test e seleção.

O canvas Freya atual tem um snapshot privado baseado em bounds/shape persistidos. Ele não compõe geometria hierárquica/mundial. Benchmarkar esse caminho mediria uma projeção sabidamente incorreta.

# Restrições

- `AGENTS.md` exige que domain/application permaneçam independentes de toolkit e que mutações usem `Action → Command → DocumentMutator → ChangeSet`.
- `09.4` exige separar geometria local, placement mundial, invalidação, geração de cache e contextos de qualidade.
- `09.5` exige coordenadas f64 finitas, políticas de tolerância nomeadas e nenhuma normalização silenciosa.
- `09.10`, `09.10.1` e `14.6` exigem payload versionado, migração em staging, save atômico e nenhuma reinterpretação in-place de arquivos antigos.
- `10.1`, `10.5`, `10.7` e `08.23` exigem espaços explícitos, preview protegido por revisão, um gesto/um undo e interação semântica acessível.
- `14.1`, `14.2`, `14.3` e `14.8` exigem fixtures determinísticas, evidência estrutural, metadata release e comandos reproduzíveis.
- O schema nativo atual é `1`. Documentos existentes devem continuar abrindo; significado histórico ambíguo não pode ser adivinhado.

# Opções

## Opção A — Mudar `to_path()` e `evaluated_path()` imediatamente para coordenadas locais

É o modelo final mais limpo, mas muda o significado de métodos existentes enquanto consumidores ainda assumem coordenadas mundiais. Isso misturaria migração nativa, migração de ferramentas, renderer e cache.

**Rejeitada como primeiro passo.** Pode ser o destino após a API de compatibilidade e a edge de migração explícitos.

## Opção B — Adicionar APIs locais/mundiais explícitas e manter legado limitado (escolhida)

Adicionar APIs cujo espaço e direção aparecem no nome, derivadas por um único resolver. Migrar criação, ferramentas, cache, seleção, exportação e canvas incrementalmente. Manter leitores v1 somente onde o frame de origem é conhecido ou há diagnóstico de migração.

**Escolhida.** Evita quebra semântica silenciosa e torna o modelo correto executável e testável.

## Opção C — Manter a convenção atual e corrigir somente o Freya

É o menor diff, mas preserva matemática divergente no core, exporters e ferramentas. Torna o adapter dono da verdade do domínio e deixa o problema de performance na camada errada.

**Rejeitada.** Viola a ponte semântica estável e o contrato de avaliação.

# Decisão

## Espaços de coordenadas

1. **Local:** geometria editável antes do placement; não inclui `bounds` nem `rotation`.
2. **Parent/placement:** origem do frame (`bounds.x/y`) e transform local em relação ao pai. No modelo TRS atual: `T(bounds_origin) * R(rotation)`.
3. **World/pasteboard:** composição dos placements dos pais: `W_parent * (T(bounds_origin) * R(rotation))`.
4. **Screen:** projeção de `ViewportCamera`; valores de tela são apresentação/interação, nunca verdade vetorial persistida.

## APIs canônicas

A fronteira document/application deve expor operações explícitas equivalentes a:

- `base_path_local(id/object)`;
- `evaluated_path_local(id/object)`;
- `world_transform(id)`;
- `base_path_world(id)`;
- `evaluated_path_world(id)`;
- `evaluated_bounds_local(id)`;
- `evaluated_bounds_world(id)`;
- `world_aabb(id, padding policy)`.

Os nomes podem seguir a convenção do repositório, mas espaço e direção devem ser explícitos. Um resultado mundial nunca pode ser passado a uma API que aplica placement novamente.

## Bounds e rotação

`bounds` é um frame nominal de placement, não um AABB universal. Bounds locais avaliados e AABB mundial são valores derivados distintos. O AABB mundial inclui a transformação composta e padding documentado para stroke/effects. Culling, seleção e handles consomem a política mundial declarada.

## Hierarquia

Um objeto raiz pode ser posicionado diretamente no frame Surface/pasteboard. O placement de um filho é relativo ao pai. Group/ungroup/reparent preservam `W_before ≈ W_after`, salvo pedido explícito de conversão local. Containers derivam geometria dos descendentes elegíveis e não são pintados por padrão, salvo contrato de papel/pintura explícito.

Ciclos, pais pendentes, transforms não finitos e matrizes não inversíveis são rejeitados com diagnósticos estruturados. Scale/shear em reparent são rejeitados até o modelo persistido representar affine completo com segurança.

## Modifiers

Modifiers geométricos são avaliados por padrão no frame local. Seus parâmetros serializados declaram o espaço:

- Contour offset: distância local;
- quad de Perspective: frame local de origem/destino;
- retângulo de Crop: frame local;
- gradiente de Transparency: vetor local ao objeto, salvo variante document-anchored explícita.

Uma variante document-anchored, se necessária, é contrato semântico separado e versionado. Não é inferida a partir de um campo existente.

## Migração

O schema atual continua legível. Uma migração futura, quando necessária, deve:

1. analisar o payload antigo em staging;
2. classificar ambiguidade de path, modifier e hierarquia;
3. converter somente quando o mapeamento for determinístico;
4. emitir diagnósticos e digest/resumo;
5. preservar dados forward desconhecidos conforme a política;
6. validar o resultado canônico;
7. gravar novo schema somente pelo save atômico normal.

Nenhuma migração pode subtrair `group.bounds` de um path ou aplicar rotação mundial uma segunda vez. Casos ambíguos falham no preflight ou exigem política explícita de reparo.

## Canvas e interação

A camada shell/application possui um único snapshot de cena toolkit-neutral. Ele contém revisão, câmera, painter order, `ObjectId` estável, projeção mundial, estilo, visibilidade/seleção/hover e overlays com espaço explícito. Freya renderiza o snapshot e não possui segundo algoritmo de placement/culling.

Input de ponteiro é convertido uma única vez de screen para document. O preview de Select captura revisão, IDs, geometria mundial, pivot/frame e espaço de coordenadas. Preview, commit e cancel usam o mesmo resolver. Commits obsoletos são rejeitados ou rebaseados somente quando o contrato permite.

# Consequências

- O domain torna-se dono da verdade geométrica; Freya e futuros shells tornam-se adapters substituíveis.
- Paths, modifiers, hierarquia, seleção, culling, hit-test e exportação compartilham uma oracle testável.
- A primeira migração é aditiva e preserva compatibilidade; leitores legados podem existir temporariamente.
- Snapshot/cache mundial pode ser benchmarked independentemente da pintura Freya e do compositor CPU de referência.
- Consumidores existentes devem migrar deliberadamente; o compilador não detecta toda mudança de significado.
- Paridade visual não é entregue por este ADR; renderer vetorial/texto/apparência e evidência de pixels continuam obrigatórios.

# Compatibilidade

- **Contrato de arquitetura:** `SchemaMigrationRequired` para interpretação persistida local de path/modifier; `BehaviorCompatibleButObservable` para projeção mundial derivada.
- **API interna pública:** APIs aditivas primeiro; legadas são depreciadas antes de remoção.
- **Formato nativo:** versão 1 continua legível; nova interpretação é edge de schema com migração e notas de release.
- **UI:** a projeção visível pode mudar, mas Action/Command/undo/selection precisam permanecer semanticamente estáveis ou gerar diagnóstico.
- **Plugin/MCP:** nenhuma mudança de permissão neste gate; consultas futuras de frame são versionadas e informam o espaço.

# Evidência obrigatória

- fixtures raiz, filho, rotação e grupo aninhado;
- transforms finitos/inválidos;
- testes `W_before/W_after` de group/ungroup/reparent;
- testes diferenciais contra dupla transformação;
- oracle local/world path e AABB;
- concordância entre hit-test/culling/selection/handle/preview/export;
- save/reopen e preflight de migração;
- snapshot estrutural determinístico e metadata de benchmark release;
- documentação EN + pt-BR e consistência do registro ADR.

# Gatilho de revisita

Revisitar somente se um future modelo de affine persistido, modifier document-anchored, sistemas multi-surface ou requisitos medidos de renderer demonstrarem que o contrato local/world não representa um workflow V1. Uma decisão substituta deve especificar migration, invalidação, exportação e compatibilidade de UI.

# Ordem de rollout

1. Adicionar APIs explícitas local/world e testes sem mudar callers legados.
2. Migrar hierarquia e modifiers em grupos limitados de comandos.
3. Migrar cache/spatial index, seleção, hit-test e preview do Select.
4. Extrair snapshot toolkit-neutral e conectar Freya.
5. Executar fixtures determinísticas, goldens estruturais e benchmarks release.
6. Só então considerar migração persistida e renderer vetorial completo.
